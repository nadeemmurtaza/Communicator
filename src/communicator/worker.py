"""
Standalone process that periodically advances due follow-up sequence
steps -- see docs/ARCHITECTURE.md section 8. The scheduler infra
(Celery/APScheduler + Redis vs. a plain poll loop) is flagged as an open
decision there; this is the minimal version that needs no new
infrastructure, and can be swapped for Celery/APScheduler later without
touching communicator.services.followup_scheduler.

Run with: python -m communicator.worker
"""

from __future__ import annotations

import asyncio
import logging

from communicator.config import settings
from communicator.database import get_session
from communicator.services.followup_scheduler import run_due_followups

logger = logging.getLogger(__name__)


async def _tick() -> None:
    async with get_session() as session:
        drafts = await run_due_followups(session)
        if drafts:
            logger.info("follow-up scheduler: generated %d draft(s)", len(drafts))


async def run_forever(interval_seconds: int | None = None) -> None:
    interval = interval_seconds or settings.followup_poll_interval_seconds
    while True:
        try:
            await _tick()
        except Exception:
            logger.exception("follow-up scheduler tick failed")
        await asyncio.sleep(interval)


def main() -> None:
    logging.basicConfig(level=logging.INFO)
    asyncio.run(run_forever())


if __name__ == "__main__":
    main()
