from fastapi import FastAPI
from fastapi.responses import RedirectResponse

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
