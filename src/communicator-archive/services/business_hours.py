from __future__ import annotations

from datetime import datetime
from zoneinfo import ZoneInfo

from communicator.models.integration_settings import IntegrationSettings


def is_within_business_hours(
    settings_row: IntegrationSettings, now: datetime | None = None
) -> bool:
    """True if business-hours gating is off, or the current time (in the
    configured timezone) falls within [start, end)."""
    if not settings_row.business_hours_enabled:
        return True

    try:
        tz = ZoneInfo(settings_row.business_timezone or "UTC")
    except Exception:  # noqa: BLE001 -- bad/unknown tz name from the Connections form
        tz = ZoneInfo("UTC")

    local_now = (now or datetime.now(tz)).astimezone(tz)
    return settings_row.business_hours_start <= local_now.hour < settings_row.business_hours_end
