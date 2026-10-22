from __future__ import annotations

from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Request, Response
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.services import sso_service

router = APIRouter()


class SSOConfigUpdatePayload(BaseModel):
    is_enabled: bool | None = None
    provider_type: str | None = None
    idp_entity_id: str | None = None
    idp_sso_url: str | None = None
    idp_x509_cert: str | None = None
    sp_entity_id: str | None = None
    sp_acs_url: str | None = None
    enforced_domains: list[str] | str | None = None
    jit_provisioning_enabled: bool | None = None
    default_role: str | None = None
    allow_unlinked_passwords: bool | None = None
    attribute_mapping: dict[str, str] | None = None


class CheckEmailPayload(BaseModel):
    email: str = Field(..., description="User email to check for SSO domain enforcement")


class SAMLAssertionJSONPayload(BaseModel):
    saml_response: str = Field(..., description="Base64 encoded or raw XML SAMLResponse")
    relay_state: str | None = Field(default="", description="Target application redirect route")


class SSOTestPayload(BaseModel):
    sample_xml: str | None = None


def _format_config(config: Any) -> dict[str, Any]:
    return {
        "id": str(config.id),
        "is_enabled": config.is_enabled,
        "provider_type": config.provider_type,
        "idp_entity_id": config.idp_entity_id,
        "idp_sso_url": config.idp_sso_url,
        "idp_x509_cert": config.idp_x509_cert,
        "sp_entity_id": config.sp_entity_id,
        "sp_acs_url": config.sp_acs_url,
        "enforced_domains": config.enforced_domains,
        "jit_provisioning_enabled": config.jit_provisioning_enabled,
        "default_role": config.default_role,
        "allow_unlinked_passwords": config.allow_unlinked_passwords,
        "attribute_mapping": config.attribute_mapping,
        "last_tested_at": config.last_tested_at.isoformat() if config.last_tested_at else None,
        "last_test_status": config.last_test_status,
        "last_test_message": config.last_test_message,
        "created_at": config.created_at.isoformat(),
        "updated_at": config.updated_at.isoformat(),
    }


@router.get("/config")
async def get_sso_config(db: AsyncSession = Depends(get_db_session)):
    """
    Returns the workspace Enterprise SSO & SAML 2.0 configuration.
    """
    config = await sso_service.get_or_create_sso_config(db)
    return {"sso_config": _format_config(config)}


@router.put("/config")
async def update_sso_config(
    payload: SSOConfigUpdatePayload,
    request: Request,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Updates enterprise SSO configuration and logs to the compliance trail.
    """
    updates = payload.model_dump(exclude_unset=True)
    client_ip = request.client.host if request.client else "127.0.0.1"

    updated = await sso_service.update_sso_config(
        session=db,
        updates=updates,
        actor="admin",
        actor_email="admin@bayworks.com",
        ip_address=client_ip,
    )
    return {"sso_config": _format_config(updated)}


@router.get("/metadata.xml")
@router.get("/metadata")
async def get_sp_metadata(
    request: Request,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Returns the standard SAML 2.0 Service Provider Metadata XML document for IdP registration.
    """
    config = await sso_service.get_or_create_sso_config(db)
    base_url = str(request.base_url).rstrip("/")
    xml_content = sso_service.generate_sp_metadata_xml(config, base_url=base_url)
    return Response(content=xml_content, media_type="application/xml")


@router.post("/check")
async def check_email_sso(
    payload: CheckEmailPayload,
    request: Request,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Checks if an email's domain enforces Single Sign-On and returns the IdP redirect URL.
    """
    base_url = str(request.base_url).rstrip("/")
    result = await sso_service.check_domain_sso_enforced(db, payload.email, base_url=base_url)
    return result


@router.get("/login")
async def initiate_sso_login(
    email: str = "",
    relay_state: str = "/",
    request: Request = None,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Generates an AuthnRequest redirect for the Identity Provider.
    """
    config = await sso_service.get_or_create_sso_config(db)
    if not config.is_enabled or not config.idp_sso_url:
        raise HTTPException(
            status_code=400,
            detail="Enterprise Single Sign-On is not configured or enabled.",
        )

    base_url = str(request.base_url).rstrip("/") if request else "http://localhost:8000"
    authn = sso_service.generate_authn_request(config, base_url=base_url, relay_state=relay_state)
    return authn


@router.post("/saml/acs")
async def assertion_consumer_service(
    request: Request,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Assertion Consumer Service (ACS) endpoint. Accepts standard HTTP-POST from IdP
    (via form body or JSON payload), validates assertion, provisions user, and returns session.
    """
    content_type = request.headers.get("content-type", "")
    saml_response = ""
    relay_state = "/"

    if "application/json" in content_type:
        try:
            body = await request.json()
            saml_response = body.get("saml_response", "") or body.get("SAMLResponse", "")
            relay_state = body.get("relay_state", "") or body.get("RelayState", "/")
        except Exception:  # noqa: BLE001, S110
            pass
    else:
        form_data = await request.form()
        saml_response = form_data.get("SAMLResponse", "")
        relay_state = form_data.get("RelayState", "/")

    if not saml_response:
        raise HTTPException(
            status_code=400,
            detail="Missing SAMLResponse in request body.",
        )

    client_ip = request.client.host if request.client else "127.0.0.1"
    user_agent = request.headers.get("user-agent", "")

    try:
        auth_result = await sso_service.process_saml_assertion(
            session=db,
            saml_response_raw=str(saml_response),
            relay_state=str(relay_state),
            ip_address=client_ip,
            user_agent=user_agent,
        )
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

    # If it's a browser form POST from the IdP, we can return an HTML script that redirects or stores the session
    if "application/json" not in content_type:
        redirect_target = auth_result.get("relay_state") or "/"
        html_content = f"""<!DOCTYPE html>
<html>
<head><title>Authenticating with Captain AI Enterprise...</title></head>
<body>
<p>Authentication successful. Redirecting...</p>
<script>
  window.sessionStorage.setItem('sso_user', JSON.stringify({auth_result["user"]}));
  window.location.href = '{redirect_target}';
</script>
</body>
</html>"""
        return Response(content=html_content, media_type="text/html")

    return auth_result


@router.post("/test")
async def test_sso_endpoint(
    payload: SSOTestPayload | None = None,
    db: AsyncSession = Depends(get_db_session),
):
    """
    Runs diagnostic and connection test against current SSO configuration.
    """
    sample_xml = payload.sample_xml if payload else None
    result = await sso_service.test_sso_connection(db, sample_xml=sample_xml)
    return result
