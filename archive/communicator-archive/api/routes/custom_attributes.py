from __future__ import annotations

import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services import custom_attribute_service

router = APIRouter(prefix="/custom-attributes", tags=["Custom Attributes & CRM Schema"])


class CreateAttributeIn(BaseModel):
    name: str = Field(..., description="Slug identifier e.g. account_tier")
    display_name: str = Field(..., description="Display label e.g. Account Tier")
    description: str = ""
    attribute_model: str = Field(
        default="contact_attribute", description="contact_attribute | conversation_attribute"
    )
    attribute_type: str = Field(
        default="text",
        description="text | number | currency | boolean | list | date | link",
    )
    options: list[str] = Field(
        default_factory=list, description="Options for list/select attribute types"
    )
    default_value: str | None = None
    is_required: bool = False
    regex_pattern: str | None = None
    regex_cue: str | None = None


class UpdateAttributeIn(BaseModel):
    display_name: str | None = None
    description: str | None = None
    options: list[str] | None = None
    default_value: str | None = None
    is_required: bool | None = None
    regex_pattern: str | None = None
    regex_cue: str | None = None


class SetAttributesPayload(BaseModel):
    attributes: dict[str, Any] = Field(..., description="Dictionary of attribute keys and values")


def _serialize_attr(a: Any) -> dict[str, Any]:
    return {
        "id": str(a.id),
        "name": a.name,
        "display_name": a.display_name,
        "description": a.description,
        "attribute_model": a.attribute_model,
        "attribute_type": a.attribute_type,
        "options": a.options or [],
        "default_value": a.default_value,
        "is_required": a.is_required,
        "regex_pattern": a.regex_pattern,
        "regex_cue": a.regex_cue,
        "created_at": a.created_at.isoformat(),
        "updated_at": a.updated_at.isoformat(),
    }


@router.get("")
async def list_custom_attributes(
    attribute_model: str | None = Query(
        None, description="Optional filter: contact_attribute or conversation_attribute"
    ),
    db: AsyncSession = Depends(get_db_session),
):
    """
    Lists custom attribute definitions, grouped or filtered by target entity model.
    """
    items = await custom_attribute_service.list_attribute_definitions(
        db, attribute_model=attribute_model
    )
    return [_serialize_attr(i) for i in items]


@router.post("")
async def create_custom_attribute(
    payload: CreateAttributeIn,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Creates a new custom attribute definition in the CRM schema.
    """
    try:
        created = await custom_attribute_service.create_attribute_definition(
            session=db,
            data=payload.model_dump(),
            actor="admin",
        )
        return _serialize_attr(created)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.get("/{id_or_name}")
async def get_custom_attribute(
    id_or_name: str,
    attribute_model: str | None = None,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Retrieves a specific custom attribute definition.
    """
    item = await custom_attribute_service.get_attribute_definition(
        db, id_or_name, attribute_model=attribute_model
    )
    if not item:
        raise HTTPException(status_code=404, detail="Custom attribute definition not found.")
    return _serialize_attr(item)


@router.put("/{id_or_name}")
async def update_custom_attribute(
    id_or_name: str,
    payload: UpdateAttributeIn,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Updates an existing custom attribute definition.
    """
    try:
        updated = await custom_attribute_service.update_attribute_definition(
            session=db,
            id_or_name=id_or_name,
            updates=payload.model_dump(exclude_unset=True),
            actor="admin",
        )
        return _serialize_attr(updated)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.delete("/{id_or_name}")
async def delete_custom_attribute(
    id_or_name: str,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Deletes a custom attribute definition.
    """
    try:
        success = await custom_attribute_service.delete_attribute_definition(
            session=db,
            id_or_name=id_or_name,
            actor="admin",
        )
        return {"success": success, "deleted": id_or_name}
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.post("/contacts/{contact_id}/values")
async def update_contact_attributes(
    contact_id: uuid.UUID,
    payload: SetAttributesPayload,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Validates and updates custom attributes on a Contact.
    """
    try:
        updated_fields = await custom_attribute_service.set_contact_custom_attributes(
            session=db,
            contact_id=contact_id,
            attributes=payload.attributes,
            actor="agent",
        )
        return {"success": True, "contact_id": str(contact_id), "custom_fields": updated_fields}
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.post("/conversations/{conversation_id}/values")
async def update_conversation_attributes(
    conversation_id: uuid.UUID,
    payload: SetAttributesPayload,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Validates and updates custom attributes on a Conversation.
    """
    try:
        updated_attrs = await custom_attribute_service.set_conversation_custom_attributes(
            session=db,
            conversation_id=conversation_id,
            attributes=payload.attributes,
            actor="agent",
        )
        return {
            "success": True,
            "conversation_id": str(conversation_id),
            "custom_attributes": updated_attrs,
        }
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
