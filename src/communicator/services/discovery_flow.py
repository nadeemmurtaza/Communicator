"""
Business Inquiry Discovery Flow -- a staged WhatsApp conversation that
understands a visitor's business problem before recommending anything,
then maps it to the tenant's own capabilities, scores it, and creates
an Inquiry record for human review.

This engine is tenant-agnostic. Every piece of business content -- the
questions, option sets, problem/capability taxonomy, and scoring weights
-- comes from a DiscoveryFlowTemplate row looked up by tenant_key, never
hardcoded here. NEWAX is the first tenant (seeded via
seed_default_discovery_template / scripts/seed_newax_tenant.py); any new
tenant gets its own template row and the same engine serves it.

Progress is kept in Conversation.custom_attributes["discovery"] so the
flow survives process restarts and never repeats an already-answered
question.
"""

from __future__ import annotations

import logging
from typing import Any

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels import get_provider
from communicator.channels.whatsapp import MetaWhatsAppProvider
from communicator.models.contact import Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.discovery_flow_template import DiscoveryFlowTemplate
from communicator.models.inbox import Inbox
from communicator.models.inquiry import Inquiry, InquiryStatus
from communicator.models.message import Message, MessageDirection, MessageStatus
from communicator.services.messaging import send_outbound_message
from communicator.services.relationship_profile import get_relationship_summary

logger = logging.getLogger(__name__)

PAYLOAD_PREFIX = "disc"
START_PAYLOAD = "action:discuss_requirement"
DEFAULT_TENANT_KEY = "newax"


async def resolve_tenant_key(session: AsyncSession, contact: Contact) -> str:
    """
    Resolves which tenant's flow should handle this contact. For now that's
    the tenant configured on the inbox the contact's channel uses; falls
    back to the default tenant if none is set (single-tenant deployments).
    """
    stmt = select(Inbox).where(Inbox.channel_type == contact.channel, Inbox.is_active.is_(True))
    inbox = (await session.execute(stmt)).scalars().first()
    if inbox is not None:
        tenant_key = (inbox.credentials or {}).get("tenantKey")
        if tenant_key:
            return tenant_key
    return DEFAULT_TENANT_KEY


async def get_template(session: AsyncSession, tenant_key: str) -> DiscoveryFlowTemplate | None:
    stmt = select(DiscoveryFlowTemplate).where(
        DiscoveryFlowTemplate.tenant_key == tenant_key, DiscoveryFlowTemplate.is_active.is_(True)
    )
    return (await session.execute(stmt)).scalar_one_or_none()


def _classify(text: str, keyword_map: dict[str, list[str]]) -> list[str]:
    lowered = text.lower()
    return [key for key, kws in keyword_map.items() if any(kw in lowered for kw in kws)]


def _score_inquiry(inquiry: Inquiry, weights: dict) -> tuple[int, dict]:
    """Transparent, additive scoring -- weights come from the tenant's template."""
    factors: dict[str, int] = {}

    max_clarity = weights.get("problem_clarity_max", 20)
    factors["problem_clarity"] = min(max_clarity, len(inquiry.problem_description.split()) // 3)
    factors["operational_impact"] = (
        weights.get("operational_impact_max", 20) if inquiry.reported_impact else 5
    )
    factors["capability_alignment"] = (
        weights.get("capability_alignment_max", 20) if inquiry.capability_mapping else 0
    )
    factors["decision_access"] = weights.get("decision_access_by_role", {}).get(
        inquiry.contact_role, weights.get("decision_access_default", 5)
    )
    factors["timeline_readiness"] = weights.get("timeline_readiness_by_value", {}).get(
        inquiry.timeline, weights.get("timeline_readiness_default", 2)
    )
    factors["budget_alignment"] = weights.get("budget_alignment_by_value", {}).get(
        inquiry.budget_range, weights.get("budget_alignment_default", 3)
    )

    return sum(factors.values()), factors


async def _send(session: AsyncSession, contact: Contact, conversation: Conversation, body: str) -> None:
    provider = await get_provider(contact.channel, session)
    await send_outbound_message(session, contact, provider, body)


async def _send_step_question(
    session: AsyncSession, contact: Contact, conversation: Conversation, step: dict
) -> None:
    provider = await get_provider(contact.channel, session)
    if step.get("free_text") or not isinstance(provider, MetaWhatsAppProvider):
        await _send(session, contact, conversation, step["question"])
        return

    options = step.get("options", [])
    rows = [
        {"id": f"{PAYLOAD_PREFIX}:{step['key']}:{value}", "title": label}
        for value, label in options
    ]
    try:
        if len(options) <= 3:
            await provider.send_interactive_buttons(
                to=contact.value,
                body_text=step["question"],
                buttons=[{"id": r["id"], "title": r["title"]} for r in rows],
            )
        else:
            await provider.send_interactive_list(
                to=contact.value,
                body_text=step["question"],
                button_title="Choose an option",
                sections=[{"title": "Options", "rows": rows}],
            )
        session.add(
            Message(
                conversation_id=conversation.id,
                direction=MessageDirection.OUTBOUND,
                channel=contact.channel,
                body=step["question"],
                provider="meta",
                provider_message_id=f"discovery_{step['key']}",
                status=MessageStatus.SENT,
                intent="discovery_flow",
            )
        )
        await session.flush()
    except Exception:  # noqa: BLE001
        logger.exception("Failed to send discovery step %s", step["key"])
        await _send(session, contact, conversation, step["question"])


def is_discovery_start(payload_id: str | None, body: str) -> bool:
    if payload_id == START_PAYLOAD:
        return True
    return body.strip().lower() in {"1", "discuss a business requirement", "discuss requirement"}


async def start_discovery(session: AsyncSession, conversation: Conversation, contact: Contact) -> None:
    tenant_key = await resolve_tenant_key(session, contact)
    template = await get_template(session, tenant_key)
    if template is None or not template.steps:
        logger.warning("No discovery flow template for tenant %s; skipping discovery", tenant_key)
        return

    # Returning customer: don't re-ask what a prior Inquiry already told us.
    # Only prefills a step's answer when the field is actually present on
    # record -- a blank/unknown field still gets asked normally.
    prefilled: dict[str, str] = {}
    relationship = await get_relationship_summary(session, contact.id)
    prior_inquiry = relationship.get("latest_inquiry")
    if prior_inquiry:
        if prior_inquiry.get("organization_name"):
            prefilled["organization_name"] = prior_inquiry["organization_name"]
        if prior_inquiry.get("industry"):
            prefilled["industry"] = prior_inquiry["industry"]

    conversation.custom_attributes = {
        **(conversation.custom_attributes or {}),
        "discovery": {"tenant_key": tenant_key, "step": 0, "answers": {}, "prefilled": prefilled},
    }
    conversation.state = ConversationState.ENGAGED
    await session.flush()

    if template.intro_message:
        intro = template.intro_message
        if relationship["relationship_stage"] in ("active_prospect", "client"):
            intro = "Good to hear from you again. " + intro
        await _send(session, contact, conversation, intro)
    await _advance_past_prefilled(session, contact, conversation, template)


async def _advance_past_prefilled(
    session: AsyncSession, contact: Contact, conversation: Conversation, template: DiscoveryFlowTemplate
) -> None:
    state = dict((conversation.custom_attributes or {}).get("discovery") or {})
    prefilled = state.get("prefilled", {})
    answers = dict(state.get("answers", {}))
    step_index = state.get("step", 0)
    steps = template.steps

    while step_index < len(steps) and steps[step_index]["key"] in prefilled:
        answers[steps[step_index]["key"]] = prefilled[steps[step_index]["key"]]
        step_index += 1

    state = {**state, "step": step_index, "answers": answers}
    conversation.custom_attributes = {**(conversation.custom_attributes or {}), "discovery": state}
    await session.flush()

    if step_index < len(steps):
        await _send_step_question(session, contact, conversation, steps[step_index])
    else:
        await _finish_discovery(session, conversation, contact, answers, template)


def is_discovery_active(conversation: Conversation) -> bool:
    return bool((conversation.custom_attributes or {}).get("discovery"))


async def handle_discovery_reply(
    session: AsyncSession,
    conversation: Conversation,
    contact: Contact,
    body: str,
    payload_id: str | None,
) -> None:
    state = dict((conversation.custom_attributes or {}).get("discovery") or {})
    tenant_key = state.get("tenant_key", DEFAULT_TENANT_KEY)
    template = await get_template(session, tenant_key)
    if template is None or not template.steps:
        return

    steps: list[dict[str, Any]] = template.steps
    step_index = state.get("step", 0)
    answers = dict(state.get("answers", {}))

    if step_index >= len(steps):
        return

    current = steps[step_index]
    if payload_id and payload_id.startswith(f"{PAYLOAD_PREFIX}:{current['key']}:"):
        answers[current["key"]] = payload_id.split(":", 2)[2]
    else:
        answers[current["key"]] = body.strip()

    step_index += 1
    state = {"tenant_key": tenant_key, "step": step_index, "answers": answers}
    conversation.custom_attributes = {**(conversation.custom_attributes or {}), "discovery": state}
    await session.flush()

    if step_index < len(steps):
        await _send_step_question(session, contact, conversation, steps[step_index])
        return

    await _finish_discovery(session, conversation, contact, answers, template)


async def _finish_discovery(
    session: AsyncSession,
    conversation: Conversation,
    contact: Contact,
    answers: dict,
    template: DiscoveryFlowTemplate,
) -> None:
    problem_text = answers.get("problem_description", "")
    problem_categories = _classify(problem_text, template.problem_category_keywords)
    capability_mapping = _classify(
        f"{problem_text} {answers.get('industry', '')}", template.capability_keywords
    )

    inquiry = Inquiry(
        tenant_key=template.tenant_key,
        conversation_id=conversation.id,
        contact_id=contact.id,
        lead_id=contact.lead_id,
        organization_name=answers.get("organization_name", ""),
        industry=answers.get("industry", ""),
        contact_role=answers.get("contact_role", ""),
        org_size_range=answers.get("org_size_range", ""),
        problem_description=problem_text,
        problem_categories=problem_categories,
        reported_impact=[answers.get("reported_impact", "")] if answers.get("reported_impact") else [],
        frequency=answers.get("frequency", ""),
        desired_outcome=answers.get("desired_outcome", ""),
        capability_mapping=capability_mapping,
        timeline=answers.get("timeline", ""),
        budget_range=answers.get("budget_range", ""),
        status=InquiryStatus.AWAITING_REVIEW,
        next_action="Human discovery discussion",
    )
    score, factors = _score_inquiry(inquiry, template.scoring_weights)
    inquiry.qualification_score = score
    inquiry.qualification_factors = factors
    session.add(inquiry)

    conversation.custom_attributes = {**(conversation.custom_attributes or {}), "discovery": None}
    conversation.state = ConversationState.HOT_LEAD
    conversation.human_takeover = True
    await session.flush()

    closing = template.closing_message or (
        "Thanks — that gives us a clear picture. Someone from our team will follow up shortly."
    )
    await _send(session, contact, conversation, closing)

    logger.info(
        "New inquiry %s (tenant=%s): score=%s org=%s capability=%s",
        inquiry.id, template.tenant_key, score, inquiry.organization_name, capability_mapping,
    )
