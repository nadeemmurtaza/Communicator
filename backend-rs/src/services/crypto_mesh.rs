//! NeuraCore zero-trust transit codec.
//!
//! Wire format is the `@newax/crypto-mesh` envelope: AES-256-GCM payload,
//! HMAC-SHA512 over a canonical envelope string, and a timestamp/signature
//! anti-replay window. The egress and ingress keys are deliberately different
//! so the relay can re-encrypt without either side's key crossing the boundary.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::Aes256Gcm;
use base64::Engine;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha512;

use crate::config::Config;

const ENVELOPE_VERSION: &str = "1.0.0";
const GCM_IV_BYTES: usize = 12;
const GCM_TAG_BYTES: usize = 16;

#[derive(Debug)]
pub enum MeshError {
    /// This deployment cannot verify envelopes at all (secrets absent or short).
    Config(String),
    /// The envelope itself is wrong: bad signature, wrong tenant, tampered
    /// ciphertext, unreadable timestamp.
    Integrity(String),
    /// The envelope is well-formed but already consumed, or outside the clock
    /// window, so it cannot be trusted.
    Replay(String),
}

impl MeshError {
    pub fn detail(&self) -> String {
        match self {
            MeshError::Config(reason) => format!("CryptoMesh is not configured: {reason}"),
            MeshError::Integrity(reason) => format!("Cryptographic verification failed: {reason}"),
            MeshError::Replay(reason) => format!("Replay attack prevented: {reason}"),
        }
    }

    pub fn is_config(&self) -> bool {
        matches!(self, MeshError::Config(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitEnvelope {
    #[serde(default)]
    pub version: String,
    pub tenant_id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub iv: String,
    pub ciphertext: String,
    pub auth_tag: String,
    pub key_id: String,
    pub timestamp: String,
    pub signature: String,
}

fn b64() -> base64::engine::general_purpose::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}

/// Signatures seen since the replay window opened. One backend instance can
/// enforce this in memory; with several instances a replayed envelope could
/// still land on a process that has not seen it, which is why the timestamp
/// window is checked first.
fn seen_signatures() -> &'static Mutex<HashMap<String, i64>> {
    static CACHE: OnceLock<Mutex<HashMap<String, i64>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone)]
pub struct CryptoMesh {
    tenant_id: String,
    entity_id: String,
    key_id: String,
    ingress_key: [u8; 32],
    egress_key: [u8; 32],
    auth_secret: Vec<u8>,
    max_clock_skew_seconds: i64,
}

impl CryptoMesh {
    /// Builds the codec from configuration. Nothing is defaulted into a
    /// substitute key: absent or short secrets are an error, and the route
    /// answers 503 rather than accepting envelopes it cannot verify.
    pub fn from_config(config: &Config) -> Result<Self, MeshError> {
        if config.neura_tenant_id.trim().is_empty() {
            return Err(MeshError::Config(
                "NEURA_CRYPTO_TENANT_ID is unset, so there is no tenant identity to verify envelopes against"
                    .to_string(),
            ));
        }
        Self::from_parts(
            &config.neura_tenant_id,
            &config.neura_entity_id,
            &config.neura_key_id,
            &config.neura_ingress_secret_hex,
            &config.neura_egress_secret_hex,
            &config.neura_auth_secret_hex,
            config.neura_max_clock_skew_seconds,
        )
    }

    /// Decodes and validates the key material itself. `from_config` is the
    /// production entry point; tests use this directly so they do not have to
    /// mutate process-global environment variables.
    pub fn from_parts(
        tenant_id: &str,
        entity_id: &str,
        key_id: &str,
        ingress_hex: &str,
        egress_hex: &str,
        auth_hex: &str,
        max_clock_skew_seconds: i64,
    ) -> Result<Self, MeshError> {
        if tenant_id.trim().is_empty() {
            return Err(MeshError::Config(
                "a tenant id is required, so there is no tenant identity to verify envelopes against".to_string(),
            ));
        }
        let ingress_key = decode_key(ingress_hex, "ingress secret")?;
        let egress_key = decode_key(egress_hex, "egress secret")?;
        let auth_secret = hex::decode(auth_hex.trim())
            .map_err(|err| MeshError::Config(format!("auth secret is not hex ({err})")))?;
        if auth_secret.len() < 32 {
            return Err(MeshError::Config(format!(
                "auth secret decodes to {} bytes; HMAC-SHA512 needs at least 32",
                auth_secret.len()
            )));
        }
        Ok(Self {
            tenant_id: tenant_id.trim().to_string(),
            entity_id: if entity_id.trim().is_empty() {
                "communicator".to_string()
            } else {
                entity_id.trim().to_string()
            },
            key_id: key_id.trim().to_string(),
            ingress_key,
            egress_key,
            auth_secret,
            max_clock_skew_seconds,
        })
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn entity_id(&self) -> &str {
        &self.entity_id
    }

    pub fn relay_key_id(&self) -> Option<&str> {
        if self.key_id.is_empty() {
            None
        } else {
            Some(self.key_id.as_str())
        }
    }

    fn aad(
        version: &str,
        tenant_id: &str,
        sender_id: &str,
        recipient_id: &str,
        key_id: &str,
        timestamp: &str,
    ) -> Vec<u8> {
        format!("{version}:{tenant_id}:{sender_id}:{recipient_id}:{key_id}:{timestamp}").into_bytes()
    }

    fn canonical(
        version: &str,
        tenant_id: &str,
        sender_id: &str,
        recipient_id: &str,
        key_id: &str,
        timestamp: &str,
        iv_b64: &str,
        auth_tag_b64: &str,
        ciphertext_b64: &str,
    ) -> String {
        format!(
            "{version}|{tenant_id}|{sender_id}|{recipient_id}|{key_id}|{timestamp}|{iv_b64}|{auth_tag_b64}|{ciphertext_b64}"
        )
    }

    fn signed_mac(&self, canonical: &str) -> Result<Hmac<Sha512>, MeshError> {
        let mut mac = <Hmac<Sha512> as Mac>::new_from_slice(&self.auth_secret)
            .map_err(|err| MeshError::Config(format!("auth secret was rejected by the HMAC primitive: {err}")))?;
        mac.update(canonical.as_bytes());
        Ok(mac)
    }

    fn signature(
        &self,
        version: &str,
        tenant_id: &str,
        sender_id: &str,
        recipient_id: &str,
        key_id: &str,
        timestamp: &str,
        iv_b64: &str,
        auth_tag_b64: &str,
        ciphertext_b64: &str,
    ) -> Result<String, MeshError> {
        let canonical = Self::canonical(
            version,
            tenant_id,
            sender_id,
            recipient_id,
            key_id,
            timestamp,
            iv_b64,
            auth_tag_b64,
            ciphertext_b64,
        );
        Ok(b64().encode(self.signed_mac(&canonical)?.finalize().into_bytes()))
    }

    /// Verifies an inbound envelope end to end and returns the decrypted JSON.
    pub fn decrypt_from_relay(&self, envelope: &TransitEnvelope) -> Result<Value, MeshError> {
        if envelope.tenant_id != self.tenant_id {
            return Err(MeshError::Integrity(format!(
                "Cross-tenant violation: envelope belongs to '{}', expected '{}'.",
                envelope.tenant_id, self.tenant_id
            )));
        }
        if envelope.recipient_id != self.entity_id {
            return Err(MeshError::Integrity(format!(
                "Mismatched recipient: envelope addressed to '{}', expected '{}'.",
                envelope.recipient_id, self.entity_id
            )));
        }

        let delivered_at = DateTime::parse_from_rfc3339(&envelope.timestamp)
            .map_err(|err| MeshError::Integrity(format!("Malformed timestamp '{}': {err}", envelope.timestamp)))?;
        let skew = (Utc::now() - delivered_at.with_timezone(&Utc)).num_seconds();
        if skew.abs() > self.max_clock_skew_seconds {
            return Err(MeshError::Replay(format!(
                "timestamp is {skew}s from now, outside the {}s window",
                self.max_clock_skew_seconds
            )));
        }

        let version = if envelope.version.is_empty() {
            ENVELOPE_VERSION
        } else {
            envelope.version.as_str()
        };
        let canonical = Self::canonical(
            version,
            &envelope.tenant_id,
            &envelope.sender_id,
            &envelope.recipient_id,
            &envelope.key_id,
            &envelope.timestamp,
            &envelope.iv,
            &envelope.auth_tag,
            &envelope.ciphertext,
        );
        let provided = b64()
            .decode(&envelope.signature)
            .map_err(|_| MeshError::Integrity("Envelope signature is not base64.".to_string()))?;
        self.signed_mac(&canonical)?
            .verify_slice(&provided)
            .map_err(|_| {
                MeshError::Integrity(
                    "Envelope signature mismatch: message has been tampered with in transit.".to_string(),
                )
            })?;

        {
            let prune_before = Utc::now().timestamp() - self.max_clock_skew_seconds * 2;
            let mut seen = seen_signatures()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            seen.retain(|_, first_seen| *first_seen >= prune_before);
            if seen.contains_key(&envelope.signature) {
                return Err(MeshError::Replay(
                    "duplicate envelope signature: this envelope was already delivered".to_string(),
                ));
            }
            seen.insert(envelope.signature.clone(), Utc::now().timestamp());
        }

        let iv = b64()
            .decode(&envelope.iv)
            .map_err(|err| MeshError::Integrity(format!("iv is not base64: {err}")))?;
        if iv.len() != GCM_IV_BYTES {
            return Err(MeshError::Integrity(format!(
                "AES-GCM needs a {GCM_IV_BYTES}-byte IV, envelope carries {}",
                iv.len()
            )));
        }
        let ciphertext = b64()
            .decode(&envelope.ciphertext)
            .map_err(|err| MeshError::Integrity(format!("ciphertext is not base64: {err}")))?;
        let tag = b64()
            .decode(&envelope.auth_tag)
            .map_err(|err| MeshError::Integrity(format!("authTag is not base64: {err}")))?;
        if tag.len() != GCM_TAG_BYTES {
            return Err(MeshError::Integrity(format!(
                "AES-GCM needs a {GCM_TAG_BYTES}-byte tag, envelope carries {}",
                tag.len()
            )));
        }

        let mut combined = ciphertext;
        combined.extend_from_slice(&tag);
        let cipher = Aes256Gcm::new_from_slice(&self.egress_key)
            .map_err(|err| MeshError::Config(format!("egress key was rejected by AES-256: {err}")))?;
        let aad = Self::aad(
            version,
            &envelope.tenant_id,
            &envelope.sender_id,
            &envelope.recipient_id,
            &envelope.key_id,
            &envelope.timestamp,
        );
        let opened = cipher
            .decrypt(
                GenericArray::from_slice(&iv),
                Payload {
                    msg: combined.as_slice(),
                    aad: &aad,
                },
            )
            .map_err(|_| {
                MeshError::Integrity(
                    "Decryption failed: the egress key does not open this ciphertext (wrong key or tampered payload)."
                        .to_string(),
                )
            })?;
        serde_json::from_slice(&opened)
            .map_err(|err| MeshError::Integrity(format!("decrypted payload is not JSON: {err}")))
    }

    /// Encrypts a payload with the ingress key for relay transit. `key_id` must
    /// be configured, because the receiving side has to know which key opens it.
    pub fn encrypt_for_relay(&self, recipient_id: &str, payload: &Value) -> Result<TransitEnvelope, MeshError> {
        if recipient_id.trim().is_empty() {
            return Err(MeshError::Integrity("recipient id is required".to_string()));
        }
        if self.key_id.is_empty() {
            return Err(MeshError::Config(
                "NEURA_CRYPTO_KEY_ID is unset, so an outbound envelope cannot name the key that opens it".to_string(),
            ));
        }
        let timestamp = Utc::now().to_rfc3339();
        // uuid v4 comes from the OS CSPRNG, which is what a GCM nonce needs.
        let mut iv = [0u8; GCM_IV_BYTES];
        iv.copy_from_slice(&uuid::Uuid::new_v4().as_bytes()[..GCM_IV_BYTES]);
        let aad = Self::aad(
            ENVELOPE_VERSION,
            &self.tenant_id,
            &self.entity_id,
            recipient_id,
            &self.key_id,
            &timestamp,
        );
        let serialized = serde_json::to_vec(payload)
            .map_err(|err| MeshError::Integrity(format!("event payload could not be serialized: {err}")))?;
        let cipher = Aes256Gcm::new_from_slice(&self.ingress_key)
            .map_err(|err| MeshError::Config(format!("ingress key was rejected by AES-256: {err}")))?;
        let sealed = cipher
            .encrypt(
                GenericArray::from_slice(&iv),
                Payload {
                    msg: serialized.as_slice(),
                    aad: &aad,
                },
            )
            .map_err(|err| MeshError::Integrity(format!("encryption failed: {err}")))?;
        if sealed.len() < GCM_TAG_BYTES {
            return Err(MeshError::Integrity(
                "encryption returned a payload shorter than the auth tag".to_string(),
            ));
        }
        let split = sealed.len() - GCM_TAG_BYTES;
        let (body, tag) = sealed.split_at(split);
        let iv_b64 = b64().encode(iv);
        let ciphertext_b64 = b64().encode(body);
        let auth_tag_b64 = b64().encode(tag);
        let signature = Self::signature(
            self,
            ENVELOPE_VERSION,
            &self.tenant_id,
            &self.entity_id,
            recipient_id,
            &self.key_id,
            &timestamp,
            &iv_b64,
            &auth_tag_b64,
            &ciphertext_b64,
        )?;
        Ok(TransitEnvelope {
            version: ENVELOPE_VERSION.to_string(),
            tenant_id: self.tenant_id.clone(),
            sender_id: self.entity_id.clone(),
            recipient_id: recipient_id.to_string(),
            iv: iv_b64,
            ciphertext: ciphertext_b64,
            auth_tag: auth_tag_b64,
            key_id: self.key_id.clone(),
            timestamp,
            signature,
        })
    }
}

fn decode_key(raw: &str, name: &str) -> Result<[u8; 32], MeshError> {
    let bytes = hex::decode(raw.trim()).map_err(|err| MeshError::Config(format!("{name} is not hex ({err})")))?;
    if bytes.len() != 32 {
        return Err(MeshError::Config(format!(
            "{name} must be 64 hex characters (32 bytes) for AES-256-GCM; got {}",
            bytes.len()
        )));
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const INGRESS: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const EGRESS: &str = INGRESS;
    const RELAY_AUTH: &str = "3333333333333333333333333333333333333333333333333333333333333333";

    fn mesh(entity: &str, tenant: &str) -> CryptoMesh {
        CryptoMesh::from_parts(tenant, entity, "kid_test_1", INGRESS, EGRESS, RELAY_AUTH, 300)
            .expect("test key material must be valid")
    }

    fn event(name: &str) -> Value {
        json!({ "name": name, "payload": { "sessionId": "s-1", "decision": "APPROVED" } })
    }

    #[test]
    fn seals_and_opens_an_envelope() {
        let sender = mesh("core-newax", "tenant-a");
        let receiver = mesh("communicator", "tenant-a");
        let name = format!("order.placed.{}", uuid::Uuid::new_v4());
        let envelope = sender
            .encrypt_for_relay("communicator", &event(&name))
            .expect("sealing must succeed");
        assert_eq!(envelope.tenant_id, "tenant-a");
        assert_eq!(envelope.recipient_id, "communicator");
        let opened = receiver
            .decrypt_from_relay(&envelope)
            .expect("a correctly sealed envelope must open");
        assert_eq!(opened.get("name").and_then(Value::as_str), Some(name.as_str()));
    }

    #[test]
    fn second_delivery_of_the_same_envelope_is_a_replay() {
        let sender = mesh("core-newax", "tenant-replay");
        let receiver = mesh("communicator", "tenant-replay");
        let envelope = sender
            .encrypt_for_relay("communicator", &event("refund.decision"))
            .expect("sealing must succeed");
        receiver
            .decrypt_from_relay(&envelope)
            .expect("first delivery is accepted");
        let second = receiver.decrypt_from_relay(&envelope);
        assert!(
            matches!(second, Err(MeshError::Replay(_))),
            "duplicate signature must be a replay, got {second:?}"
        );
    }

    #[test]
    fn tampered_ciphertext_fails_authentication() {
        let sender = mesh("core-newax", "tenant-tamper");
        let receiver = mesh("communicator", "tenant-tamper");
        let mut envelope = sender
            .encrypt_for_relay("communicator", &event("order.shipped"))
            .expect("sealing must succeed");
        // Re-sign what a naive implementation would verify, so the AEAD tag is
        // the only thing left standing between the two parties.
        envelope.ciphertext = b64().encode(vec![0u8; 32]);
        envelope.signature = receiver
            .signature(
                &envelope.version,
                &envelope.tenant_id,
                &envelope.sender_id,
                &envelope.recipient_id,
                &envelope.key_id,
                &envelope.timestamp,
                &envelope.iv,
                &envelope.auth_tag,
                &envelope.ciphertext,
            )
            .expect("signing must succeed");
        let result = receiver.decrypt_from_relay(&envelope);
        assert!(
            matches!(result, Err(MeshError::Integrity(_))),
            "swapped ciphertext must not decrypt, got {result:?}"
        );
    }

    #[test]
    fn forged_signature_is_rejected() {
        let sender = mesh("core-newax", "tenant-forge");
        let receiver = mesh("communicator", "tenant-forge");
        let mut envelope = sender
            .encrypt_for_relay("communicator", &event("ticket.created"))
            .expect("sealing must succeed");
        envelope.signature = b64().encode(vec![7u8; 64]);
        let result = receiver.decrypt_from_relay(&envelope);
        assert!(
            matches!(result, Err(MeshError::Integrity(_))),
            "a signature the auth secret did not produce must fail, got {result:?}"
        );
    }

    #[test]
    fn cross_tenant_and_wrong_recipient_are_refused() {
        let sender = mesh("core-newax", "tenant-x");
        let receiver = mesh("communicator", "tenant-y");
        let envelope = sender
            .encrypt_for_relay("communicator", &event("order.placed"))
            .expect("sealing must succeed");
        let result = receiver.decrypt_from_relay(&envelope);
        assert!(
            matches!(result, Err(MeshError::Integrity(_))),
            "another tenant's envelope must be refused, got {result:?}"
        );

        let same_tenant = mesh("communicator", "tenant-x");
        let mut misaddressed = same_tenant
            .encrypt_for_relay("someone-else", &event("order.placed"))
            .expect("sealing must succeed");
        misaddressed.signature = receiver
            .signature(
                &misaddressed.version,
                &misaddressed.tenant_id,
                &misaddressed.sender_id,
                &misaddressed.recipient_id,
                &misaddressed.key_id,
                &misaddressed.timestamp,
                &misaddressed.iv,
                &misaddressed.auth_tag,
                &misaddressed.ciphertext,
            )
            .expect("signing must succeed");
        let result = receiver.decrypt_from_relay(&misaddressed);
        assert!(
            matches!(result, Err(MeshError::Integrity(_))),
            "an envelope addressed elsewhere must be refused, got {result:?}"
        );
    }

    #[test]
    fn envelope_outside_the_clock_window_is_a_replay() {
        let receiver = mesh("communicator", "tenant-skew");
        let stale = (Utc::now() - chrono::Duration::seconds(400)).to_rfc3339();
        let iv = b64().encode([3u8; GCM_IV_BYTES]);
        let body = b64().encode([4u8; 8]);
        let tag = b64().encode([5u8; GCM_TAG_BYTES]);
        let signature = receiver
            .signature("1.0.0", "tenant-skew", "core-newax", "communicator", "kid_test_1", &stale, &iv, &tag, &body)
            .expect("signing must succeed");
        let envelope = TransitEnvelope {
            version: "1.0.0".to_string(),
            tenant_id: "tenant-skew".to_string(),
            sender_id: "core-newax".to_string(),
            recipient_id: "communicator".to_string(),
            iv,
            ciphertext: body,
            auth_tag: tag,
            key_id: "kid_test_1".to_string(),
            timestamp: stale,
            signature,
        };
        let result = receiver.decrypt_from_relay(&envelope);
        assert!(
            matches!(result, Err(MeshError::Replay(_))),
            "a 400s-old envelope must be refused, got {result:?}"
        );
    }

    #[test]
    fn unusable_key_material_is_refused_rather_than_substituted() {
        assert!(matches!(
            CryptoMesh::from_parts("tenant-a", "communicator", "kid", "abcd", EGRESS, RELAY_AUTH, 300),
            Err(MeshError::Config(_))
        ));
        assert!(matches!(
            CryptoMesh::from_parts("tenant-a", "communicator", "kid", INGRESS, EGRESS, "3333", 300),
            Err(MeshError::Config(_))
        ));
        assert!(matches!(
            CryptoMesh::from_parts("", "communicator", "kid", INGRESS, EGRESS, RELAY_AUTH, 300),
            Err(MeshError::Config(_))
        ));
        assert!(matches!(
            CryptoMesh::from_parts("tenant-a", "communicator", "kid", INGRESS, EGRESS, "not-hex", 300),
            Err(MeshError::Config(_))
        ));
    }

    #[test]
    fn outbound_sealing_requires_a_key_identifier() {
        let mut sender = mesh("core-newax", "tenant-kid");
        sender.key_id = String::new();
        assert!(matches!(
            sender.encrypt_for_relay("communicator", &event("order.placed")),
            Err(MeshError::Config(_))
        ));
    }
}
