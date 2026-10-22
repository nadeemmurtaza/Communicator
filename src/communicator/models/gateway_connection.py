import enum
import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import Boolean, DateTime, Integer, String
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class GatewayType(str, enum.Enum):
    SMS = "sms"
    EMAIL = "email"
    VOICE_SIP = "voice_sip"


class SMSProvider(str, enum.Enum):
    TWILIO = "twilio"
    MESSAGEBIRD = "messagebird"
    VONAGE = "vonage"
    PLIVO = "plivo"
    SINCH = "sinch"
    AWS_SNS = "aws_sns"
    INFOBIP = "infobip"
    CUSTOM_HTTP = "custom_http"


class EmailProvider(str, enum.Enum):
    SMTP = "smtp"
    SENDGRID = "sendgrid"
    AWS_SES = "aws_ses"
    MAILGUN = "mailgun"
    POSTMARK = "postmark"
    RESEND = "resend"
    CUSTOM_WEBHOOK = "custom_webhook"


class VoiceSIPProvider(str, enum.Enum):
    TWILIO_VOICE = "twilio_voice"
    TELNYX = "telnyx"
    PLIVO_VOICE = "plivo_voice"
    VONAGE_VOICE = "vonage_voice"
    SIP_TRUNK = "sip_trunk"
    ASTERISK_FREESWITCH = "asterisk_freeswitch"
    CUSTOM_WEBRTC = "custom_webrtc"


class ChannelGatewayConnection(Base):
    """
    Universal multi-provider gateway connection.
    Enables connecting ANY SMS provider, ANY Email provider, and ANY VoIP/SIP provider.
    """

    __tablename__ = "channel_gateway_connections"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    gateway_type: Mapped[str] = mapped_column(String(50), nullable=False)  # sms, email, voice_sip
    provider: Mapped[str] = mapped_column(String(50), nullable=False)
    name: Mapped[str] = mapped_column(String(255), nullable=False)
    is_default: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)

    # Credentials (tokens, secrets, passwords - masked on retrieval)
    credentials: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)

    # Configuration (endpoints, headers, ports, SIP domains, caller IDs, templates)
    config: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)

    # Test status & health diagnostics
    test_status: Mapped[str] = mapped_column(String(50), default="untested", nullable=False)
    latency_ms: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    last_tested_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )
