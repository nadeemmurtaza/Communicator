"""Phase 5 multi-language support -- see docs/ARCHITECTURE.md section 9."""

from __future__ import annotations

from communicator.services.translation_service import translate_text as _service_translate_text


async def translate_text(text: str, target_language: str) -> str:
    """Translates text to target language, delegating to the unified translation service."""
    if not text or target_language in ("", "en"):
        return text
    res = await _service_translate_text(text=text, target_language=target_language)
    return res.get("translated_text", text)
