from __future__ import annotations

import uuid
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services import translation_service

router = APIRouter(prefix="/translation", tags=["Auto-Translation"])


class DetectLanguagePayload(BaseModel):
    text: str = Field(..., description="Text snippet to analyze")


class TranslateTextPayload(BaseModel):
    text: str = Field(..., description="Text to translate")
    target_language: str = Field(..., description="Target ISO language code, e.g. en, es, fr")
    source_language: str | None = Field(default=None, description="Optional source language code")


class OutboundTranslatePayload(BaseModel):
    text: str = Field(..., description="Original agent reply text in primary language")
    target_language: str | None = Field(
        default=None, description="Optional explicit target language"
    )


class TranslationSettingsUpdatePayload(BaseModel):
    is_enabled: bool | None = None
    auto_translate_inbound: bool | None = None
    auto_translate_outbound: bool | None = None
    agent_primary_language: str | None = None
    preferred_engine: str | None = None
    confidence_threshold: float | None = None
    enabled_languages: list[str] | None = None


@router.get("/languages")
async def list_supported_languages():
    """
    Returns the catalog of 50+ supported languages, script types, and native names.
    """
    return {
        "languages": list(translation_service.SUPPORTED_LANGUAGES.values()),
        "total": len(translation_service.SUPPORTED_LANGUAGES),
    }


@router.get("/settings")
async def get_translation_settings(db: AsyncSession = Depends(get_db_session)):
    """
    Retrieves the workspace bi-directional auto-translation settings.
    """
    settings = await translation_service.get_or_create_translation_settings(db)
    return {
        "settings": {
            "id": str(settings.id),
            "is_enabled": settings.is_enabled,
            "auto_translate_inbound": settings.auto_translate_inbound,
            "auto_translate_outbound": settings.auto_translate_outbound,
            "agent_primary_language": settings.agent_primary_language,
            "preferred_engine": settings.preferred_engine,
            "confidence_threshold": settings.confidence_threshold,
            "enabled_languages": settings.enabled_languages,
            "created_at": settings.created_at.isoformat(),
            "updated_at": settings.updated_at.isoformat(),
        }
    }


@router.put("/settings")
async def update_translation_settings(
    payload: TranslationSettingsUpdatePayload,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Updates workspace bi-directional auto-translation settings.
    """
    updates = payload.model_dump(exclude_unset=True)
    updated = await translation_service.update_translation_settings(db, updates)
    return {
        "settings": {
            "id": str(updated.id),
            "is_enabled": updated.is_enabled,
            "auto_translate_inbound": updated.auto_translate_inbound,
            "auto_translate_outbound": updated.auto_translate_outbound,
            "agent_primary_language": updated.agent_primary_language,
            "preferred_engine": updated.preferred_engine,
            "confidence_threshold": updated.confidence_threshold,
            "enabled_languages": updated.enabled_languages,
            "updated_at": updated.updated_at.isoformat(),
        }
    }


@router.post("/detect")
async def detect_language_endpoint(payload: DetectLanguagePayload):
    """
    Detects the natural language and confidence level of an incoming message.
    """
    return translation_service.detect_language(payload.text)


@router.post("/translate")
async def translate_text_endpoint(payload: TranslateTextPayload):
    """
    Translates arbitrary text from source language to target language.
    """
    return await translation_service.translate_text(
        text=payload.text,
        target_language=payload.target_language,
        source_language=payload.source_language,
    )


@router.post("/messages/{message_id}/translate")
async def translate_message_endpoint(
    message_id: uuid.UUID,
    target_language: str = Query("en", description="Target ISO language code, defaults to en"),
    db: AsyncSession = Depends(get_db_session),
):
    """
    Translates an incoming or existing message on-demand and saves translation to message metadata.
    """
    try:
        translation = await translation_service.translate_message(
            session=db,
            message_id=message_id,
            target_language=target_language,
        )
        return {"success": True, "message_id": str(message_id), "translation": translation}
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@router.post("/conversations/{conversation_id}/prepare-outbound")
async def prepare_outbound_translation_endpoint(
    conversation_id: uuid.UUID,
    payload: OutboundTranslatePayload,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Prepares a translated outbound reply based on customer's detected language.
    """
    try:
        result = await translation_service.prepare_outbound_translation(
            session=db,
            conversation_id=conversation_id,
            text=payload.text,
            target_language=payload.target_language,
        )
        return result
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))
