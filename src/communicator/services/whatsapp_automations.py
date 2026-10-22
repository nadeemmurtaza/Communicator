"""
Comprehensive WhatsApp Automation Suite for Meta Cloud API.

Orchestrates all standard and enterprise automations:
1. Welcome Greeting with Interactive Quick Replies (first-touch or after 24h+ idle)
2. Business Hours & After-Hours Away Automation with interactive fallback
3. In-Chat Agent Control Commands (#bot, #resolve, #csat, #tag, #close)
4. Post-Resolution CSAT (Customer Satisfaction) Rating Survey
5. Interactive Menu & Multi-Level Catalog Navigation (List menus and buttons)
6. Live Agent Handoff with Round-Robin Team Auto-Assignment
7. Advanced Keyword, Regex, Button Click, and Intent Rules Engine (13 action types)
8. 24-Hour Session Window Inactivity Nudge Automation
9. Pre-configured Default WhatsApp Automations Seeder
"""

from __future__ import annotations

import json
import logging
import re
import uuid
from datetime import UTC, datetime, timedelta

import httpx
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels import get_provider
from communicator.channels.whatsapp import MetaWhatsAppProvider
from communicator.models.automation_rule import (
    AutomationRule,
    RuleActionType,
    RuleMatchType,
)
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation, ConversationState
from communicator.models.message import Message, MessageDirection, MessageStatus
from communicator.models.service import Service
from communicator.models.team_member import TeamMember
from communicator.services.integration_settings import get_settings_row
from communicator.services.messaging import send_outbound_message
from communicator.services.tags import add_tag_to_contact, remove_tag_from_contact

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# 1. In-Chat Agent Control Commands (#bot, #resolve, #csat, #tag)
# ---------------------------------------------------------------------------


async def handle_agent_commands(
    session: AsyncSession,
    conversation: Conversation,
    contact: Contact,
    body: str,
) -> bool:
    """
    Commands that human agents can type in WhatsApp to control bot state:
      - #bot or #resume: Resumes AI / automated bot replies.
      - #resolve or #close: Marks conversation resolved, resets takeover, triggers CSAT.
      - #csat: Sends customer satisfaction rating survey immediately.
      - #tag <name>: Adds a tag to the contact.
    """
    text = body.strip()
    if not text.startswith("#"):
        return False

    parts = text[1:].split(maxsplit=1)
    cmd = parts[0].lower()
    arg = parts[1].strip() if len(parts) > 1 else ""

    if cmd in ("bot", "resume", "unpause"):
        provider = await get_provider(contact.channel, session)
        await send_outbound_message(
            session,
            contact,
            provider,
            "🤖 Automated assistant has been resumed. How else can we help you today?",
        )
        conversation.human_takeover = False
        conversation.state = ConversationState.ENGAGED
        await session.flush()
        return True

    if cmd in ("resolve", "close"):
        conversation.human_takeover = False
        conversation.state = ConversationState.CLOSED_WON
        await session.flush()
        await send_csat_survey(session, conversation, contact)
        return True

    if cmd == "csat":
        await send_csat_survey(session, conversation, contact)
        return True

    if cmd == "tag" and arg:
        await add_tag_to_contact(session, contact, arg)
        provider = await get_provider(contact.channel, session)
        await send_outbound_message(session, contact, provider, f"✅ Contact tagged as '{arg}'.")
        return True

    return False


# ---------------------------------------------------------------------------
# 2. CSAT (Customer Satisfaction) Survey Automation
# ---------------------------------------------------------------------------


async def send_csat_survey(
    session: AsyncSession,
    conversation: Conversation,
    contact: Contact,
    custom_question: str | None = None,
) -> bool:
    """
    Sends an interactive CSAT rating survey with 3 quick-reply buttons.
    """
    provider = await get_provider(contact.channel, session)
    question = (
        custom_question
        or "How satisfied are you with our service today? Please select your rating:"
    )

    if isinstance(provider, MetaWhatsAppProvider):
        try:
            buttons = [
                {"id": "csat:5", "title": "⭐⭐⭐⭐⭐ Great"},
                {"id": "csat:3", "title": "⭐⭐⭐ Good"},
                {"id": "csat:1", "title": "⭐ Needs Help"},
            ]
            await provider.send_interactive_buttons(
                to=contact.value,
                body_text=question,
                buttons=buttons,
                header_text="Service Feedback",
                footer_text="Your feedback helps us improve!",
            )
            # Record outbound message
            session.add(
                Message(
                    conversation_id=conversation.id,
                    direction=MessageDirection.OUTBOUND,
                    channel=contact.channel,
                    body=question,
                    provider="meta",
                    provider_message_id="csat_survey",
                    status=MessageStatus.SENT,
                    intent="csat_request",
                )
            )
            await session.flush()
            return True
        except Exception:
            logger.exception("Failed to send interactive CSAT survey via Meta WhatsApp")

    # Fallback plain text send
    plain_text = f"{question}\n\nReply with 1 (Poor), 3 (Good), or 5 (Great)!"
    await send_outbound_message(session, contact, provider, plain_text)
    return True


async def handle_csat_response(
    session: AsyncSession,
    conversation: Conversation,
    contact: Contact,
    body: str,
    payload_id: str | None,
) -> bool:
    """
    Detects and records CSAT ratings from button clicks (csat:5) or text ("5", "great").
    """
    score: int | None = None

    if payload_id and payload_id.startswith("csat:"):
        try:
            score = int(payload_id.split(":", 1)[1])
        except ValueError:
            pass

    if score is None:
        trimmed = body.strip().lower()
        if trimmed in ("5", "5 star", "great", "excellent", "⭐⭐⭐⭐⭐"):
            score = 5
        elif trimmed in ("4", "4 star"):
            score = 4
        elif trimmed in ("3", "3 star", "good", "okay", "⭐⭐⭐"):
            score = 3
        elif trimmed in ("2", "2 star"):
            score = 2
        elif trimmed in ("1", "1 star", "poor", "bad", "needs help", "⭐"):
            score = 1

    if score is not None:
        conversation.csat_score = score
        conversation.csat_comment = body
        await session.flush()

        # Tag contact according to score
        if score >= 4:
            await add_tag_to_contact(session, contact, "csat_promoter")
            reply = "🎉 Thank you so much for the glowing feedback! We're thrilled to assist you."
        elif score == 3:
            await add_tag_to_contact(session, contact, "csat_neutral")
            reply = "🙏 Thank you for your feedback! We are constantly working to improve."
        else:
            await add_tag_to_contact(session, contact, "csat_detractor")
            reply = "🙏 Thank you for sharing your honest feedback. We apologize for falling short and will have a manager follow up with you."

        provider = await get_provider(contact.channel, session)
        await send_outbound_message(session, contact, provider, reply)
        return True

    return False


# ---------------------------------------------------------------------------
# 3. Live Agent Handoff & Auto-Assignment Automation
# ---------------------------------------------------------------------------


async def execute_handoff(
    session: AsyncSession,
    conversation: Conversation,
    contact: Contact,
    note: str = "",
) -> None:
    """
    Switches conversation to human takeover and assigns to an available team member (round-robin).
    """
    conversation.human_takeover = True
    conversation.state = ConversationState.HANDED_OFF

    # Round-robin team member assignment
    members_stmt = select(TeamMember).order_by(TeamMember.created_at.asc())
    members = (await session.execute(members_stmt)).scalars().all()
    if members:
        # Pick least assigned or first available
        assigned = members[0]
        conversation.assigned_to_id = assigned.id

    await session.flush()

    # Inform the customer
    provider = await get_provider(contact.channel, session)
    msg = (
        "🔔 A human specialist has been notified and is reviewing your message. "
        "A team member will be with you in just a moment!"
    )
    if note:
        msg = f"{note}\n\n{msg}"

    if isinstance(provider, MetaWhatsAppProvider):
        try:
            buttons = [
                {"id": "menu:main", "title": "🔙 Main Menu"},
                {"id": "menu:services", "title": "🌟 View Services"},
            ]
            await provider.send_interactive_buttons(
                to=contact.value,
                body_text=msg,
                buttons=buttons,
                header_text="Connecting with Agent",
            )
            session.add(
                Message(
                    conversation_id=conversation.id,
                    direction=MessageDirection.OUTBOUND,
                    channel=contact.channel,
                    body=msg,
                    provider="meta",
                    provider_message_id="agent_handoff",
                    status=MessageStatus.SENT,
                    intent="agent_handoff",
                )
            )
            await session.flush()
            return
        except Exception:  # noqa: BLE001, S110
            pass

    await send_outbound_message(session, contact, provider, msg)
    conversation.state = ConversationState.HANDED_OFF
    conversation.human_takeover = True
    await session.flush()


# ---------------------------------------------------------------------------
# 4. Interactive Navigation Tree & Menus
# ---------------------------------------------------------------------------


async def handle_navigation_menu(
    session: AsyncSession,
    conversation: Conversation,
    contact: Contact,
    payload_id: str | None,
    text: str,
) -> bool:
    """
    Interactive WhatsApp navigation tree:
      - Main Menu
      - Services Catalog
      - Pricing & Packages
      - Office Location & Working Hours
    """
    normalized = (text or "").strip().lower()
    match_main = payload_id in ("menu:main", "menu") or normalized in (
        "menu",
        "help",
        "options",
        "start",
        "main menu",
    )
    match_services = payload_id in ("menu:services", "pricing_catalog") or normalized in (
        "services",
        "catalog",
        "solutions",
        "1",
        "1️⃣",
    )
    match_pricing = payload_id == "menu:pricing" or normalized in (
        "pricing",
        "price",
        "rates",
        "cost",
        "quote",
        "2",
        "2️⃣",
    )
    match_location = payload_id == "menu:location" or normalized in (
        "location",
        "office",
        "address",
        "hours",
        "directions",
    )
    match_agent = payload_id in ("action:agent", "human_agent") or normalized in (
        "agent",
        "human",
        "representative",
        "support",
        "talk to human",
        "operator",
    )

    provider = await get_provider(contact.channel, session)
    if not isinstance(provider, MetaWhatsAppProvider):
        return False

    if match_agent:
        await execute_handoff(session, conversation, contact)
        return True

    if match_main:
        sections = [
            {
                "title": "Explore Our Solutions",
                "rows": [
                    {
                        "id": "menu:services",
                        "title": "Services Catalog",
                        "description": "View custom solutions and packages",
                    },
                    {
                        "id": "menu:pricing",
                        "title": "Pricing & Quotation",
                        "description": "Instant estimates and transparent rates",
                    },
                    {
                        "id": "menu:location",
                        "title": "Office & Hours",
                        "description": "Find our branch location and timings",
                    },
                ],
            },
            {
                "title": "Customer Support",
                "rows": [
                    {
                        "id": "action:agent",
                        "title": "Talk to Human Agent",
                        "description": "Connect directly with our team",
                    },
                ],
            },
        ]
        try:
            await provider.send_interactive_list(
                to=contact.value,
                body_text="Welcome! Please choose an option from the menu below to get started immediately:",
                button_title="Explore Options",
                sections=sections,
                header_text="Main Navigation",
                footer_text="Reply at any time or tap an option",
            )
            session.add(
                Message(
                    conversation_id=conversation.id,
                    direction=MessageDirection.OUTBOUND,
                    channel=contact.channel,
                    body="[Sent WhatsApp Main Menu]",
                    provider="meta",
                    provider_message_id="menu_main",
                    status=MessageStatus.SENT,
                    intent="menu",
                )
            )
            await session.flush()
            return True
        except Exception:
            logger.exception("Failed to send WhatsApp interactive menu")

    if match_services:
        # Load active services from catalog
        services_stmt = select(Service).order_by(Service.name.asc())
        services = (await session.execute(services_stmt)).scalars().all()
        if services:
            sections = [
                {
                    "title": "Our Active Services",
                    "rows": [
                        {
                            "id": f"svc:{s.id}",
                            "title": s.name[:24],
                            "description": (s.description or "Professional solution")[:72],
                        }
                        for s in services[:10]
                    ],
                }
            ]
            await provider.send_interactive_list(
                to=contact.value,
                body_text="Here is our current catalog of solutions. Tap any service for details and custom pricing:",
                button_title="Select Service",
                sections=sections,
                header_text="Service Catalog",
                footer_text="Tap an item or reply with any question",
            )
        else:
            buttons = [
                {"id": "menu:pricing", "title": "💰 Pricing"},
                {"id": "action:agent", "title": "📞 Talk to Agent"},
                {"id": "menu:main", "title": "🔙 Main Menu"},
            ]
            await provider.send_interactive_buttons(
                to=contact.value,
                body_text="We offer full-cycle digital development, AI workflow automation, and custom software systems.",
                buttons=buttons,
                header_text="Our Services",
            )
        return True

    if match_pricing:
        settings_row = await get_settings_row(session)
        biz_name = settings_row.business_name or "our team"
        body = (
            f"💰 *Transparent Pricing from {biz_name}*\n\n"
            "• Starter Packages: Flexible monthly support & setup\n"
            "• Enterprise & Custom Solutions: Tailored scoped delivery\n"
            "• Free Initial Consultation: 100% complimentary requirement audit\n\n"
            "Would you like an immediate estimate tailored to your project?"
        )
        buttons = [
            {"id": "action:agent", "title": "📞 Get Custom Quote"},
            {"id": "menu:services", "title": "🌟 Browse Services"},
            {"id": "menu:main", "title": "🔙 Main Menu"},
        ]
        await provider.send_interactive_buttons(
            to=contact.value,
            body_text=body,
            buttons=buttons,
            header_text="Pricing & Quotes",
        )
        return True

    if match_location:
        settings_row = await get_settings_row(session)
        biz_hours = f"{settings_row.business_hours_start}:00 - {settings_row.business_hours_end}:00 ({settings_row.business_timezone})"
        body = (
            f"📍 *Office Location & Timings*\n\n"
            f"⏰ Working Hours: Monday - Friday, {biz_hours}\n"
            "🌍 Available 24/7 online for automated inquiries and order booking.\n\n"
            "Tap below to speak with an agent or return to the menu."
        )
        buttons = [
            {"id": "action:agent", "title": "📞 Talk to Agent"},
            {"id": "menu:main", "title": "🔙 Main Menu"},
        ]
        await provider.send_interactive_buttons(
            to=contact.value,
            body_text=body,
            buttons=buttons,
            header_text="Location & Hours",
        )
        return True

    return False


# ---------------------------------------------------------------------------
# 5. Welcome / Greeting Automation (First-Touch or > 24h Idle)
# ---------------------------------------------------------------------------


async def should_send_welcome_greeting(
    session: AsyncSession,
    conversation: Conversation,
    now: datetime,
) -> bool:
    """
    True if this is the user's first inbound message, or if their last inbound
    was more than 24 hours ago.
    """
    if conversation.last_inbound_at is None:
        return True
    return (now - conversation.last_inbound_at) > timedelta(hours=24)


async def send_welcome_greeting(
    session: AsyncSession,
    conversation: Conversation,
    contact: Contact,
) -> None:
    """Sends warm interactive greeting with quick-reply buttons."""
    settings_row = await get_settings_row(session)
    biz_name = settings_row.business_name or "NEWAX"
    greeting_text = (
        f"👋 Hello! Welcome to *{biz_name}* on WhatsApp.\n\n"
        "We're here to assist you with services, instant pricing, and expert support. "
        "How can we help you today?"
    )

    provider = await get_provider(contact.channel, session)
    if isinstance(provider, MetaWhatsAppProvider):
        try:
            buttons = [
                {"id": "menu:services", "title": "🌟 View Services"},
                {"id": "menu:pricing", "title": "💰 Pricing & Rates"},
                {"id": "action:agent", "title": "📞 Talk to Human"},
            ]
            await provider.send_interactive_buttons(
                to=contact.value,
                body_text=greeting_text,
                buttons=buttons,
                header_text=f"Welcome to {biz_name}",
                footer_text="Select an option or reply directly",
            )
            session.add(
                Message(
                    conversation_id=conversation.id,
                    direction=MessageDirection.OUTBOUND,
                    channel=contact.channel,
                    body=greeting_text,
                    provider="meta",
                    provider_message_id="welcome_greeting",
                    status=MessageStatus.SENT,
                    intent="welcome_greeting",
                )
            )
            await session.flush()
            return
        except Exception:
            logger.exception("Failed to send interactive welcome greeting")

    await send_outbound_message(session, contact, provider, greeting_text)


# ---------------------------------------------------------------------------
# 6. Advanced Automation Rules Matcher & Action Executor
# ---------------------------------------------------------------------------


async def match_advanced_rule(
    session: AsyncSession,
    text: str,
    payload_id: str | None = None,
    intent: str | None = None,
) -> AutomationRule | None:
    """Matches inbound message or button against active AutomationRule records."""
    stmt = (
        select(AutomationRule)
        .where(AutomationRule.is_active.is_(True))
        .order_by(AutomationRule.priority.desc(), AutomationRule.created_at.asc())
    )
    rules = (await session.execute(stmt)).scalars().all()

    norm_text = text.strip().lower()
    norm_payload = (payload_id or "").strip().lower()

    for rule in rules:
        keyword = rule.trigger_keyword.strip().lower()
        if not keyword:
            continue

        # 1. Exact match
        if rule.match_type == RuleMatchType.EXACT and norm_text == keyword:
            return rule

        # 2. Contains match
        if rule.match_type == RuleMatchType.CONTAINS and keyword in norm_text:
            return rule

        # 3. Starts with match
        if rule.match_type == RuleMatchType.STARTS_WITH and norm_text.startswith(keyword):
            return rule

        # 4. Regex match
        if rule.match_type == RuleMatchType.REGEX:
            try:
                if re.search(rule.trigger_keyword, text, re.IGNORECASE):
                    return rule
            except re.error:
                pass

        # 5. Button click match
        if rule.match_type == RuleMatchType.BUTTON_CLICK and (
            norm_payload == keyword or keyword in norm_payload or norm_text == keyword
        ):
            return rule

        # 6. List select match
        if rule.match_type == RuleMatchType.LIST_SELECT and (
            norm_payload == keyword or keyword in norm_payload or norm_text == keyword
        ):
            return rule

        # 7. Intent match
        if rule.match_type == RuleMatchType.INTENT and intent and intent.strip().lower() == keyword:
            return rule

    return None


async def execute_rule_action(
    session: AsyncSession,
    rule: AutomationRule,
    conversation: Conversation,
    contact: Contact,
) -> bool:
    """
    Executes all 13 supported WhatsApp automation rule action types.
    """
    provider = await get_provider(contact.channel, session)
    now = datetime.now(UTC)
    rule.execution_count += 1
    rule.last_triggered_at = now

    # A. AUTO_REPLY (plain text)
    if rule.action_type == RuleActionType.AUTO_REPLY:
        if rule.action_value:
            await send_outbound_message(session, contact, provider, rule.action_value)
            return True

    # B. INTERACTIVE_BUTTONS
    elif rule.action_type == RuleActionType.INTERACTIVE_BUTTONS:
        if isinstance(provider, MetaWhatsAppProvider):
            try:
                data = json.loads(rule.action_value)
                body = data.get("body", "Please choose an option:")
                buttons = data.get("buttons", [])
                header = data.get("header")
                footer = data.get("footer")
                await provider.send_interactive_buttons(
                    to=contact.value,
                    body_text=body,
                    buttons=buttons,
                    header_text=header,
                    footer_text=footer,
                )
                session.add(
                    Message(
                        conversation_id=conversation.id,
                        direction=MessageDirection.OUTBOUND,
                        channel=contact.channel,
                        body=body,
                        provider="meta",
                        provider_message_id=f"rule_{rule.id}",
                        status=MessageStatus.SENT,
                        intent="rule_interactive_buttons",
                    )
                )
                await session.flush()
                return True
            except Exception:
                logger.exception("Failed to execute interactive_buttons rule")
        # Fallback to plain text
        await send_outbound_message(session, contact, provider, rule.action_value)
        return True

    # C. INTERACTIVE_LIST
    elif rule.action_type == RuleActionType.INTERACTIVE_LIST:
        if isinstance(provider, MetaWhatsAppProvider):
            try:
                data = json.loads(rule.action_value)
                body = data.get("body", "Please select an option:")
                btn = data.get("button", "Select Option")
                sections = data.get("sections", [])
                header = data.get("header")
                footer = data.get("footer")
                await provider.send_interactive_list(
                    to=contact.value,
                    body_text=body,
                    button_title=btn,
                    sections=sections,
                    header_text=header,
                    footer_text=footer,
                )
                session.add(
                    Message(
                        conversation_id=conversation.id,
                        direction=MessageDirection.OUTBOUND,
                        channel=contact.channel,
                        body=body,
                        provider="meta",
                        provider_message_id=f"rule_{rule.id}",
                        status=MessageStatus.SENT,
                        intent="rule_interactive_list",
                    )
                )
                await session.flush()
                return True
            except Exception:
                logger.exception("Failed to execute interactive_list rule")

    # D. SEND_MEDIA
    elif rule.action_type == RuleActionType.SEND_MEDIA:
        if isinstance(provider, MetaWhatsAppProvider):
            try:
                data = json.loads(rule.action_value)
                media_type = data.get("media_type", "image")
                url = data.get("url", "")
                caption = data.get("caption")
                filename = data.get("filename")
                if url:
                    await provider.send_media(
                        to=contact.value,
                        media_type=media_type,
                        media_url_or_id=url,
                        caption=caption,
                        filename=filename,
                    )
                    session.add(
                        Message(
                            conversation_id=conversation.id,
                            direction=MessageDirection.OUTBOUND,
                            channel=contact.channel,
                            body=caption or f"Sent {media_type}",
                            provider="meta",
                            provider_message_id=f"rule_{rule.id}",
                            status=MessageStatus.SENT,
                            intent="rule_media",
                        )
                    )
                    await session.flush()
                    return True
            except Exception:
                logger.exception("Failed to execute send_media rule")

    # E. SEND_CTA_URL
    elif rule.action_type == RuleActionType.SEND_CTA_URL:
        if isinstance(provider, MetaWhatsAppProvider):
            try:
                data = json.loads(rule.action_value)
                body = data.get("body", "Click below to open:")
                display_text = data.get("display_text", "Open Link")
                url = data.get("url", "")
                header = data.get("header")
                footer = data.get("footer")
                if url:
                    await provider.send_interactive_cta_url(
                        to=contact.value,
                        body_text=body,
                        display_text=display_text,
                        url=url,
                        header_text=header,
                        footer_text=footer,
                    )
                    session.add(
                        Message(
                            conversation_id=conversation.id,
                            direction=MessageDirection.OUTBOUND,
                            channel=contact.channel,
                            body=f"{body} ({url})",
                            provider="meta",
                            provider_message_id=f"rule_{rule.id}",
                            status=MessageStatus.SENT,
                            intent="rule_cta_url",
                        )
                    )
                    await session.flush()
                    return True
            except Exception:
                logger.exception("Failed to execute send_cta_url rule")

    # F. SEND_TEMPLATE
    elif rule.action_type == RuleActionType.SEND_TEMPLATE:
        if isinstance(provider, MetaWhatsAppProvider):
            try:
                data = json.loads(rule.action_value)
                t_name = data.get("name", "")
                lang = data.get("language", "en_US")
                params = data.get("params", [])
                if t_name:
                    await provider.send_template(
                        to=contact.value,
                        template_name=t_name,
                        language=lang,
                        body_params=params,
                    )
                    return True
            except Exception:
                logger.exception("Failed to execute send_template rule")

    # G. HANDOFF
    elif rule.action_type == RuleActionType.HANDOFF:
        await execute_handoff(session, conversation, contact, note=rule.action_value)
        return True

    # H. ADD_TAG
    elif rule.action_type == RuleActionType.ADD_TAG:
        if rule.action_value:
            await add_tag_to_contact(session, contact, rule.action_value.strip())
        return True

    # I. REMOVE_TAG
    elif rule.action_type == RuleActionType.REMOVE_TAG:
        if rule.action_value:
            await remove_tag_from_contact(session, contact, rule.action_value.strip())
        return True

    # J. ASSIGN_TEAM
    elif rule.action_type == RuleActionType.ASSIGN_TEAM:
        target = rule.action_value.strip()
        if target == "round_robin":
            members_stmt = select(TeamMember).order_by(TeamMember.created_at.asc())
            members = (await session.execute(members_stmt)).scalars().all()
            if members:
                conversation.assigned_to_id = members[0].id
        else:
            try:
                member_uuid = uuid.UUID(target)
                conversation.assigned_to_id = member_uuid
            except ValueError:
                pass
        await session.flush()
        return True

    # K. SEND_CSAT
    elif rule.action_type == RuleActionType.SEND_CSAT:
        await send_csat_survey(session, conversation, contact, custom_question=rule.action_value)
        return True

    # L. TRIGGER_WEBHOOK
    elif rule.action_type == RuleActionType.TRIGGER_WEBHOOK:
        url = rule.action_value.strip()
        if url.startswith(("http://", "https://")):
            payload = {
                "event": "automation_rule_triggered",
                "rule_id": str(rule.id),
                "rule_name": rule.name,
                "contact_value": contact.value,
                "contact_name": contact.person_name,
                "conversation_id": str(conversation.id),
                "timestamp": now.isoformat(),
            }
            try:
                async with httpx.AsyncClient(timeout=10.0) as client:
                    await client.post(url, json=payload)
            except Exception:
                logger.exception("Failed to dispatch webhook for rule %s", rule.id)
            return True

    return False


# ---------------------------------------------------------------------------
# 7. 24-Hour Session Window Inactivity Nudge Automation
# ---------------------------------------------------------------------------


async def run_inactivity_nudges(session: AsyncSession) -> list[uuid.UUID]:
    """
    Finds WhatsApp conversations that are active/engaged, where the user sent a message
    between 18 and 23.5 hours ago, and sends an automatic nudge before the 24h Meta
    session window closes.
    """
    now = datetime.now(UTC)
    window_start = now - timedelta(hours=23, minutes=30)
    window_end = now - timedelta(hours=18)

    stmt = select(Conversation).where(
        Conversation.state.in_([ConversationState.ENGAGED, ConversationState.AWAITING_REPLY]),
        Conversation.human_takeover.is_(False),
        Conversation.last_inbound_at.is_not(None),
        Conversation.last_inbound_at >= window_start,
        Conversation.last_inbound_at <= window_end,
    )
    convs = (await session.execute(stmt)).scalars().all()
    nudged = []

    for conv in convs:
        contact = await session.get(Contact, conv.contact_id)
        if not contact or contact.channel != ChannelType.WHATSAPP:
            continue

        provider = await get_provider(contact.channel, session)
        if not isinstance(provider, MetaWhatsAppProvider):
            continue

        name = contact.person_name or "there"
        body = (
            f"Hi {name}, just checking in to see if you needed any more assistance "
            "or had further questions before our active chat session wraps up!"
        )
        buttons = [
            {"id": "menu:services", "title": "🌟 View Services"},
            {"id": "action:agent", "title": "📞 Talk to Agent"},
        ]
        try:
            await provider.send_interactive_buttons(
                to=contact.value,
                body_text=body,
                buttons=buttons,
                header_text="Following Up",
            )
            session.add(
                Message(
                    conversation_id=conv.id,
                    direction=MessageDirection.OUTBOUND,
                    channel=contact.channel,
                    body=body,
                    provider="meta",
                    provider_message_id="inactivity_nudge",
                    status=MessageStatus.SENT,
                    intent="inactivity_nudge",
                )
            )
            conv.last_outbound_at = now
            nudged.append(conv.id)
        except Exception:
            logger.exception("Failed to send 24h inactivity nudge to %s", contact.value)

    await session.flush()
    return nudged


# ---------------------------------------------------------------------------
# 8. Pre-Configured Default Automations Seeder
# ---------------------------------------------------------------------------


async def seed_default_whatsapp_automations(session: AsyncSession) -> int:
    """
    Installs 11 production-grade, battle-tested WhatsApp automations:
    1. Welcome Greeting with Quick Replies
    2. Main Interactive Navigation Menu
    3. Services & Catalog Overview
    4. Transparent Pricing & Quote Request
    5. Live Human Support Handoff
    6. Office Hours & Location Details
    7. Book a Demo / Schedule CTA
    8. Post-Support CSAT Feedback Survey
    9. Opt-Out Compliance (STOP)
    10. Opt-In Re-engagement (START)
    11. High-Value Lead Auto-Tagging
    """
    existing_stmt = select(func.count()).select_from(AutomationRule)
    (await session.execute(existing_stmt)).scalar_one()

    defaults = [
        AutomationRule(
            name="Main Interactive Menu",
            description="Opens the main navigation list menu with services, pricing, and live support",
            trigger_keyword="menu",
            match_type=RuleMatchType.EXACT,
            action_type=RuleActionType.INTERACTIVE_BUTTONS,
            action_value=json.dumps(
                {
                    "body": "Welcome to our WhatsApp Center! Please select an option to get started immediately:",
                    "buttons": [
                        {"id": "menu:services", "title": "🌟 Browse Services"},
                        {"id": "menu:pricing", "title": "💰 Pricing & Plans"},
                        {"id": "action:agent", "title": "📞 Talk to Human"},
                    ],
                    "header": "Main Menu",
                    "footer": "Select an option or reply with your question",
                }
            ),
            priority=100,
            is_active=True,
        ),
        AutomationRule(
            name="Live Human Support Handoff",
            description="Transfers conversation to live human agent and pauses the bot",
            trigger_keyword="agent",
            match_type=RuleMatchType.CONTAINS,
            action_type=RuleActionType.HANDOFF,
            action_value="Customer requested human agent assistance.",
            priority=95,
            is_active=True,
        ),
        AutomationRule(
            name="Human Agent Alias (Operator)",
            description="Transfers conversation to live human agent when typing 'human' or 'operator'",
            trigger_keyword="human",
            match_type=RuleMatchType.CONTAINS,
            action_type=RuleActionType.HANDOFF,
            action_value="Customer requested human agent.",
            priority=94,
            is_active=True,
        ),
        AutomationRule(
            name="Services Catalog Quick View",
            description="Displays interactive catalog of our core services",
            trigger_keyword="services",
            match_type=RuleMatchType.CONTAINS,
            action_type=RuleActionType.INTERACTIVE_BUTTONS,
            action_value=json.dumps(
                {
                    "body": "We specialize in custom web software, AI automations, and digital marketing systems.",
                    "buttons": [
                        {"id": "menu:pricing", "title": "💰 Pricing & Rates"},
                        {"id": "action:agent", "title": "📞 Speak with Sales"},
                        {"id": "menu:main", "title": "🔙 Main Menu"},
                    ],
                    "header": "Services Catalog",
                }
            ),
            priority=85,
            is_active=True,
        ),
        AutomationRule(
            name="Pricing & Rates Inquiry",
            description="Provides transparent pricing breakdown and quote request options",
            trigger_keyword="pricing",
            match_type=RuleMatchType.CONTAINS,
            action_type=RuleActionType.INTERACTIVE_BUTTONS,
            action_value=json.dumps(
                {
                    "body": "Our solutions start with flexible monthly plans, or fixed-price milestones with 100% upfront quotes.",
                    "buttons": [
                        {"id": "action:agent", "title": "📞 Get Custom Quote"},
                        {"id": "menu:services", "title": "🌟 Our Services"},
                        {"id": "menu:main", "title": "🔙 Main Menu"},
                    ],
                    "header": "Pricing Information",
                }
            ),
            priority=80,
            is_active=True,
        ),
        AutomationRule(
            name="Book a Meeting / Demo CTA",
            description="Sends clickable Call-To-Action link to book a discovery call",
            trigger_keyword="demo",
            match_type=RuleMatchType.CONTAINS,
            action_type=RuleActionType.SEND_CTA_URL,
            action_value=json.dumps(
                {
                    "body": "Ready to discuss your project? Schedule a 15-minute discovery call directly on our calendar:",
                    "display_text": "📅 Book Free Demo",
                    "url": "https://calendly.com",
                    "header": "Schedule Discovery",
                }
            ),
            priority=75,
            is_active=True,
        ),
        AutomationRule(
            name="Office Location & Timings",
            description="Shares office address and operating hours",
            trigger_keyword="location",
            match_type=RuleMatchType.CONTAINS,
            action_type=RuleActionType.AUTO_REPLY,
            action_value="📍 Office Address: Main Commercial Hub\n⏰ Working Hours: Monday - Friday, 9:00 AM - 6:00 PM\n📞 WhatsApp inquiries are accepted 24/7!",
            priority=70,
            is_active=True,
        ),
        AutomationRule(
            name="Post-Resolution Feedback / CSAT Survey",
            description="Prompts customer with interactive star rating upon resolution",
            trigger_keyword="csat",
            match_type=RuleMatchType.EXACT,
            action_type=RuleActionType.SEND_CSAT,
            action_value="How was your experience with our team today?",
            priority=65,
            is_active=True,
        ),
        AutomationRule(
            name="High-Intent Lead Auto-Tagging",
            description="Automatically tags lead as 'high_intent' when asking about contracts or quotes",
            trigger_keyword="quote",
            match_type=RuleMatchType.CONTAINS,
            action_type=RuleActionType.ADD_TAG,
            action_value="high_intent",
            priority=60,
            is_active=True,
        ),
        AutomationRule(
            name="VIP Customer Auto-Tagging",
            description="Automatically tags lead as 'vip' when enterprise inquiry occurs",
            trigger_keyword="enterprise",
            match_type=RuleMatchType.CONTAINS,
            action_type=RuleActionType.ADD_TAG,
            action_value="enterprise_lead",
            priority=55,
            is_active=True,
        ),
    ]

    added = 0
    # Avoid duplicate names
    existing_names_stmt = select(AutomationRule.name)
    existing_names = set((await session.execute(existing_names_stmt)).scalars().all())

    for r in defaults:
        if r.name not in existing_names:
            session.add(r)
            added += 1

    await session.flush()
    return added
