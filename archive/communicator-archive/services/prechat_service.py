from __future__ import annotations

import logging
import re
import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.compliance.optout import record_consent
from communicator.models.consent import ConsentStatus
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.inbox import Inbox
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection, MessageStatus

logger = logging.getLogger(__name__)

EMAIL_REGEX = re.compile(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$")

DEFAULT_PRECHAT_FIELDS: list[dict[str, Any]] = [
    {
        "id": "f_name",
        "name": "fullName",
        "label": "Full Name",
        "type": "text",
        "placeholder": "e.g. Jane Doe",
        "required": True,
        "options": [],
        "order": 1,
    },
    {
        "id": "f_email",
        "name": "email",
        "label": "Email Address",
        "type": "email",
        "placeholder": "e.g. jane@company.com",
        "required": True,
        "options": [],
        "order": 2,
    },
    {
        "id": "f_inquiry",
        "name": "inquiryType",
        "label": "What can we help you with?",
        "type": "select",
        "placeholder": "Select inquiry reason",
        "required": True,
        "options": [
            "Sales & Pricing",
            "Technical Support",
            "Billing & Invoices",
            "Partnerships & Other",
        ],
        "order": 3,
    },
    {
        "id": "f_company",
        "name": "company",
        "label": "Company / Organization",
        "type": "text",
        "placeholder": "e.g. Acme Innovations",
        "required": False,
        "options": [],
        "order": 4,
    },
    {
        "id": "f_message",
        "name": "message",
        "label": "Initial Question or Requirement",
        "type": "textarea",
        "placeholder": "Briefly describe your request...",
        "required": False,
        "options": [],
        "order": 5,
    },
]

DEFAULT_ROUTING_RULES: list[dict[str, Any]] = [
    {
        "id": "rule_billing",
        "field": "inquiryType",
        "operator": "equals",
        "value": "Billing & Invoices",
        "assign_priority": "high",
        "assign_tag": "billing",
        "team_target": "Finance & Billing",
    },
    {
        "id": "rule_sales",
        "field": "inquiryType",
        "operator": "equals",
        "value": "Sales & Pricing",
        "assign_priority": "high",
        "assign_tag": "sales_lead",
        "team_target": "Sales Operations",
    },
    {
        "id": "rule_tech",
        "field": "inquiryType",
        "operator": "equals",
        "value": "Technical Support",
        "assign_priority": "medium",
        "assign_tag": "tier1_support",
        "team_target": "Engineering Support",
    },
]


def _build_default_prechat_config() -> dict[str, Any]:
    return {
        "enabled": True,
        "title": "Welcome to Live Chat",
        "description": "Please share a few details so Captain AI or our team can assist you right away.",
        "submit_button_text": "Start Conversation",
        "require_email": True,
        "fields": [dict(f) for f in DEFAULT_PRECHAT_FIELDS],
        "routing_rules": [dict(r) for r in DEFAULT_ROUTING_RULES],
    }


async def _get_website_inbox(session: AsyncSession) -> Inbox:
    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.WEBSITE)
    inbox = (await session.execute(stmt)).scalars().first()
    if not inbox:
        inbox = Inbox(
            name="Website Live Chat",
            channel_type=ChannelType.WEBSITE,
            is_active=True,
            greeting_enabled=True,
            greeting_message="👋 Welcome to our live chat! How can our support team help you today?",
            credentials={
                "websiteToken": f"web_live_{uuid.uuid4().hex[:16]}",
                "websiteDomain": "bayworks.com",
                "preChatConfig": _build_default_prechat_config(),
            },
        )
        session.add(inbox)
        await session.flush()
    return inbox


async def get_prechat_config(session: AsyncSession) -> dict[str, Any]:
    """Retrieves current pre-chat intake configuration and active routing rules."""
    inbox = await _get_website_inbox(session)
    creds = dict(inbox.credentials or {})
    cfg = creds.get("preChatConfig")

    if not isinstance(cfg, dict) or "fields" not in cfg:
        cfg = _build_default_prechat_config()
        # Migrate old flat fields if present
        if (
            "preChatFields" in creds
            and isinstance(creds["preChatFields"], list)
            and creds["preChatFields"]
        ):
            cfg["fields"] = [
                {
                    "id": f"field_{idx}",
                    "name": f.get("name", f"field_{idx}"),
                    "label": f.get("label", "Field"),
                    "type": f.get("type", "text"),
                    "placeholder": f.get("placeholder", ""),
                    "required": bool(f.get("required", False)),
                    "options": f.get("options", []),
                    "order": idx + 1,
                }
                for idx, f in enumerate(creds["preChatFields"])
            ]
        if "preChatFormEnabled" in creds:
            cfg["enabled"] = bool(creds["preChatFormEnabled"])
        creds["preChatConfig"] = cfg
        inbox.credentials = creds
        await session.flush()

    return {
        "website_token": creds.get("websiteToken", ""),
        "website_domain": creds.get("websiteDomain", "bayworks.com"),
        **cfg,
    }


async def update_prechat_config(session: AsyncSession, updates: dict[str, Any]) -> dict[str, Any]:
    """Updates pre-chat form structure, validation parameters, and conditional routing rules."""
    inbox = await _get_website_inbox(session)
    creds = dict(inbox.credentials or {})
    current_cfg = dict(creds.get("preChatConfig") or _build_default_prechat_config())

    for key in ("enabled", "title", "description", "submit_button_text", "require_email"):
        if key in updates and updates[key] is not None:
            current_cfg[key] = updates[key]

    if "fields" in updates and isinstance(updates["fields"], list):
        clean_fields = []
        for i, field in enumerate(updates["fields"]):
            fid = str(field.get("id") or f"field_{uuid.uuid4().hex[:8]}")
            raw_name = str(field.get("name") or f"custom_{i}").strip()
            slug_name = re.sub(r"[^a-zA-Z0-9_]", "", raw_name) or f"field_{i}"
            clean_fields.append(
                {
                    "id": fid,
                    "name": slug_name,
                    "label": str(field.get("label") or slug_name).strip(),
                    "type": str(field.get("type") or "text").lower(),
                    "placeholder": str(field.get("placeholder") or ""),
                    "required": bool(field.get("required", False)),
                    "options": [
                        str(opt).strip() for opt in field.get("options", []) if str(opt).strip()
                    ],
                    "order": int(field.get("order", i + 1)),
                }
            )
        current_cfg["fields"] = sorted(clean_fields, key=lambda f: f["order"])

    if "routing_rules" in updates and isinstance(updates["routing_rules"], list):
        clean_rules = []
        for i, rule in enumerate(updates["routing_rules"]):
            clean_rules.append(
                {
                    "id": str(rule.get("id") or f"rule_{uuid.uuid4().hex[:8]}"),
                    "field": str(rule.get("field") or "").strip(),
                    "operator": str(rule.get("operator") or "equals").lower(),
                    "value": str(rule.get("value") or "").strip(),
                    "assign_priority": str(rule.get("assign_priority") or "medium").lower(),
                    "assign_tag": str(rule.get("assign_tag") or "").strip().lower(),
                    "team_target": str(rule.get("team_target") or "").strip(),
                }
            )
        current_cfg["routing_rules"] = clean_rules

    creds["preChatConfig"] = current_cfg
    creds["preChatFormEnabled"] = current_cfg["enabled"]
    creds["preChatFields"] = [
        {"name": f["name"], "label": f["label"], "required": f["required"]}
        for f in current_cfg["fields"]
    ]
    inbox.credentials = creds
    await session.flush()

    return {
        "website_token": creds.get("websiteToken", ""),
        "website_domain": creds.get("websiteDomain", "bayworks.com"),
        **current_cfg,
    }


async def reset_prechat_defaults(session: AsyncSession) -> dict[str, Any]:
    """Resets pre-chat intake fields and routing rules back to factory defaults."""
    default_cfg = _build_default_prechat_config()
    return await update_prechat_config(session, default_cfg)


def validate_submission_answers(
    fields_config: list[dict[str, Any]], answers: dict[str, Any], require_email: bool = True
) -> list[str]:
    """Validates submitted intake answers against field definitions."""
    errors: list[str] = []

    for f in fields_config:
        name = f["name"]
        val = answers.get(name)
        val_str = str(val).strip() if val is not None else ""

        if f.get("required") and not val_str:
            errors.append(f"Field '{f.get('label') or name}' is required.")
            continue

        if val_str and f.get("type") == "email" and not EMAIL_REGEX.match(val_str):
            errors.append(f"Field '{f.get('label') or name}' must be a valid email address.")

        if (
            val_str
            and f.get("type") == "select"
            and f.get("options")
            and val_str not in f["options"]
        ):
            errors.append(
                f"Selected value '{val_str}' for '{f.get('label') or name}' is not among allowed choices: {', '.join(f['options'])}."
            )

    if require_email:
        email_val = str(answers.get("email") or answers.get("emailAddress") or "").strip()
        if (not email_val or not EMAIL_REGEX.match(email_val)) and not any(
            "email" in err.lower() for err in errors
        ):
            errors.append("A valid email address is required to initiate live chat.")

    return errors


def evaluate_routing_rules(
    routing_rules: list[dict[str, Any]], answers: dict[str, Any]
) -> dict[str, Any]:
    """
    Evaluates conditional routing rules against submitted answers.
    Returns calculated priority, tag list, and destination team.
    """
    priority = "medium"
    tags: list[str] = []
    team_target: str | None = None
    applied_rules: list[str] = []

    priority_weight = {"low": 1, "medium": 2, "high": 3, "urgent": 4}

    for rule in routing_rules:
        f_name = rule.get("field", "")
        op = rule.get("operator", "equals")
        target_val = str(rule.get("value", "")).strip().lower()

        submitted_val = str(answers.get(f_name, "")).strip().lower()
        if not submitted_val and f_name == "inquiryType":
            # Check fallback aliases
            submitted_val = (
                str(answers.get("inquiry_type") or answers.get("inquiry") or "").strip().lower()
            )

        matched = False
        if (
            op == "equals"
            and submitted_val == target_val
            or op == "contains"
            and target_val in submitted_val
            or op == "not_equals"
            and submitted_val
            and submitted_val != target_val
        ):
            matched = True

        if matched:
            applied_rules.append(rule.get("id", "rule"))
            rule_priority = rule.get("assign_priority", "").lower()
            if rule_priority in priority_weight and priority_weight.get(
                rule_priority, 0
            ) > priority_weight.get(priority, 0):
                priority = rule_priority

            tag = rule.get("assign_tag")
            if tag and tag not in tags:
                tags.append(tag)

            if rule.get("team_target") and not team_target:
                team_target = rule["team_target"]

    return {
        "priority": priority,
        "tags": tags,
        "team_target": team_target,
        "applied_rules": applied_rules,
    }


async def submit_prechat_intake(
    session: AsyncSession,
    website_token: str,
    answers: dict[str, Any],
    client_ip: str | None = None,
) -> dict[str, Any]:
    """
    Validates pre-chat form submission, provisions or enriches contact profile,
    evaluates conditional routing rules, creates active conversation, and injects initial inquiry.
    """
    # 1. Retrieve inbox matching website token
    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.WEBSITE)
    inboxes = (await session.execute(stmt)).scalars().all()

    target_inbox: Inbox | None = None
    for i in inboxes:
        creds = i.credentials or {}
        if creds.get("websiteToken") == website_token:
            target_inbox = i
            break
    if not target_inbox and inboxes:
        target_inbox = inboxes[0]

    if not target_inbox:
        raise ValueError("No website live chat channel found.")

    creds = dict(target_inbox.credentials or {})
    prechat_cfg = creds.get("preChatConfig") or _build_default_prechat_config()

    # 2. Validation
    fields = prechat_cfg.get("fields", DEFAULT_PRECHAT_FIELDS)
    require_email = bool(prechat_cfg.get("require_email", True))
    errors = validate_submission_answers(fields, answers, require_email=require_email)
    if errors:
        return {
            "success": False,
            "errors": errors,
        }

    # 3. Extract core contact attributes
    email = (
        str(
            answers.get("email")
            or answers.get("emailAddress")
            or f"guest_{uuid.uuid4().hex[:8]}@livechat.local"
        )
        .strip()
        .lower()
    )
    full_name = str(
        answers.get("fullName")
        or answers.get("name")
        or answers.get("full_name")
        or "Website Visitor"
    ).strip()
    phone = str(answers.get("phone") or answers.get("phoneNumber") or "").strip()
    company = str(answers.get("company") or answers.get("companyName") or "").strip()
    initial_message = str(
        answers.get("message") or answers.get("initialMessage") or answers.get("question") or ""
    ).strip()

    # 4. Find or Create Contact
    contact_stmt = select(Contact).where(
        Contact.channel == ChannelType.WEBSITE, Contact.value == email
    )
    contact = (await session.execute(contact_stmt)).scalar_one_or_none()

    if not contact:
        lead = Lead(
            harvester_institution_id=f"intake_{uuid.uuid4().hex[:12]}",
            name=company or full_name,
            institution_type="Live Chat Qualified Visitor",
            city="Online",
        )
        session.add(lead)
        await session.flush()

        contact_fields = {
            "prechat_answers": answers,
            "company": company,
            "first_intake_at": datetime.now(UTC).isoformat(),
        }
        if phone:
            contact_fields["phone"] = phone

        contact = Contact(
            lead_id=lead.id,
            channel=ChannelType.WEBSITE,
            value=email,
            person_name=full_name,
            custom_fields=contact_fields,
        )
        session.add(contact)
        await session.flush()
        await record_consent(
            session,
            contact.id,
            ConsentStatus.OPTED_IN,
            source="prechat_intake_form",
            note=f"Submitted pre-chat intake on {creds.get('websiteDomain', 'website')}",
        )
    else:
        if full_name:
            contact.person_name = full_name
        cf = dict(contact.custom_fields or {})
        if phone:
            cf["phone"] = phone
        cf["prechat_answers"] = answers
        if company:
            cf["company"] = company
        cf["last_intake_at"] = datetime.now(UTC).isoformat()
        contact.custom_fields = cf

    # 5. Evaluate conditional routing rules
    routing_rules = prechat_cfg.get("routing_rules", DEFAULT_ROUTING_RULES)
    routing_result = evaluate_routing_rules(routing_rules, answers)

    # 6. Find or Create Conversation
    conv_stmt = (
        select(Conversation)
        .where(Conversation.contact_id == contact.id)
        .order_by(Conversation.created_at.desc())
    )
    conv = (await session.execute(conv_stmt)).scalars().first()

    is_new_conv = False
    if not conv:
        is_new_conv = True
        custom_attrs = {
            "source": "website_live_chat",
            "visitor_email": email,
            "intake_submission": answers,
            "tags": routing_result["tags"],
            "assigned_team": routing_result["team_target"],
            "client_ip": client_ip,
        }
        conv = Conversation(
            contact_id=contact.id,
            inbox_id=target_inbox.id,
            priority=routing_result["priority"],
            custom_attributes=custom_attrs,
        )
        session.add(conv)
        await session.flush()
    else:
        conv.priority = routing_result["priority"]
        attrs = dict(conv.custom_attributes or {})
        existing_tags = list(attrs.get("tags") or [])
        for t in routing_result["tags"]:
            if t not in existing_tags:
                existing_tags.append(t)
        attrs["tags"] = existing_tags
        attrs["intake_submission"] = answers
        if routing_result["team_target"]:
            attrs["assigned_team"] = routing_result["team_target"]
        conv.custom_attributes = attrs

    # 7. Auto-inject greeting if conversation is brand new
    if is_new_conv and target_inbox.greeting_enabled and target_inbox.greeting_message:
        greet = Message(
            conversation_id=conv.id,
            direction=MessageDirection.OUTBOUND,
            channel=ChannelType.WEBSITE,
            body=target_inbox.greeting_message,
            provider="web_widget",
            provider_message_id=f"greet_{uuid.uuid4().hex[:12]}",
            status=MessageStatus.DELIVERED,
            sender_type="agent",
            sender_name=target_inbox.name,
            sent_at=datetime.now(UTC),
        )
        session.add(greet)

    # 8. If visitor submitted an initial question/message in the form, inject as inbound message
    inbound_msg_id: str | None = None
    if initial_message:
        user_msg = Message(
            conversation_id=conv.id,
            direction=MessageDirection.INBOUND,
            channel=ChannelType.WEBSITE,
            body=initial_message,
            provider="web_widget",
            provider_message_id=f"intake_msg_{uuid.uuid4().hex[:12]}",
            status=MessageStatus.RECEIVED,
            sender_type="contact",
            sender_name=full_name,
            sent_at=datetime.now(UTC),
        )
        session.add(user_msg)
        await session.flush()
        inbound_msg_id = str(user_msg.id)

    await session.flush()

    return {
        "success": True,
        "contact_id": str(contact.id),
        "conversation_id": str(conv.id),
        "visitor_name": contact.person_name,
        "visitor_email": contact.value,
        "routing_applied": routing_result,
        "initial_message_id": inbound_msg_id,
        "is_new_conversation": is_new_conv,
    }
