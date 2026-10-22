from fastapi import FastAPI
from fastapi.responses import RedirectResponse

from communicator.api.routes import (
    conversations,
    drafts,
    live_proof,
    live_proof_ui,
    scheduler,
    sequences,
    services,
    webhooks,
)

app = FastAPI(
    title="Communicator API",
    description="Multi-channel outreach, customer AI live support, and Verilo-style LiveProof return verification.",
    version="0.2.0",
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

# LiveProof verification portal and endpoints (WO-Verilo live camera return verification)
app.include_router(live_proof.router, prefix="/api/v1")
app.include_router(live_proof_ui.router)
