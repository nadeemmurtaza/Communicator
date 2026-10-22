from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.responses import RedirectResponse

from communicator.vault_loader import load_secrets_from_nid_vault


@asynccontextmanager
async def lifespan(app: FastAPI):
    # Pre-bootstrap: resolve dynamic secrets from nID Vault
    await load_secrets_from_nid_vault()
    yield

from communicator.api.routes import (
    ai_connections,
    audit_logs,
    automation,
    broadcasts,
    canned_responses,
    captain_ai,
    contacts,
    conversations,
    crypto_mesh,
    nid_auth,
    copilot_approvals,
    custom_attributes,
    drafts,
    gateways,
    inboxes,
    issue_trackers,
    knowledge_base,
    live_proof,
    live_proof_ui,
    macros,
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
    lifespan=lifespan,
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


# Webhooks authenticate themselves (Twilio signature / Postmark shared secret)
app.include_router(webhooks.router, prefix="/api/v1")
app.include_router(widget.router, prefix="/api/v1")
app.include_router(widget_customizer.router, prefix="/api/v1")
app.include_router(prechat.router, prefix="/api/v1")
app.include_router(audit_logs.router, prefix="/api/v1")
app.include_router(roles.router, prefix="/api/v1")
app.include_router(sso.router, prefix="/api/v1/sso", tags=["sso"])
app.include_router(translation.router, prefix="/api/v1")
app.include_router(custom_attributes.router, prefix="/api/v1")
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

# LiveProof verification portal and endpoints (WO-Verilo live camera return verification)
app.include_router(live_proof.router, prefix="/api/v1")
app.include_router(live_proof_ui.router)

# Zero-Trust Crypto Mesh Ingress & nID Identity Auth
app.include_router(crypto_mesh.router, prefix="/api/v1")
app.include_router(nid_auth.router, prefix="/api/v1")
