import enum
import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import Boolean, DateTime, ForeignKey, String, Text
from sqlalchemy.dialects.postgresql import JSONB, UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class IssueTrackerProvider(str, enum.Enum):
    LINEAR = "linear"
    GITHUB = "github"
    JIRA = "jira"
    SLACK = "slack"


class IssueTrackerConfig(Base):
    """
    Configuration and API credentials for external issue trackers
    (Linear, GitHub Issues, Jira) and Slack workspace sync.
    """

    __tablename__ = "issue_tracker_configs"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    provider: Mapped[str] = mapped_column(String(50), nullable=False, unique=True, index=True)
    account_name: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    is_enabled: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    test_mode: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    
    # Encrypted / stored credentials:
    # Linear: {"api_key": "lin_api_..."}
    # GitHub: {"access_token": "ghp_..."}
    # Jira: {"domain": "acme.atlassian.net", "email": "agent@acme.com", "api_token": "..."}
    # Slack: {"bot_token": "xoxb-...", "webhook_url": "https://hooks.slack.com/..."}
    credentials: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)
    
    # Workspace defaults:
    # Linear: {"team_id": "...", "team_key": "ENG", "default_project_id": "..."}
    # GitHub: {"owner": "acme-corp", "repo": "product-engine"}
    # Jira: {"project_key": "SUP", "issue_type": "Bug"}
    # Slack: {"channel_id": "C012345678", "channel_name": "#customer-support", "sync_replies": True}
    config: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)

    last_tested_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    last_test_status: Mapped[str] = mapped_column(String(50), default="untested", nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )


class LinkedIssue(Base):
    """
    Links a conversation or support ticket to an external issue in Linear, GitHub, or Jira.
    """

    __tablename__ = "linked_issues"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    conversation_id: Mapped[uuid.UUID | None] = mapped_column(
        UUID(as_uuid=True),
        ForeignKey("conversations.id", ondelete="CASCADE"),
        nullable=True,
        index=True,
    )
    ticket_id: Mapped[uuid.UUID | None] = mapped_column(
        UUID(as_uuid=True),
        ForeignKey("tickets.id", ondelete="CASCADE"),
        nullable=True,
        index=True,
    )

    provider: Mapped[str] = mapped_column(String(50), nullable=False)  # "linear" | "github" | "jira"
    external_issue_id: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    issue_key: Mapped[str] = mapped_column(String(100), nullable=False, index=True)  # e.g. "ENG-142", "#84", "PROJ-205"
    issue_url: Mapped[str] = mapped_column(String(1024), nullable=False)
    title: Mapped[str] = mapped_column(String(512), nullable=False)
    status: Mapped[str] = mapped_column(String(100), default="Open", nullable=False)
    priority: Mapped[str] = mapped_column(String(50), default="medium", nullable=False)
    assigned_to: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    created_by_email: Mapped[str] = mapped_column(String(255), default="agent@newax.com", nullable=False)
    sync_status: Mapped[str] = mapped_column(String(50), default="synced", nullable=False)
    metadata_json: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )


class SlackConversationSync(Base):
    """
    Tracks real-time conversation synchronization to an internal Slack channel and thread.
    """

    __tablename__ = "slack_conversation_syncs"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    conversation_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True),
        ForeignKey("conversations.id", ondelete="CASCADE"),
        nullable=False,
        unique=True,
        index=True,
    )
    slack_channel_id: Mapped[str] = mapped_column(String(100), nullable=False)
    slack_channel_name: Mapped[str] = mapped_column(String(255), default="#customer-support", nullable=False)
    slack_thread_ts: Mapped[str] = mapped_column(String(100), nullable=False)  # Slack thread identifier
    is_synced: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)

    last_synced_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC), nullable=False
    )
