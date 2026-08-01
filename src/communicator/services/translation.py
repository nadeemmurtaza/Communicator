"""Phase 5 multi-language support -- see docs/ARCHITECTURE.md section 9."""

from __future__ import annotations

from communicator.ai.claude_client import call_claude

_SYSTEM_PROMPT = (
    "Translate the user's message into the target language. Respond with "
    "only the translated text, no quotes, no commentary, no markdown."
)


async def translate_text(text: str, target_language: str) -> str:
    if not text or target_language in ("", "en"):
        return text
    return await call_claude(_SYSTEM_PROMPT, f"Target language: {target_language}\n\n{text}")
