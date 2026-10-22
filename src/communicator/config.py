from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    database_url: str = "postgresql+asyncpg://postgres:development-only@localhost:5432/communicator"
    test_database_url: str = (
        "postgresql+asyncpg://communicator_test:TEST_PASSWORD@127.0.0.1:5432/communicator_test"
    )
    app_env: str = "development"

    # Harvester integration -- Communicator only ever talks to Harvester's
    # API, never its database directly. See docs/ARCHITECTURE.md.
    harvester_api_base_url: str = "http://localhost:8000/api/v1"
    harvester_api_token: str = ""

    # Reply drafting
    anthropic_api_key: str = ""
    anthropic_model: str = "claude-sonnet-5"
    reply_auto_send_confidence: float = 0.85

    # Follow-up scheduler (Phase 4)
    follow_up_quiet_hours_start: int = 9
    follow_up_quiet_hours_end: int = 18
    follow_up_weekly_message_cap: int = 3
    whatsapp_session_window_hours: int = 24

    # Hot-lead alerts (Phase 5). Slack-compatible incoming webhook URL --
    # left blank, alerts are skipped (still recorded in AuditLog either way).
    sales_alert_webhook_url: str = ""

    # Channel providers
    twilio_account_sid: str = ""
    twilio_auth_token: str = ""
    twilio_sms_from: str = ""
    twilio_whatsapp_from: str = ""
    email_provider_api_key: str = ""
    email_from_address: str = ""

    # WhatsApp provider selection -- "twilio" (session-message only, Phase 2)
    # or "meta" (direct Graph/Cloud API, ported from the standalone reply
    # bot). Meta needs its own token/phone-number-id/verify-token below.
    whatsapp_provider: str = "twilio"
    meta_whatsapp_access_token: str = ""
    meta_whatsapp_phone_number_id: str = ""
    meta_whatsapp_api_version: str = "v21.0"
    meta_webhook_verify_token: str = ""
    meta_app_secret: str = ""

    # Shared secret Postmark's inbound-webhook URL must include as
    # ?token=... -- Postmark doesn't sign inbound webhook requests, unlike
    # Twilio, so this is the only thing standing between the endpoint and
    # anyone who finds the URL.
    postmark_inbound_webhook_secret: str = ""

    dev_auth_enabled: bool = False

    # LiveProof (Verilo-style live camera verification for returns & refunds)
    # Enterprise multi-tenant SaaS scoped URL base and storage.
    public_url: str = "http://localhost:8001"
    live_proof_storage_dir: str = "var/media/live_proof"
    ecosystem_event_bus_url: str = ""
    ecosystem_gateway_key: str = ""
    # nID Single Identity Provider & Secrets Vault
    nid_base_url: str = "http://localhost:4000"
    nid_client_id: str = ""
    nid_client_secret: str = ""
    nid_redirect_uri: str = "http://localhost:8001/api/v1/auth/nid/callback"
    nid_vault_addr: str = "http://localhost:4000"
    nid_vault_role_id: str = ""
    nid_vault_secret_id: str = ""

    # Banking-Grade Zero-Trust Crypto Mesh Configuration
    neura_crypto_tenant_id: str = "default"
    neura_crypto_entity_id: str = "communicator"
    neura_crypto_key_id: str = "kid_communicator_v1"
    neura_crypto_ingress_secret_hex: str = ""
    neura_crypto_egress_secret_hex: str = ""
    neura_crypto_auth_secret_hex: str = ""
    nid_allow_mock_auth: bool = False
    neura_relay_gateway_url: str = "http://localhost:3000/api/v1/crypto-mesh/relay"
    erp_api_base_url: str = "http://localhost:3001/api/v1"
    newcom_esp_api_base_url: str = "http://localhost:8081/api/v1"

    model_config = SettingsConfigDict(env_file=".env", env_file_encoding="utf-8", extra="ignore")


def validate_test_database_url(test_url: str, production_url: str) -> None:
    if test_url == production_url:
        raise RuntimeError("TEST_DATABASE_URL must not match DATABASE_URL.")
    if "test" not in test_url.lower():
        raise RuntimeError("Test database name must contain 'test'.")


settings = Settings()
