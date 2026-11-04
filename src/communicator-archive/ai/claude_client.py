from __future__ import annotations

import json

import httpx

from communicator.config import settings

API_URL = "https://api.anthropic.com/v1/messages"


async def call_claude(system: str, user: str, max_tokens: int = 1024) -> str:
    headers = {
        "x-api-key": settings.anthropic_api_key,
        "anthropic-version": "2023-06-01",
        "content-type": "application/json",
    }
    payload = {
        "model": settings.anthropic_model,
        "max_tokens": max_tokens,
        "system": system,
        "messages": [{"role": "user", "content": user}],
    }
    async with httpx.AsyncClient(timeout=60.0) as client:
        response = await client.post(API_URL, headers=headers, json=payload)
        response.raise_for_status()
        data = response.json()
    return data["content"][0]["text"]


def parse_json_reply(text: str) -> dict:
    text = text.strip()
    if text.startswith("```"):
        text = text.strip("`")
        text = text.removeprefix("json")
    return json.loads(text)
