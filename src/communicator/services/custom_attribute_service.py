from __future__ import annotations

import logging
import re
import uuid
from datetime import datetime
from typing import Any

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import Contact
from communicator.models.conversation import Conversation
from communicator.models.custom_attribute import CustomAttributeDefinition
from communicator.services.audit_service import record_audit_event

logger = logging.getLogger(__name__)

ALLOWED_TYPES = {"text", "number", "currency", "boolean", "list", "date", "link"}
ALLOWED_MODELS = {"contact_attribute", "conversation_attribute"}

DEFAULT_ATTRIBUTES = [
    # Contact Attributes
    {
        "name": "account_tier",
        "display_name": "Account Tier",
        "description": "Customer commercial subscription tier",
        "attribute_model": "contact_attribute",
        "attribute_type": "list",
        "options": ["Free", "Starter", "Growth", "Enterprise"],
        "default_value": "Starter",
        "is_required": False,
    },
    {
        "name": "company_size",
        "display_name": "Company Size",
        "description": "Number of employees at lead's institution",
        "attribute_model": "contact_attribute",
        "attribute_type": "list",
        "options": ["1-10", "11-50", "51-200", "201-1000", "1000+"],
        "default_value": "11-50",
        "is_required": False,
    },
    {
        "name": "annual_revenue",
        "display_name": "Annual Revenue (ARR)",
        "description": "Contract value or expected recurring revenue",
        "attribute_model": "contact_attribute",
        "attribute_type": "currency",
        "default_value": "0",
        "is_required": False,
    },
    {
        "name": "is_vip",
        "display_name": "VIP Customer",
        "description": "High-priority enterprise account flag",
        "attribute_model": "contact_attribute",
        "attribute_type": "boolean",
        "default_value": "false",
        "is_required": False,
    },
    {
        "name": "renewal_date",
        "display_name": "Contract Renewal Date",
        "description": "Upcoming subscription expiration or renewal date",
        "attribute_model": "contact_attribute",
        "attribute_type": "date",
        "is_required": False,
    },
    # Conversation Attributes
    {
        "name": "issue_category",
        "display_name": "Issue Category",
        "description": "Triage topic classification for this thread",
        "attribute_model": "conversation_attribute",
        "attribute_type": "list",
        "options": [
            "Billing",
            "Technical Support",
            "Feature Request",
            "Account Access",
            "Sales Inquiry",
            "General Inquiry",
        ],
        "default_value": "General Inquiry",
        "is_required": False,
    },
    {
        "name": "escalation_level",
        "display_name": "Escalation Level",
        "description": "Support tier handling this conversation",
        "attribute_model": "conversation_attribute",
        "attribute_type": "list",
        "options": ["Tier 1", "Tier 2", "Engineering", "Executive Escrow"],
        "default_value": "Tier 1",
        "is_required": False,
    },
    {
        "name": "billing_dispute_amount",
        "display_name": "Dispute Amount (USD)",
        "description": "Customer requested refund or dispute total",
        "attribute_model": "conversation_attribute",
        "attribute_type": "currency",
        "is_required": False,
    },
    {
        "name": "jira_key",
        "display_name": "Linked Issue Key",
        "description": "External issue tracker ticket reference (e.g. ENG-1234)",
        "attribute_model": "conversation_attribute",
        "attribute_type": "text",
        "is_required": False,
    },
]


def _normalize_name(name: str) -> str:
    cleaned = re.sub(r"[^a-zA-Z0-9_]", "_", name.strip().lower())
    return re.sub(r"_+", "_", cleaned).strip("_")


async def initialize_default_attributes(session: AsyncSession) -> None:
    """
    Auto-seeds default enterprise CRM custom attribute definitions if the table is empty.
    """
    existing_count = (
        await session.scalar(select(CustomAttributeDefinition.id).limit(1))
    ) is not None
    if existing_count:
        return

    for item in DEFAULT_ATTRIBUTES:
        attr = CustomAttributeDefinition(
            name=item["name"],
            display_name=item["display_name"],
            description=item.get("description", ""),
            attribute_model=item["attribute_model"],
            attribute_type=item["attribute_type"],
            options=item.get("options", []),
            default_value=item.get("default_value"),
            is_required=item.get("is_required", False),
        )
        session.add(attr)
    await session.commit()


async def list_attribute_definitions(
    session: AsyncSession,
    attribute_model: str | None = None,
) -> list[CustomAttributeDefinition]:
    """
    Lists custom attribute definitions, optionally filtered by attribute_model.
    """
    await initialize_default_attributes(session)
    stmt = select(CustomAttributeDefinition)
    if attribute_model:
        stmt = stmt.where(CustomAttributeDefinition.attribute_model == attribute_model)
    stmt = stmt.order_by(CustomAttributeDefinition.display_name.asc())
    return (await session.scalars(stmt)).all()


async def get_attribute_definition(
    session: AsyncSession,
    id_or_name: str,
    attribute_model: str | None = None,
) -> CustomAttributeDefinition | None:
    """
    Finds a custom attribute definition by UUID or slug name.
    """
    await initialize_default_attributes(session)
    try:
        val_uuid = uuid.UUID(id_or_name)
        attr = await session.get(CustomAttributeDefinition, val_uuid)
        if attr:
            return attr
    except ValueError:
        pass

    stmt = select(CustomAttributeDefinition).where(CustomAttributeDefinition.name == id_or_name)
    if attribute_model:
        stmt = stmt.where(CustomAttributeDefinition.attribute_model == attribute_model)
    return await session.scalar(stmt)


async def create_attribute_definition(
    session: AsyncSession,
    data: dict[str, Any],
    actor: str = "admin",
) -> CustomAttributeDefinition:
    """
    Creates a new custom attribute definition and records a SOC 2 audit log.
    """
    raw_name = data.get("name", "")
    slug_name = _normalize_name(raw_name)
    if not slug_name:
        raise ValueError("Attribute name (slug) cannot be empty.")

    attr_model = data.get("attribute_model", "contact_attribute")
    if attr_model not in ALLOWED_MODELS:
        raise ValueError(
            f"Invalid attribute_model '{attr_model}'. Allowed models: {sorted(ALLOWED_MODELS)}"
        )

    attr_type = data.get("attribute_type", "text")
    if attr_type not in ALLOWED_TYPES:
        raise ValueError(
            f"Invalid attribute_type '{attr_type}'. Allowed types: {sorted(ALLOWED_TYPES)}"
        )

    # Check for duplicate
    existing = await session.scalar(
        select(CustomAttributeDefinition).where(
            CustomAttributeDefinition.name == slug_name,
            CustomAttributeDefinition.attribute_model == attr_model,
        )
    )
    if existing:
        raise ValueError(
            f"An attribute named '{slug_name}' already exists for model '{attr_model}'."
        )

    options = data.get("options", [])
    if attr_type == "list" and not options:
        raise ValueError("Attributes of type 'list' must specify at least one option.")

    definition = CustomAttributeDefinition(
        name=slug_name,
        display_name=data.get("display_name", slug_name.replace("_", " ").title()),
        description=data.get("description", ""),
        attribute_model=attr_model,
        attribute_type=attr_type,
        options=options if isinstance(options, list) else [],
        default_value=str(data["default_value"]) if data.get("default_value") is not None else None,
        is_required=bool(data.get("is_required", False)),
        regex_pattern=data.get("regex_pattern"),
        regex_cue=data.get("regex_cue"),
    )
    session.add(definition)
    await session.commit()
    await session.refresh(definition)

    await record_audit_event(
        session=session,
        action="custom_attribute.created",
        entity_type="custom_attribute_definition",
        entity_id=definition.id,
        actor=actor,
        compliance_category="configuration_change",
        details={
            "name": definition.name,
            "display_name": definition.display_name,
            "attribute_model": definition.attribute_model,
            "attribute_type": definition.attribute_type,
        },
        description=f"Created custom CRM attribute '{definition.display_name}' ({definition.name}) for {definition.attribute_model}",
    )

    return definition


async def update_attribute_definition(
    session: AsyncSession,
    id_or_name: str,
    updates: dict[str, Any],
    actor: str = "admin",
) -> CustomAttributeDefinition:
    """
    Updates an existing custom attribute definition.
    """
    definition = await get_attribute_definition(session, id_or_name)
    if not definition:
        raise ValueError(f"Custom attribute '{id_or_name}' not found.")

    if "display_name" in updates:
        definition.display_name = str(updates["display_name"]).strip()
    if "description" in updates:
        definition.description = str(updates["description"]).strip()
    if "options" in updates and isinstance(updates["options"], list):
        definition.options = updates["options"]
    if "is_required" in updates:
        definition.is_required = bool(updates["is_required"])
    if "default_value" in updates:
        definition.default_value = (
            str(updates["default_value"]) if updates["default_value"] is not None else None
        )
    if "regex_pattern" in updates:
        definition.regex_pattern = updates["regex_pattern"]
    if "regex_cue" in updates:
        definition.regex_cue = updates["regex_cue"]

    await session.commit()
    await session.refresh(definition)

    await record_audit_event(
        session=session,
        action="custom_attribute.updated",
        entity_type="custom_attribute_definition",
        entity_id=definition.id,
        actor=actor,
        compliance_category="configuration_change",
        details={"name": definition.name, "display_name": definition.display_name},
        description=f"Updated custom CRM attribute '{definition.display_name}' ({definition.name})",
    )

    return definition


async def delete_attribute_definition(
    session: AsyncSession,
    id_or_name: str,
    actor: str = "admin",
) -> bool:
    """
    Deletes a custom attribute definition.
    """
    definition = await get_attribute_definition(session, id_or_name)
    if not definition:
        raise ValueError(f"Custom attribute '{id_or_name}' not found.")

    attr_id = definition.id
    attr_name = definition.name
    await session.delete(definition)
    await session.commit()

    await record_audit_event(
        session=session,
        action="custom_attribute.deleted",
        entity_type="custom_attribute_definition",
        entity_id=attr_id,
        actor=actor,
        compliance_category="configuration_change",
        details={"name": attr_name},
        description=f"Deleted custom CRM attribute '{attr_name}'",
    )

    return True


def validate_attribute_value(
    definition: CustomAttributeDefinition,
    value: Any,
) -> tuple[bool, Any, str | None]:
    """
    Validates and casts a value against a custom attribute schema definition.
    Returns (is_valid, parsed_value, error_message).
    """
    if value is None or value == "":
        if definition.is_required:
            return False, None, f"Attribute '{definition.display_name}' is required."
        return True, None, None

    attr_type = definition.attribute_type

    if attr_type == "text":
        return True, str(value).strip(), None

    elif attr_type in ("number", "currency"):
        cleaned_str = str(value).replace("$", "").replace(",", "").strip()
        try:
            num = float(cleaned_str) if "." in cleaned_str else int(cleaned_str)
            return True, num, None
        except ValueError:
            return (
                False,
                None,
                f"Attribute '{definition.display_name}' must be a valid numeric value.",
            )

    elif attr_type == "boolean":
        if isinstance(value, bool):
            return True, value, None
        s = str(value).strip().lower()
        if s in ("true", "1", "yes", "t"):
            return True, True, None
        if s in ("false", "0", "no", "f"):
            return True, False, None
        return False, None, f"Attribute '{definition.display_name}' must be a boolean (true/false)."

    elif attr_type == "list":
        val_str = str(value).strip()
        if val_str not in definition.options:
            return (
                False,
                None,
                f"Value '{val_str}' is not in allowed options: {definition.options}",
            )
        return True, val_str, None

    elif attr_type == "date":
        val_str = str(value).strip()
        try:
            datetime.fromisoformat(val_str)
            return True, val_str, None
        except ValueError:
            if re.match(r"^\d{4}-\d{2}-\d{2}$", val_str):
                return True, val_str, None
            return (
                False,
                None,
                f"Attribute '{definition.display_name}' must be an ISO format date (YYYY-MM-DD).",
            )

    elif attr_type == "link":
        val_str = str(value).strip()
        if not (val_str.startswith(("http://", "https://"))):
            return (
                False,
                None,
                f"Attribute '{definition.display_name}' must be a valid HTTP/HTTPS URL.",
            )
        return True, val_str, None

    return True, value, None


async def set_contact_custom_attributes(
    session: AsyncSession,
    contact_id: uuid.UUID,
    attributes: dict[str, Any],
    actor: str = "agent",
) -> dict[str, Any]:
    """
    Validates and updates custom attributes on a Contact.
    """
    contact = await session.get(Contact, contact_id)
    if not contact:
        raise ValueError(f"Contact {contact_id} not found.")

    definitions = await list_attribute_definitions(session, attribute_model="contact_attribute")
    def_map = {d.name: d for d in definitions}

    current_fields = dict(contact.custom_fields or {})
    for key, val in attributes.items():
        definition = def_map.get(key)
        if definition:
            valid, parsed_val, err = validate_attribute_value(definition, val)
            if not valid:
                raise ValueError(err)
            current_fields[key] = parsed_val
        else:
            # Open custom field
            current_fields[key] = val

    contact.custom_fields = current_fields
    await session.commit()
    await session.refresh(contact)

    await record_audit_event(
        session=session,
        action="contact.attributes_updated",
        entity_type="contact",
        entity_id=contact.id,
        actor=actor,
        compliance_category="data_update",
        details={"custom_fields": current_fields},
        description=f"Updated CRM custom attributes on contact {contact.person_name or contact.value}",
    )

    return current_fields


async def set_conversation_custom_attributes(
    session: AsyncSession,
    conversation_id: uuid.UUID,
    attributes: dict[str, Any],
    actor: str = "agent",
) -> dict[str, Any]:
    """
    Validates and updates custom attributes on a Conversation.
    """
    conv = await session.get(Conversation, conversation_id)
    if not conv:
        raise ValueError(f"Conversation {conversation_id} not found.")

    definitions = await list_attribute_definitions(
        session, attribute_model="conversation_attribute"
    )
    def_map = {d.name: d for d in definitions}

    current_attrs = dict(conv.custom_attributes or {})
    for key, val in attributes.items():
        definition = def_map.get(key)
        if definition:
            valid, parsed_val, err = validate_attribute_value(definition, val)
            if not valid:
                raise ValueError(err)
            current_attrs[key] = parsed_val
        else:
            current_attrs[key] = val

    conv.custom_attributes = current_attrs
    await session.commit()
    await session.refresh(conv)

    await record_audit_event(
        session=session,
        action="conversation.attributes_updated",
        entity_type="conversation",
        entity_id=conv.id,
        actor=actor,
        compliance_category="data_update",
        details={"custom_attributes": current_attrs},
        description=f"Updated CRM custom attributes on conversation {conv.id}",
    )

    return current_attrs
