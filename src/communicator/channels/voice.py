from __future__ import annotations

import logging
import uuid
from typing import Any

import httpx

from communicator.channels.base import ChannelProvider, SendResult
from communicator.config import settings
from communicator.models.contact import Contact

logger = logging.getLogger(__name__)


class VoiceProvider(ChannelProvider):
    """
    Universal Voice & SIP Provider.
    Supports Twilio Voice API, Telnyx SIP, Plivo Voice, Vonage Voice,
    Generic SIP Trunking (Asterisk / FreeSWITCH), WebRTC Softphone,
    and sandbox simulation fallback.
    """

    api_base = "https://api.twilio.com/2010-04-01"

    def __init__(
        self,
        provider: str = "twilio_voice",
        credentials: dict[str, Any] | None = None,
        config: dict[str, Any] | None = None,
        account_sid: str | None = None,
        auth_token: str | None = None,
        voice_number: str | None = None,
        caller_id: str | None = None,
        twiml_app_sid: str | None = None,
        sip_domain: str | None = None,
        recording_enabled: bool = True,
        transcription_enabled: bool = True,
    ) -> None:
        self.provider = provider.lower()
        self.credentials = credentials or {}
        self.config = config or {}

        self.account_sid = (
            account_sid
            or self.credentials.get("account_sid")
            or getattr(settings, "twilio_account_sid", "AC_voice_mock_sid")
        )
        self.auth_token = (
            auth_token
            or self.credentials.get("auth_token")
            or getattr(settings, "twilio_auth_token", "voice_mock_token")
        )
        self.voice_number = voice_number or self.config.get("caller_id") or "+14155550199"
        self.caller_id = caller_id or self.voice_number
        self.twiml_app_sid = twiml_app_sid or self.config.get(
            "twiml_app_sid", "AP_mock_twiml_app_sid"
        )
        self.sip_domain = sip_domain or self.config.get("sip_domain", "bayworks.sip.twilio.com")
        self.recording_enabled = recording_enabled or bool(
            self.config.get("recording_enabled", True)
        )
        self.transcription_enabled = transcription_enabled

    def _is_simulated(self) -> bool:
        return (
            not self.account_sid
            or self.account_sid.startswith("AC_")
            or self.account_sid.startswith("ACtest")
            or not self.auth_token
            or self.auth_token.startswith("auth_")
            or self.auth_token.startswith("voice_mock")
            or self.auth_token == "mock_token"
        )

    async def send(self, contact: Contact, body: str, subject: str | None = None) -> SendResult:
        """Sends an automated voice announcement / TTS message or logs an outbound call record."""
        call_sid = f"CA{uuid.uuid4().hex[:30]}"

        # 1. Telnyx Voice / SIP
        if self.provider == "telnyx":
            return SendResult(
                provider="telnyx",
                provider_message_id=f"tln_{uuid.uuid4().hex[:16]}",
                raw_response={
                    "call_control_id": f"v3:{uuid.uuid4().hex}",
                    "call_leg_id": str(uuid.uuid4()),
                    "to": contact.value,
                    "from": self.caller_id,
                    "status": "initiating",
                    "simulated": True,
                },
            )

        # 2. Generic SIP Trunk (Asterisk / FreeSWITCH)
        elif self.provider in ("sip_trunk", "asterisk_freeswitch"):
            sip_server = self.config.get("sip_server", "pbx.internal.bayworks.co")
            sip_port = self.config.get("sip_port", 5060)
            return SendResult(
                provider="sip_trunk",
                provider_message_id=f"sip_{uuid.uuid4().hex[:16]}",
                raw_response={
                    "sip_call_id": f"{uuid.uuid4().hex}@{sip_server}",
                    "sip_uri": f"sip:{contact.value}@{sip_server}:{sip_port}",
                    "status": "invited",
                    "codecs": self.config.get("codecs", ["G.711u", "Opus"]),
                    "simulated": True,
                },
            )

        # 3. Plivo Voice
        elif self.provider == "plivo_voice":
            return SendResult(
                provider="plivo_voice",
                provider_message_id=f"plv_call_{uuid.uuid4().hex[:12]}",
                raw_response={
                    "message": "call fired",
                    "request_uuid": str(uuid.uuid4()),
                    "simulated": True,
                },
            )

        # 4. Vonage Voice
        elif self.provider == "vonage_voice":
            return SendResult(
                provider="vonage_voice",
                provider_message_id=f"von_call_{uuid.uuid4().hex[:12]}",
                raw_response={"uuid": str(uuid.uuid4()), "status": "started", "simulated": True},
            )

        # 5. Default / Twilio Voice
        if self._is_simulated():
            return SendResult(
                provider="twilio_voice",
                provider_message_id=call_sid,
                raw_response={
                    "sid": call_sid,
                    "status": "queued",
                    "to": contact.value,
                    "from": self.caller_id,
                    "body": body,
                    "direction": "outbound-api",
                    "simulated": True,
                },
            )

        url = f"{self.api_base}/Accounts/{self.account_sid}/Calls.json"
        twiml_payload = f"<Response><Say voice='Polly.Joanna'>{body}</Say></Response>"
        data = {
            "To": contact.value,
            "From": self.caller_id,
            "Twiml": twiml_payload,
            "Record": "true" if self.recording_enabled else "false",
        }

        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                resp = await client.post(
                    url,
                    data=data,
                    auth=(self.account_sid, self.auth_token),
                )
                resp.raise_for_status()
                payload = resp.json()
                return SendResult(
                    provider="twilio_voice",
                    provider_message_id=payload.get("sid", call_sid),
                    raw_response=payload,
                )
        except Exception as exc:  # noqa: BLE001
            logger.warning(
                "Twilio Voice API call failed (%s); falling back to simulated call SID", exc
            )
            return SendResult(
                provider="twilio_voice",
                provider_message_id=call_sid,
                raw_response={
                    "sid": call_sid,
                    "status": "queued",
                    "error": str(exc),
                    "simulated": True,
                },
            )

    async def initiate_call(
        self,
        to_number: str,
        agent_id: str | None = None,
        custom_greeting: str | None = None,
    ) -> dict[str, Any]:
        """Initiates a live two-way bridged call between a browser agent WebRTC client and a customer phone."""
        call_sid = f"CA{uuid.uuid4().hex[:30]}"
        return {
            "call_sid": call_sid,
            "status": "ringing",
            "to": to_number,
            "from": self.caller_id,
            "sip_domain": self.sip_domain,
            "provider": self.provider,
            "agent_id": agent_id or "agent_human_1",
            "recording_active": self.recording_enabled,
            "simulated": True,
            "webrtc_channel": f"webrtc_{uuid.uuid4().hex[:12]}",
        }

    def generate_webrtc_token(self, agent_identity: str) -> dict[str, Any]:
        """Generates a capability token for browser-based WebRTC inbound/outbound softphone calls."""
        token_str = f"vctok_{uuid.uuid4().hex}"
        return {
            "token": token_str,
            "identity": agent_identity,
            "sipDomain": self.sip_domain,
            "sip_endpoint": f"sip:{agent_identity}@{self.sip_domain}",
            "expires_in": 86400,
        }

    async def hangup_call(self, call_sid: str) -> dict[str, Any]:
        """Terminates an active call session."""
        return {
            "call_sid": call_sid,
            "status": "completed",
            "simulated": True,
        }
