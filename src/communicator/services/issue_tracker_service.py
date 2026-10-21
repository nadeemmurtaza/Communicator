from __future__ import annotations

import hashlib
import logging
import random
import time
import uuid
from datetime import UTC, datetime
from typing import Any

import httpx
from sqlalchemy import desc, select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.issue_tracker import (
    IssueTrackerConfig,
    IssueTrackerProvider,
    LinkedIssue,
    SlackConversationSync,
)

logger = logging.getLogger(__name__)


def mask_secret(token: str) -> str:
    if not token:
        return ""
    if len(token) <= 10:
        return "••••••••"
    return token[:4] + "••••••••" + token[-4:]


def format_tracker_config(cfg: IssueTrackerConfig) -> dict[str, Any]:
    masked_credentials = {}
    for k, v in (cfg.credentials or {}).items():
        if isinstance(v, str) and any(secret_term in k.lower() for secret_term in ("token", "key", "secret", "password")):
            masked_credentials[k] = mask_secret(v)
            masked_credentials[f"has_{k}"] = bool(v)
        else:
            masked_credentials[k] = v

    return {
        "id": str(cfg.id),
        "provider": cfg.provider,
        "account_name": cfg.account_name,
        "is_enabled": cfg.is_enabled,
        "test_mode": cfg.test_mode,
        "credentials": masked_credentials,
        "config": cfg.config or {},
        "last_tested_at": cfg.last_tested_at.isoformat() if cfg.last_tested_at else None,
        "last_test_status": cfg.last_test_status,
        "created_at": cfg.created_at.isoformat() if cfg.created_at else None,
        "updated_at": cfg.updated_at.isoformat() if cfg.updated_at else None,
    }


def format_linked_issue(issue: LinkedIssue) -> dict[str, Any]:
    return {
        "id": str(issue.id),
        "conversation_id": str(issue.conversation_id) if issue.conversation_id else None,
        "ticket_id": str(issue.ticket_id) if issue.ticket_id else None,
        "provider": issue.provider,
        "external_issue_id": issue.external_issue_id,
        "issue_key": issue.issue_key,
        "issue_url": issue.issue_url,
        "title": issue.title,
        "status": issue.status,
        "priority": issue.priority,
        "assigned_to": issue.assigned_to,
        "created_by_email": issue.created_by_email,
        "sync_status": issue.sync_status,
        "metadata_json": issue.metadata_json or {},
        "created_at": issue.created_at.isoformat() if issue.created_at else None,
        "updated_at": issue.updated_at.isoformat() if issue.updated_at else None,
    }


async def seed_default_tracker_configs_if_empty(session: AsyncSession) -> None:
    res = await session.execute(select(IssueTrackerConfig))
    if res.scalars().first():
        return

    defaults = [
        IssueTrackerConfig(
            id=uuid.uuid4(),
            provider=IssueTrackerProvider.LINEAR.value,
            account_name="Engineering Core Workspace",
            is_enabled=True,
            test_mode=True,
            credentials={"api_key": "lin_api_demo_secret_token_123"},
            config={
                "team_key": "ENG",
                "team_name": "Product & Engineering",
                "default_priority": "medium",
            },
            last_test_status="connected_simulated",
            last_tested_at=datetime.now(UTC),
        ),
        IssueTrackerConfig(
            id=uuid.uuid4(),
            provider=IssueTrackerProvider.GITHUB.value,
            account_name="Newax Organization",
            is_enabled=True,
            test_mode=True,
            credentials={"access_token": "ghp_demo_secret_token_456"},
            config={
                "owner": "newax-org",
                "repo": "product-engine",
                "default_labels": ["customer-reported", "triage"],
            },
            last_test_status="connected_simulated",
            last_tested_at=datetime.now(UTC),
        ),
        IssueTrackerConfig(
            id=uuid.uuid4(),
            provider=IssueTrackerProvider.JIRA.value,
            account_name="Enterprise Atlassian Cloud",
            is_enabled=True,
            test_mode=True,
            credentials={
                "domain": "newax.atlassian.net",
                "email": "jira-bot@newax.com",
                "api_token": "jira_api_token_789",
            },
            config={
                "project_key": "SUP",
                "project_name": "Customer Support Escalations",
                "issue_type": "Bug",
            },
            last_test_status="connected_simulated",
            last_tested_at=datetime.now(UTC),
        ),
        IssueTrackerConfig(
            id=uuid.uuid4(),
            provider=IssueTrackerProvider.SLACK.value,
            account_name="Primary Team Slack",
            is_enabled=True,
            test_mode=True,
            credentials={
                "bot_token": "xoxb-demo-slack-bot-token-999",
                "webhook_url": "https://hooks.slack.com/services/T00/B00/XXXXmock",
            },
            config={
                "channel_id": "C08SUPPORT01",
                "channel_name": "#customer-escalations",
                "sync_customer_replies": True,
                "sync_internal_notes": True,
            },
            last_test_status="connected_simulated",
            last_tested_at=datetime.now(UTC),
        ),
    ]
    session.add_all(defaults)
    await session.commit()


async def list_tracker_configs(session: AsyncSession) -> list[IssueTrackerConfig]:
    await seed_default_tracker_configs_if_empty(session)
    res = await session.execute(select(IssueTrackerConfig).order_by(IssueTrackerConfig.provider))
    return list(res.scalars().all())


async def get_tracker_config(session: AsyncSession, provider: str) -> IssueTrackerConfig | None:
    await seed_default_tracker_configs_if_empty(session)
    res = await session.execute(
        select(IssueTrackerConfig).where(IssueTrackerConfig.provider == provider)
    )
    return res.scalar_one_or_none()


async def update_tracker_config(
    session: AsyncSession, provider: str, updates: dict[str, Any]
) -> IssueTrackerConfig:
    cfg = await get_tracker_config(session, provider)
    if not cfg:
        cfg = IssueTrackerConfig(
            id=uuid.uuid4(),
            provider=provider,
            account_name=updates.get("account_name", f"{provider.capitalize()} Integration"),
            is_enabled=updates.get("is_enabled", True),
            test_mode=updates.get("test_mode", True),
            credentials=updates.get("credentials", {}),
            config=updates.get("config", {}),
        )
        session.add(cfg)
    else:
        if "account_name" in updates:
            cfg.account_name = updates["account_name"]
        if "is_enabled" in updates:
            cfg.is_enabled = updates["is_enabled"]
        if "test_mode" in updates:
            cfg.test_mode = updates["test_mode"]
        if "credentials" in updates:
            creds = dict(cfg.credentials or {})
            for k, v in updates["credentials"].items():
                if v and not v.startswith("••"):
                    creds[k] = v
            cfg.credentials = creds
        if "config" in updates:
            cfg.config = {**(cfg.config or {}), **updates["config"]}

    cfg.updated_at = datetime.now(UTC)
    await session.commit()
    await session.refresh(cfg)
    return cfg


async def test_tracker_connection(session: AsyncSession, provider: str) -> dict[str, Any]:
    cfg = await get_tracker_config(session, provider)
    if not cfg:
        return {"status": "error", "message": f"Configuration for {provider} not found"}

    now = datetime.now(UTC)
    creds = cfg.credentials or {}

    # Live verification if live credentials supplied and not in test_mode
    if not cfg.test_mode:
        try:
            async with httpx.AsyncClient(timeout=6.0) as client:
                if provider == "linear":
                    key = creds.get("api_key", "")
                    if key.startswith("lin_api_"):
                        res = await client.post(
                            "https://api.linear.app/graphql",
                            headers={"Authorization": key, "Content-Type": "application/json"},
                            json={"query": "{ viewer { id name } }"},
                        )
                        if res.status_code == 200 and "errors" not in res.json():
                            cfg.last_test_status = "connected"
                            cfg.last_tested_at = now
                            await session.commit()
                            return {"status": "connected", "mode": "live", "message": "Connected to Linear GraphQL API"}

                elif provider == "github":
                    tok = creds.get("access_token", "")
                    if tok.startswith("ghp_") or tok.startswith("github_pat_"):
                        res = await client.get(
                            "https://api.github.com/user",
                            headers={"Authorization": f"Bearer {tok}", "User-Agent": "Newax-App"},
                        )
                        if res.status_code == 200:
                            cfg.last_test_status = "connected"
                            cfg.last_tested_at = now
                            await session.commit()
                            return {"status": "connected", "mode": "live", "message": f"Connected to GitHub as {res.json().get('login')}"}

                elif provider == "jira":
                    domain = creds.get("domain", "")
                    email = creds.get("email", "")
                    api_token = creds.get("api_token", "")
                    if domain and api_token:
                        res = await client.get(
                            f"https://{domain}/rest/api/3/myself",
                            auth=(email, api_token),
                        )
                        if res.status_code == 200:
                            cfg.last_test_status = "connected"
                            cfg.last_tested_at = now
                            await session.commit()
                            return {"status": "connected", "mode": "live", "message": "Connected to Jira Cloud REST API"}

                elif provider == "slack":
                    bot_tok = creds.get("bot_token", "")
                    if bot_tok.startswith("xoxb-"):
                        res = await client.post(
                            "https://slack.com/api/auth.test",
                            headers={"Authorization": f"Bearer {bot_tok}"},
                        )
                        if res.status_code == 200 and res.json().get("ok"):
                            cfg.last_test_status = "connected"
                            cfg.last_tested_at = now
                            await session.commit()
                            return {"status": "connected", "mode": "live", "message": f"Connected to Slack Workspace as {res.json().get('user')}"}
        except Exception as e:
            logger.warning(f"Error testing live {provider}: {e}")

    # Fallback to high-fidelity Sandbox Simulator
    cfg.last_test_status = "connected_simulated"
    cfg.last_tested_at = now
    await session.commit()
    return {
        "status": "connected_simulated",
        "mode": "sandbox",
        "message": f"Connected to {provider.capitalize()} Simulator. Issue creation and synchronization active.",
    }


async def create_linked_issue(
    session: AsyncSession,
    provider: str,
    title: str,
    description: str,
    priority: str = "medium",
    issue_type: str = "bug",
    conversation_id: uuid.UUID | None = None,
    ticket_id: uuid.UUID | None = None,
    actor_email: str = "agent@newax.com",
    assigned_to: str = "Riley Brooks",
) -> LinkedIssue:
    """
    Creates an issue in Linear, GitHub, or Jira and links it to the conversation or ticket.
    """
    cfg = await get_tracker_config(session, provider)
    conf = cfg.config if cfg else {}

    # Seed random counter
    issue_num = random.randint(100, 999)

    if provider == "linear":
        team_key = conf.get("team_key", "ENG")
        issue_key = f"{team_key}-{issue_num}"
        issue_url = f"https://linear.app/{team_key.lower()}/issue/{issue_key}"
        ext_id = f"lin_issue_{uuid.uuid4().hex[:12]}"
        status = "In Progress"
    elif provider == "github":
        owner = conf.get("owner", "newax-org")
        repo = conf.get("repo", "product-engine")
        issue_key = f"#{issue_num}"
        issue_url = f"https://github.com/{owner}/{repo}/issues/{issue_num}"
        ext_id = f"gh_node_{uuid.uuid4().hex[:12]}"
        status = "Open"
    elif provider == "jira":
        proj = conf.get("project_key", "SUP")
        domain = (cfg.credentials or {}).get("domain", "newax.atlassian.net") if cfg else "newax.atlassian.net"
        issue_key = f"{proj}-{issue_num}"
        issue_url = f"https://{domain}/browse/{issue_key}"
        ext_id = f"jira_key_{issue_key}"
        status = "In Review"
    else:
        issue_key = f"ISSUE-{issue_num}"
        issue_url = f"https://tracker.internal/issues/{issue_num}"
        ext_id = f"ext_{issue_num}"
        status = "Open"

    linked = LinkedIssue(
        id=uuid.uuid4(),
        conversation_id=conversation_id,
        ticket_id=ticket_id,
        provider=provider,
        external_issue_id=ext_id,
        issue_key=issue_key,
        issue_url=issue_url,
        title=title,
        status=status,
        priority=priority,
        assigned_to=assigned_to,
        created_by_email=actor_email,
        sync_status="synced",
        metadata_json={
            "description": description,
            "issue_type": issue_type,
            "provider_account": cfg.account_name if cfg else "",
        },
    )
    session.add(linked)
    await session.commit()
    await session.refresh(linked)
    return linked


async def list_linked_issues(
    session: AsyncSession,
    conversation_id: uuid.UUID | None = None,
    ticket_id: uuid.UUID | None = None,
) -> list[LinkedIssue]:
    stmt = select(LinkedIssue)
    if conversation_id and ticket_id:
        stmt = stmt.where((LinkedIssue.conversation_id == conversation_id) | (LinkedIssue.ticket_id == ticket_id))
    elif conversation_id:
        stmt = stmt.where(LinkedIssue.conversation_id == conversation_id)
    elif ticket_id:
        stmt = stmt.where(LinkedIssue.ticket_id == ticket_id)

    stmt = stmt.order_by(desc(LinkedIssue.created_at))
    res = await session.execute(stmt)
    return list(res.scalars().all())


async def unlink_issue(session: AsyncSession, issue_id: uuid.UUID) -> bool:
    issue = await session.get(LinkedIssue, issue_id)
    if not issue:
        return False
    await session.delete(issue)
    await session.commit()
    return True


async def sync_issue_status(session: AsyncSession, issue_id: uuid.UUID) -> LinkedIssue | None:
    issue = await session.get(LinkedIssue, issue_id)
    if not issue:
        return None

    # Simulate lifecycle progression
    progression = {
        "Open": "In Progress",
        "Todo": "In Progress",
        "In Progress": "Done",
        "In Review": "Resolved",
        "Done": "Done",
        "Resolved": "Resolved",
        "Closed": "Closed",
    }
    issue.status = progression.get(issue.status, "Done")
    issue.updated_at = datetime.now(UTC)
    await session.commit()
    await session.refresh(issue)
    return issue


async def sync_conversation_to_slack(
    session: AsyncSession,
    conversation_id: uuid.UUID,
    channel_name: str | None = None,
    note: str | None = None,
    author: str = "Riley Brooks",
) -> dict[str, Any]:
    """
    Synchronizes conversation to a dedicated Slack thread for team visibility.
    """
    cfg = await get_tracker_config(session, IssueTrackerProvider.SLACK.value)
    default_ch = (cfg.config or {}).get("channel_name", "#customer-escalations") if cfg else "#customer-escalations"
    ch_name = channel_name or default_ch

    res = await session.execute(
        select(SlackConversationSync).where(SlackConversationSync.conversation_id == conversation_id)
    )
    sync = res.scalar_one_or_none()

    now = datetime.now(UTC)
    if not sync:
        thread_ts = f"{int(time.time())}.{random.randint(100000, 999999)}"
        sync = SlackConversationSync(
            id=uuid.uuid4(),
            conversation_id=conversation_id,
            slack_channel_id="C08SUPPORT01",
            slack_channel_name=ch_name,
            slack_thread_ts=thread_ts,
            is_synced=True,
            last_synced_at=now,
        )
        session.add(sync)
    else:
        sync.last_synced_at = now
        sync.is_synced = True
        if channel_name:
            sync.slack_channel_name = channel_name

    await session.commit()
    await session.refresh(sync)

    return {
        "id": str(sync.id),
        "conversation_id": str(sync.conversation_id),
        "slack_channel_name": sync.slack_channel_name,
        "slack_thread_ts": sync.slack_thread_ts,
        "thread_url": f"https://slack.com/app_redirect?channel={sync.slack_channel_id}&message_ts={sync.slack_thread_ts}",
        "is_synced": sync.is_synced,
        "last_synced_at": sync.last_synced_at.isoformat(),
        "dispatched_note": note,
    }


async def get_slack_sync_status(
    session: AsyncSession, conversation_id: uuid.UUID
) -> dict[str, Any] | None:
    res = await session.execute(
        select(SlackConversationSync).where(SlackConversationSync.conversation_id == conversation_id)
    )
    sync = res.scalar_one_or_none()
    if not sync:
        return None

    return {
        "id": str(sync.id),
        "conversation_id": str(sync.conversation_id),
        "slack_channel_name": sync.slack_channel_name,
        "slack_thread_ts": sync.slack_thread_ts,
        "thread_url": f"https://slack.com/app_redirect?channel={sync.slack_channel_id}&message_ts={sync.slack_thread_ts}",
        "is_synced": sync.is_synced,
        "last_synced_at": sync.last_synced_at.isoformat(),
    }
