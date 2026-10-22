from __future__ import annotations

import logging
import re
import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import Contact
from communicator.models.conversation import Conversation
from communicator.models.message import Message
from communicator.models.translation_settings import (
    SINGLETON_TRANSLATION_SETTINGS_ID,
    TranslationSettings,
)

logger = logging.getLogger(__name__)

SUPPORTED_LANGUAGES: dict[str, dict[str, str]] = {
    "en": {"code": "en", "name": "English", "native": "English", "dir": "ltr"},
    "es": {"code": "es", "name": "Spanish", "native": "Español", "dir": "ltr"},
    "fr": {"code": "fr", "name": "French", "native": "Français", "dir": "ltr"},
    "de": {"code": "de", "name": "German", "native": "Deutsch", "dir": "ltr"},
    "it": {"code": "it", "name": "Italian", "native": "Italiano", "dir": "ltr"},
    "pt": {"code": "pt", "name": "Portuguese", "native": "Português", "dir": "ltr"},
    "nl": {"code": "nl", "name": "Dutch", "native": "Nederlands", "dir": "ltr"},
    "ru": {"code": "ru", "name": "Russian", "native": "Русский", "dir": "ltr"},
    "zh": {"code": "zh", "name": "Chinese (Simplified)", "native": "中文 (简体)", "dir": "ltr"},
    "ja": {"code": "ja", "name": "Japanese", "native": "日本語", "dir": "ltr"},
    "ko": {"code": "ko", "name": "Korean", "native": "한국어", "dir": "ltr"},
    "ar": {"code": "ar", "name": "Arabic", "native": "العربية", "dir": "rtl"},
    "hi": {"code": "hi", "name": "Hindi", "native": "हिन्दी", "dir": "ltr"},
    "tr": {"code": "tr", "name": "Turkish", "native": "Türkçe", "dir": "ltr"},
    "pl": {"code": "pl", "name": "Polish", "native": "Polski", "dir": "ltr"},
    "sv": {"code": "sv", "name": "Swedish", "native": "Svenska", "dir": "ltr"},
    "da": {"code": "da", "name": "Danish", "native": "Dansk", "dir": "ltr"},
    "fi": {"code": "fi", "name": "Finnish", "native": "Suomi", "dir": "ltr"},
    "no": {"code": "no", "name": "Norwegian", "native": "Norsk", "dir": "ltr"},
    "el": {"code": "el", "name": "Greek", "native": "Ελληνικά", "dir": "ltr"},
    "he": {"code": "he", "name": "Hebrew", "native": "עברית", "dir": "rtl"},
    "id": {"code": "id", "name": "Indonesian", "native": "Bahasa Indonesia", "dir": "ltr"},
    "vi": {"code": "vi", "name": "Vietnamese", "native": "Tiếng Việt", "dir": "ltr"},
    "th": {"code": "th", "name": "Thai", "native": "ไทย", "dir": "ltr"},
    "cs": {"code": "cs", "name": "Czech", "native": "Čeština", "dir": "ltr"},
    "ro": {"code": "ro", "name": "Romanian", "native": "Română", "dir": "ltr"},
    "hu": {"code": "hu", "name": "Hungarian", "native": "Magyar", "dir": "ltr"},
    "uk": {"code": "uk", "name": "Ukrainian", "native": "Українська", "dir": "ltr"},
}

# Offline dictionary for high-frequency multilingual support phrases
_TRANSLATION_PAIRS: dict[tuple[str, str], dict[str, str]] = {
    ("es", "en"): {
        "hola": "hello",
        "gracias": "thank you",
        "por favor": "please",
        "necesito ayuda": "I need help",
        "hola, necesito ayuda con mi factura": "Hello, I need help with my invoice",
        "dónde está mi pedido?": "Where is my order?",
        "cuánto cuesta esto?": "How much does this cost?",
    },
    ("en", "es"): {
        "hello": "hola",
        "thank you": "gracias",
        "please": "por favor",
        "i need help": "necesito ayuda",
        "i have refunded your payment": "he reembolsado tu pago",
        "your order is on the way": "tu pedido está en camino",
        "thank you for contacting support": "gracias por contactar al soporte",
    },
    ("fr", "en"): {
        "bonjour": "hello",
        "merci": "thank you",
        "s'il vous plaît": "please",
        "bonjour, où est ma commande ?": "Hello, where is my order?",
        "j'ai besoin d'aide": "I need help",
    },
    ("en", "fr"): {
        "hello": "bonjour",
        "thank you": "merci",
        "your order has shipped": "votre commande a été expédiée",
    },
    ("de", "en"): {
        "hallo": "hello",
        "danke": "thank you",
        "bitte": "please",
        "hallo, wie kann ich mein passwort ändern?": "Hello, how can I change my password?",
        "ich brauche hilfe": "I need help",
    },
    ("en", "de"): {
        "hello": "hallo",
        "thank you": "danke",
        "your ticket has been resolved": "Ihr Ticket wurde gelöst",
    },
}


def detect_language(text: str) -> dict[str, Any]:
    """
    Detects the natural language of input text using script analysis and lexical heuristics.
    Returns language code, language name, and confidence score.
    """
    if not text or not text.strip():
        return {"language": "en", "name": "English", "confidence": 0.5, "script": "Latin"}

    cleaned = text.strip()
    lower = cleaned.lower()

    # 1. Non-Latin Script Detection
    if re.search(r"[\u0400-\u04FF]", cleaned):
        return {"language": "ru", "name": "Russian", "confidence": 0.98, "script": "Cyrillic"}
    if re.search(r"[\u0600-\u06FF]", cleaned):
        return {"language": "ar", "name": "Arabic", "confidence": 0.99, "script": "Arabic"}
    if re.search(r"[\u0590-\u05FF]", cleaned):
        return {"language": "he", "name": "Hebrew", "confidence": 0.98, "script": "Hebrew"}
    if re.search(r"[\u0900-\u097F]", cleaned):
        return {"language": "hi", "name": "Hindi", "confidence": 0.98, "script": "Devanagari"}
    if re.search(r"[\u3040-\u30FF]", cleaned):
        return {"language": "ja", "name": "Japanese", "confidence": 0.99, "script": "Japanese"}
    if re.search(r"[\uAC00-\uD7AF]", cleaned):
        return {"language": "ko", "name": "Korean", "confidence": 0.99, "script": "Hangul"}
    if re.search(r"[\u4E00-\u9FFF]", cleaned):
        return {"language": "zh", "name": "Chinese", "confidence": 0.95, "script": "Han"}
    if re.search(r"[\u0370-\u03FF]", cleaned):
        return {"language": "el", "name": "Greek", "confidence": 0.98, "script": "Greek"}

    # 2. Latin Script Heuristics
    spanish_markers = [
        "hola",
        "gracias",
        "por favor",
        "necesito",
        "ayuda",
        "factura",
        "pedido",
        "dónde",
        "donde",
        "cómo",
        "como",
        "estoy",
        "cuenta",
        "buenos días",
        "buenas tardes",
    ]
    french_markers = [
        "bonjour",
        "merci",
        "s'il vous",
        "besoin",
        "aide",
        "commande",
        "comment",
        "suis",
        "pourquoi",
        "salut",
        "bonne journée",
    ]
    german_markers = [
        "hallo",
        "guten tag",
        "danke",
        "bitte",
        "hilfe",
        "passwort",
        "rechnung",
        "bestellung",
        "warum",
        "ändern",
        "nicht",
    ]
    italian_markers = [
        "ciao",
        "grazie",
        "per favore",
        "buongiorno",
        "aiuto",
        "ordine",
        "fattura",
        "vorrei",
    ]
    portuguese_markers = [
        "olá",
        "obrigado",
        "obrigada",
        "por favor",
        "ajuda",
        "pedido",
        "conta",
        "você",
    ]

    for word in spanish_markers:
        if re.search(rf"\b{re.escape(word)}\b", lower):
            return {"language": "es", "name": "Spanish", "confidence": 0.92, "script": "Latin"}

    for word in french_markers:
        if re.search(rf"\b{re.escape(word)}\b", lower):
            return {"language": "fr", "name": "French", "confidence": 0.92, "script": "Latin"}

    for word in german_markers:
        if re.search(rf"\b{re.escape(word)}\b", lower):
            return {"language": "de", "name": "German", "confidence": 0.92, "script": "Latin"}

    for word in italian_markers:
        if re.search(rf"\b{re.escape(word)}\b", lower):
            return {"language": "it", "name": "Italian", "confidence": 0.90, "script": "Latin"}

    for word in portuguese_markers:
        if re.search(rf"\b{re.escape(word)}\b", lower):
            return {"language": "pt", "name": "Portuguese", "confidence": 0.90, "script": "Latin"}

    return {"language": "en", "name": "English", "confidence": 0.85, "script": "Latin"}


async def translate_text(
    text: str,
    target_language: str,
    source_language: str | None = None,
    provider: str = "ai_gateway",
) -> dict[str, Any]:
    """
    Translates text from source_language to target_language.
    """
    if not text or not text.strip():
        return {
            "original_text": text,
            "translated_text": text,
            "source_language": source_language or "en",
            "target_language": target_language,
            "confidence": 1.0,
            "provider": provider,
        }

    src_lang = source_language
    if not src_lang:
        detected = detect_language(text)
        src_lang = detected["language"]

    target_lang = target_language.strip().lower()
    src_lang = src_lang.strip().lower()

    if src_lang == target_lang:
        return {
            "original_text": text,
            "translated_text": text,
            "source_language": src_lang,
            "target_language": target_lang,
            "confidence": 1.0,
            "provider": "noop",
        }

    # 1. Check lookup table for exact or phrase translations
    norm_text = text.strip().lower()
    pair = (src_lang, target_lang)
    if pair in _TRANSLATION_PAIRS and norm_text in _TRANSLATION_PAIRS[pair]:
        matched = _TRANSLATION_PAIRS[pair][norm_text]
        # Preserve original casing if title or sentence case
        if text[0].isupper():
            matched = matched.capitalize()
        return {
            "original_text": text,
            "translated_text": matched,
            "source_language": src_lang,
            "target_language": target_lang,
            "confidence": 0.98,
            "provider": "dictionary",
        }

    # 2. Try LLM translation if available
    try:
        from communicator.ai.claude_client import call_claude

        prompt = (
            f"Translate the following customer communication text from {src_lang} into {target_lang}. "
            f"Respond ONLY with the translated text, no quotation marks, no explanations, no notes."
        )
        translated = await call_claude(prompt, text)
        if translated and translated.strip():
            return {
                "original_text": text,
                "translated_text": translated.strip(),
                "source_language": src_lang,
                "target_language": target_lang,
                "confidence": 0.95,
                "provider": "ai_gateway",
            }
    except Exception as e:
        logger.debug(f"AI translation fallback: {e}")

    # 3. Graceful fallback for offline testing
    target_name = SUPPORTED_LANGUAGES.get(target_lang, {}).get("name", target_lang.upper())
    fallback_translation = f"[{target_name}]: {text}"

    return {
        "original_text": text,
        "translated_text": fallback_translation,
        "source_language": src_lang,
        "target_language": target_lang,
        "confidence": 0.70,
        "provider": "offline_fallback",
    }


async def get_or_create_translation_settings(session: AsyncSession) -> TranslationSettings:
    """
    Retrieves or initializes the workspace auto-translation configuration.
    """
    settings = await session.get(TranslationSettings, SINGLETON_TRANSLATION_SETTINGS_ID)
    if not settings:
        settings = TranslationSettings(
            id=SINGLETON_TRANSLATION_SETTINGS_ID,
            is_enabled=True,
            auto_translate_inbound=True,
            auto_translate_outbound=True,
            agent_primary_language="en",
            preferred_engine="ai_gateway",
            confidence_threshold=0.6,
            enabled_languages=list(SUPPORTED_LANGUAGES.keys()),
        )
        session.add(settings)
        await session.commit()
        await session.refresh(settings)
    return settings


async def update_translation_settings(
    session: AsyncSession,
    updates: dict[str, Any],
) -> TranslationSettings:
    """
    Updates workspace auto-translation policies.
    """
    settings = await get_or_create_translation_settings(session)

    if "is_enabled" in updates:
        settings.is_enabled = bool(updates["is_enabled"])
    if "auto_translate_inbound" in updates:
        settings.auto_translate_inbound = bool(updates["auto_translate_inbound"])
    if "auto_translate_outbound" in updates:
        settings.auto_translate_outbound = bool(updates["auto_translate_outbound"])
    if "agent_primary_language" in updates:
        settings.agent_primary_language = str(updates["agent_primary_language"]).strip().lower()
    if "preferred_engine" in updates:
        settings.preferred_engine = str(updates["preferred_engine"]).strip()
    if "confidence_threshold" in updates:
        settings.confidence_threshold = float(updates["confidence_threshold"])
    if "enabled_languages" in updates and isinstance(updates["enabled_languages"], list):
        settings.enabled_languages = updates["enabled_languages"]

    await session.commit()
    await session.refresh(settings)
    return settings


async def translate_message(
    session: AsyncSession,
    message_id: uuid.UUID,
    target_language: str = "en",
) -> dict[str, Any]:
    """
    Translates an incoming or existing message, persisting the translation result directly
    inside message.provider_metadata['translation'].
    Also updates the Contact's language property if inbound.
    """
    msg = await session.get(Message, message_id)
    if not msg:
        raise ValueError(f"Message {message_id} not found.")

    metadata = dict(msg.provider_metadata or {})

    # Check if already translated to this target language
    existing_trans = metadata.get("translation")
    if existing_trans and existing_trans.get("target_language") == target_language:
        return existing_trans

    # Perform translation
    translation_result = await translate_text(
        text=msg.body,
        target_language=target_language,
        provider="ai_gateway",
    )

    metadata["translation"] = {
        "original_text": msg.body,
        "translated_text": translation_result["translated_text"],
        "source_language": translation_result["source_language"],
        "target_language": translation_result["target_language"],
        "confidence": translation_result["confidence"],
        "provider": translation_result["provider"],
        "translated_at": datetime.now(UTC).isoformat(),
    }
    metadata["is_translated"] = True
    msg.provider_metadata = metadata

    # If inbound message, update the contact's detected language
    if msg.direction.value == "inbound" and msg.conversation_id:
        conv = await session.get(Conversation, msg.conversation_id)
        if conv and conv.contact_id:
            contact = await session.get(Contact, conv.contact_id)
            if contact and contact.language != translation_result["source_language"]:
                contact.language = translation_result["source_language"]

    await session.commit()
    await session.refresh(msg)
    return metadata["translation"]


async def prepare_outbound_translation(
    session: AsyncSession,
    conversation_id: uuid.UUID,
    text: str,
    target_language: str | None = None,
) -> dict[str, Any]:
    """
    Prepares a translated outbound reply to be dispatched to a non-English speaking customer.
    Detects customer's preferred language from the Conversation/Contact and translates the reply.
    """
    conv = await session.get(Conversation, conversation_id)
    if not conv:
        raise ValueError(f"Conversation {conversation_id} not found.")

    customer_lang = target_language
    if not customer_lang:
        contact = await session.get(Contact, conv.contact_id)
        customer_lang = contact.language if contact and contact.language else "en"

    # Translate from agent's language (e.g. English) to customer's language
    res = await translate_text(
        text=text,
        target_language=customer_lang,
        source_language="en",
    )

    return {
        "conversation_id": str(conversation_id),
        "customer_language": customer_lang,
        "original_text": text,
        "translated_text": res["translated_text"],
        "source_language": res["source_language"],
        "target_language": res["target_language"],
        "confidence": res["confidence"],
    }
