from fastapi import FastAPI
from fastapi.responses import RedirectResponse

from communicator.api.routes import (
    automation,
    broadcasts,
    contacts,
    conversations,
    drafts,
    knowledge_base,
    scheduler,
    sequences,
    services,
    settings,
    team,
    webhooks,
)

app = FastAPI(
    title="Communicator API",
    description="Multi-channel outreach and conversation engine for NEWAX leads.",
    version="0.1.0",
)


@app.get("/", include_in_schema=False)
async def root():
    return RedirectResponse(url="/docs")


@app.get("/api/v1/health")
async def health_check():
    return {"status": "ok"}


# Webhooks authenticate themselves (Twilio signature / Postmark shared
# secret) rather than via user bearer auth, so they're not behind any
# get_current_user-style dependency.
app.include_router(webhooks.router, prefix="/api/v1")
app.include_router(drafts.router, prefix="/api/v1")
app.include_router(services.router, prefix="/api/v1")
app.include_router(conversations.router, prefix="/api/v1")
app.include_router(scheduler.router, prefix="/api/v1")
app.include_router(sequences.router, prefix="/api/v1")
app.include_router(settings.router, prefix="/api/v1")
app.include_router(knowledge_base.router, prefix="/api/v1")
app.include_router(team.router, prefix="/api/v1")
app.include_router(automation.router, prefix="/api/v1")
app.include_router(contacts.router, prefix="/api/v1")
app.include_router(broadcasts.router, prefix="/api/v1")
