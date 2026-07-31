"""
Thin async client over Harvester's REST API.

Communicator never connects to Harvester's database directly -- see
docs/ARCHITECTURE.md and harvester/docs/communicator-integration.md for the
integration contract. Two of the methods below (``qualified``/
``updated_since`` filtering and outcome push) call endpoints that are
proposed but not yet implemented on the Harvester side; they're written
against the contract documented there so this client is ready the moment
those endpoints land.
"""
from __future__ import annotations

import httpx

from communicator.config import settings


class HarvesterClient:
    def __init__(self, base_url: str | None = None, token: str | None = None) -> None:
        self.base_url = (base_url or settings.harvester_api_base_url).rstrip("/")
        self.token = token if token is not None else settings.harvester_api_token

    def _headers(self) -> dict[str, str]:
        headers = {"Accept": "application/json"}
        if self.token:
            headers["Authorization"] = f"Bearer {self.token}"
        return headers

    async def list_qualified_institutions(
        self, updated_since: str | None = None, limit: int = 50, offset: int = 0
    ) -> list[dict]:
        """Incremental pull of leads ready for outreach."""
        params: dict[str, str | int] = {"qualified": "true", "limit": limit, "offset": offset}
        if updated_since:
            params["updated_since"] = updated_since

        async with httpx.AsyncClient(base_url=self.base_url, headers=self._headers(), timeout=30.0) as client:
            response = await client.get("/institutions", params=params)
            response.raise_for_status()
            return response.json().get("items", [])

    async def get_institution(self, institution_id: str) -> dict:
        """Full dossier (contact points, qualification rationale) for AI context."""
        async with httpx.AsyncClient(base_url=self.base_url, headers=self._headers(), timeout=30.0) as client:
            response = await client.get(f"/institutions/{institution_id}")
            response.raise_for_status()
            return response.json()

    async def list_approved_drafts(self) -> list[dict]:
        """Drafts Harvester's own review flow has approved for first-touch send."""
        async with httpx.AsyncClient(base_url=self.base_url, headers=self._headers(), timeout=30.0) as client:
            response = await client.get("/drafts", params={"status": "approved"})
            response.raise_for_status()
            return response.json()

    async def push_outreach_outcome(
        self,
        *,
        institution_id: str,
        draft_id: str | None = None,
        opened: bool = False,
        clicked: bool = False,
        replied: bool = False,
        positive_reply: bool | None = None,
        reply_text: str = "",
    ) -> dict:
        """Report conversation engagement back onto Harvester's OutreachOutcome."""
        payload = {
            "institution_id": institution_id,
            "draft_id": draft_id,
            "opened": opened,
            "clicked": clicked,
            "replied": replied,
            "positive_reply": positive_reply,
            "reply_text": reply_text,
        }
        async with httpx.AsyncClient(base_url=self.base_url, headers=self._headers(), timeout=30.0) as client:
            response = await client.post("/outreach-outcomes", json=payload)
            response.raise_for_status()
            return response.json()
