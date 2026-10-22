import uuid
from datetime import UTC, datetime

from sqlalchemy import Boolean, DateTime, String
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class DiscoveryFlowTemplate(Base):
    """
    A tenant's business-inquiry discovery flow -- the staged questions,
    option sets, problem/capability taxonomy, and scoring weights that
    drive discovery_flow.py. Communicator is multi-tenant: each tenant
    (identified by `tenant_key`, e.g. "newax") gets its own template so
    the same engine serves every organization's WhatsApp automation
    without any tenant's business content living in code. NEWAX is the
    first tenant, seeded as data, not special-cased logic.
    """

    __tablename__ = "discovery_flow_templates"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tenant_key: Mapped[str] = mapped_column(String(100), nullable=False, unique=True, index=True)
    name: Mapped[str] = mapped_column(String(255), nullable=False)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)

    # Ordered list of {key, question, free_text?|options:[[value,label]]}
    steps: Mapped[list[dict]] = mapped_column(JSONB, default=list, nullable=False)

    # {category_key: [keyword, ...]} -- used to tag problem_description
    problem_category_keywords: Mapped[dict] = mapped_column(JSONB, default=dict, nullable=False)

    # {capability_key: [keyword, ...]} -- maps the problem to the tenant's
    # own service lines (for NEWAX: enterprise_software / intelligent_automation
    # / connected_infrastructure; another tenant defines its own set)
    capability_keywords: Mapped[dict] = mapped_column(JSONB, default=dict, nullable=False)
    capability_labels: Mapped[dict] = mapped_column(JSONB, default=dict, nullable=False)

    # Scoring weights per dimension, and the role/timeline/budget point maps
    scoring_weights: Mapped[dict] = mapped_column(JSONB, default=dict, nullable=False)

    # Copy used around the flow (intro line, closing line)
    intro_message: Mapped[str] = mapped_column(String(500), default="", nullable=False)
    closing_message: Mapped[str] = mapped_column(String(500), default="", nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
    )
