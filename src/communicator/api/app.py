from pathlib import Path

from fastapi import FastAPI, HTTPException
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles

from communicator.api.routes import (
    ai_connections,
    audit_logs,
    automation,
    broadcasts,
    canned_responses,
    captain_ai,
    contacts,
    conversations,
    copilot_approvals,
    custom_attributes,
    discovery,
    drafts,
    gateways,
    inboxes,
    instagram_auth,
    issue_trackers,
    knowledge_base,
    macros,
    nid_auth,
    organizations,
    outgoing_webhooks,
    prechat,
    presence,
    reports,
    roles,
    routing,
    scheduler,
    segments,
    self_improving,
    sequences,
    services,
    settings,
    simulations,
    sla,
    sso,
    stripe_integration,
    team,
    teams,
    tickets,
    translation,
    webhooks,
    widget,
    widget_customizer,
)

app = FastAPI(
    title="Communicator API",
    description="Multi-channel outreach and conversation engine for NEWAX leads.",
    version="0.1.0",
)


@app.get("/api/v1/health")
async def health_check():
    return {"status": "ok"}


# Webhooks authenticate themselves (Twilio signature / Postmark shared secret)
app.include_router(webhooks.router, prefix="/api/v1")
app.include_router(instagram_auth.router, prefix="/api/v1")
app.include_router(nid_auth.router, prefix="/api/v1")
app.include_router(organizations.router, prefix="/api/v1")
app.include_router(widget.router, prefix="/api/v1")
app.include_router(widget_customizer.router, prefix="/api/v1")
app.include_router(prechat.router, prefix="/api/v1")
app.include_router(audit_logs.router, prefix="/api/v1")
app.include_router(roles.router, prefix="/api/v1")
app.include_router(sso.router, prefix="/api/v1/sso", tags=["sso"])
app.include_router(translation.router, prefix="/api/v1")
app.include_router(custom_attributes.router, prefix="/api/v1")
app.include_router(discovery.router, prefix="/api/v1")
app.include_router(inboxes.router, prefix="/api/v1")
app.include_router(drafts.router, prefix="/api/v1")
app.include_router(services.router, prefix="/api/v1")
app.include_router(conversations.router, prefix="/api/v1")
app.include_router(canned_responses.router, prefix="/api/v1")
app.include_router(teams.router, prefix="/api/v1")
app.include_router(scheduler.router, prefix="/api/v1")
app.include_router(sequences.router, prefix="/api/v1")
app.include_router(settings.router, prefix="/api/v1")
app.include_router(knowledge_base.router, prefix="/api/v1")
app.include_router(team.router, prefix="/api/v1")
app.include_router(automation.router, prefix="/api/v1")
app.include_router(contacts.router, prefix="/api/v1")
app.include_router(broadcasts.router, prefix="/api/v1")
app.include_router(reports.router, prefix="/api/v1")
app.include_router(captain_ai.router, prefix="/api/v1")
app.include_router(presence.router, prefix="/api/v1")
app.include_router(macros.router, prefix="/api/v1")
app.include_router(sla.router, prefix="/api/v1")
app.include_router(routing.router, prefix="/api/v1")
app.include_router(segments.router, prefix="/api/v1")
app.include_router(copilot_approvals.router, prefix="/api/v1")
app.include_router(tickets.router, prefix="/api/v1")
app.include_router(simulations.router, prefix="/api/v1")
app.include_router(self_improving.router, prefix="/api/v1")
app.include_router(ai_connections.router, prefix="/api/v1")
app.include_router(gateways.router, prefix="/api/v1")
app.include_router(outgoing_webhooks.router, prefix="/api/v1")
app.include_router(stripe_integration.router, prefix="/api/v1")
app.include_router(issue_trackers.router, prefix="/api/v1")

# Built frontend (frontend/dist, from `npm run build`) -- served from the
# same origin as the API so the Instagram OAuth redirect_uri, webhook URLs
# etc. all point at one domain in production instead of needing a separate
# static host. Mounted/registered last so it never shadows an /api/v1/* route.
FRONTEND_DIST = Path(__file__).resolve().parents[3] / "frontend" / "dist"
if (FRONTEND_DIST / "assets").is_dir():
    app.mount("/assets", StaticFiles(directory=FRONTEND_DIST / "assets"), name="frontend-assets")


@app.get("/{full_path:path}", include_in_schema=False)
async def spa(full_path: str):
    """SPA fallback: any non-API, non-static path serves index.html and lets
    react-router handle the route client-side.

    index.html references the current build's hashed asset filenames, so it
    must never be cached (by the browser or by Cloudflare's edge) -- a
    cached index.html keeps pointing at a stale JS/CSS bundle after every
    deploy until the cache expires. The hashed files under /assets are safe
    to cache indefinitely since a new build always gets new filenames.
    """
    index = FRONTEND_DIST / "index.html"
    if not index.is_file():
        raise HTTPException(
            status_code=404, detail="Frontend build not found. Run `npm run build`."
        )
    return FileResponse(index, headers={"Cache-Control": "no-store, must-revalidate"})
