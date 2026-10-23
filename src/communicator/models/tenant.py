import uuid
from datetime import UTC, datetime

from sqlalchemy import DateTime, String
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class Tenant(Base):
    """
    An organization using Communicator -- the top of the sharing hierarchy
    every other tenant-scoped table (contacts, templates, settings, ...)
    hangs off via its own tenant_key column. NEWAX is the first tenant
    ("newax"); every tenant created afterward is created the same way,
    through POST /organizations, never hardcoded.
    """

    __tablename__ = "tenants"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tenant_key: Mapped[str] = mapped_column(String(100), unique=True, nullable=False, index=True)
    name: Mapped[str] = mapped_column(String(255), nullable=False)
    owner_account_id: Mapped[str] = mapped_column(String(255), default="", nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
