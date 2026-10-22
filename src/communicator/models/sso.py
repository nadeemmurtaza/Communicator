from __future__ import annotations

import uuid
from datetime import UTC, datetime

from sqlalchemy import Boolean, DateTime, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base

SINGLETON_SSO_CONFIG_ID = uuid.UUID("00000000-0000-0000-0000-000000000055")


class SSOConfig(Base):
    """
    Enterprise Single Sign-On (SSO) and SAML 2.0 Identity Provider configuration.
    Supports Okta, Azure AD / Microsoft Entra ID, Google Workspace, OneLogin, and custom SAML 2.0 IdPs.
    """

    __tablename__ = "sso_configs"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    is_enabled: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    provider_type: Mapped[str] = mapped_column(
        String(50), default="saml", nullable=False
    )  # saml | okta | azure_ad | google_workspace | onelogin

    # Identity Provider (IdP) Settings
    idp_entity_id: Mapped[str] = mapped_column(String(500), default="", nullable=False)
    idp_sso_url: Mapped[str] = mapped_column(String(1000), default="", nullable=False)
    idp_x509_cert: Mapped[str] = mapped_column(Text, default="", nullable=False)

    # Service Provider (SP) Settings (Our Platform)
    sp_entity_id: Mapped[str] = mapped_column(
        String(500), default="urn:communicator:sp", nullable=False
    )
    sp_acs_url: Mapped[str] = mapped_column(
        String(1000), default="/api/v1/sso/saml/acs", nullable=False
    )

    # Enterprise Access & Enforcement
    enforced_domains: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)
    allow_unlinked_passwords: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)

    # Just-in-Time (JIT) Provisioning & RBAC Mapping
    jit_provisioning_enabled: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    default_role: Mapped[str] = mapped_column(String(50), default="agent", nullable=False)
    attribute_mapping: Mapped[dict[str, str]] = mapped_column(
        JSONB,
        default=lambda: {
            "email": "email",
            "name": "name",
            "first_name": "first_name",
            "last_name": "last_name",
            "role": "role",
        },
        nullable=False,
    )

    # Health & Verification Status
    last_tested_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    last_test_status: Mapped[str] = mapped_column(String(50), default="untested", nullable=False)
    last_test_message: Mapped[str] = mapped_column(String(500), default="", nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )
