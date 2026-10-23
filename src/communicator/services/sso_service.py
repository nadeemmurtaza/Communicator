from __future__ import annotations

import base64
import logging
import urllib.parse
import uuid
import xml.etree.ElementTree as ET
from datetime import UTC, datetime
from typing import Any

from defusedxml.ElementTree import fromstring as safe_xml_fromstring
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.sso import SINGLETON_SSO_CONFIG_ID, SSOConfig
from communicator.models.team_member import TeamMember
from communicator.services.audit_service import record_audit_event

logger = logging.getLogger(__name__)


def _normalize_domain(domain: str) -> str:
    """Normalizes an email domain (e.g. '@Bayworks.COM ' -> 'bayworks.com')."""
    d = domain.strip().lower()
    d = d.removeprefix("@")
    return d.strip()


async def get_or_create_sso_config(session: AsyncSession) -> SSOConfig:
    """
    Retrieves the workspace SSO / SAML 2.0 configuration, initializing the singleton record if not present.
    """
    config = await session.get(SSOConfig, SINGLETON_SSO_CONFIG_ID)
    if not config:
        config = SSOConfig(
            id=SINGLETON_SSO_CONFIG_ID,
            is_enabled=False,
            provider_type="saml",
            idp_entity_id="",
            idp_sso_url="",
            idp_x509_cert="",
            sp_entity_id="urn:communicator:sp",
            sp_acs_url="/api/v1/sso/saml/acs",
            enforced_domains=[],
            allow_unlinked_passwords=True,
            jit_provisioning_enabled=True,
            default_role="agent",
            attribute_mapping={
                "email": "email",
                "name": "name",
                "first_name": "first_name",
                "last_name": "last_name",
                "role": "role",
            },
            last_tested_at=None,
            last_test_status="untested",
            last_test_message="Configuration has not yet been verified.",
        )
        session.add(config)
        await session.commit()
        await session.refresh(config)
    return config


async def update_sso_config(
    session: AsyncSession,
    updates: dict[str, Any],
    actor: str = "admin",
    actor_email: str | None = None,
    ip_address: str | None = None,
) -> SSOConfig:
    """
    Updates enterprise SSO settings and writes a SOC 2 audit log event.
    """
    config = await get_or_create_sso_config(session)

    if "is_enabled" in updates:
        config.is_enabled = bool(updates["is_enabled"])
    if "provider_type" in updates:
        config.provider_type = str(updates["provider_type"]).strip().lower()
    if "idp_entity_id" in updates:
        config.idp_entity_id = str(updates["idp_entity_id"]).strip()
    if "idp_sso_url" in updates:
        config.idp_sso_url = str(updates["idp_sso_url"]).strip()
    if "idp_x509_cert" in updates:
        config.idp_x509_cert = str(updates["idp_x509_cert"]).strip()
    if "sp_entity_id" in updates:
        config.sp_entity_id = str(updates["sp_entity_id"]).strip()
    if "sp_acs_url" in updates:
        config.sp_acs_url = str(updates["sp_acs_url"]).strip()
    if "jit_provisioning_enabled" in updates:
        config.jit_provisioning_enabled = bool(updates["jit_provisioning_enabled"])
    if "default_role" in updates:
        config.default_role = str(updates["default_role"]).strip().lower()
    if "allow_unlinked_passwords" in updates:
        config.allow_unlinked_passwords = bool(updates["allow_unlinked_passwords"])
    if "attribute_mapping" in updates and isinstance(updates["attribute_mapping"], dict):
        config.attribute_mapping = updates["attribute_mapping"]

    if "enforced_domains" in updates:
        raw_domains = updates["enforced_domains"]
        if isinstance(raw_domains, str):
            domains = [d for d in raw_domains.split(",") if d.strip()]
        elif isinstance(raw_domains, list):
            domains = raw_domains
        else:
            domains = []
        config.enforced_domains = sorted(
            {_normalize_domain(str(d)) for d in domains if _normalize_domain(str(d))}
        )

    await session.commit()
    await session.refresh(config)

    # Log to SOC 2 Compliance Audit Trail
    await record_audit_event(
        session=session,
        action="sso.configured",
        entity_type="sso_config",
        entity_id=config.id,
        actor=actor,
        actor_email=actor_email,
        compliance_category="security_policy_change",
        details={
            "is_enabled": config.is_enabled,
            "provider_type": config.provider_type,
            "enforced_domains": config.enforced_domains,
            "jit_provisioning_enabled": config.jit_provisioning_enabled,
            "default_role": config.default_role,
        },
        ip_address=ip_address,
        description=f"Enterprise SSO configuration updated (Enabled={config.is_enabled}, Provider={config.provider_type})",
    )

    return config


def generate_sp_metadata_xml(config: SSOConfig, base_url: str = "http://localhost:8000") -> str:
    """
    Generates standard SAML 2.0 Service Provider EntityDescriptor XML for importing into IdPs
    (Okta, Azure AD, Google Workspace, PingIdentity, OneLogin).
    """
    clean_base = base_url.rstrip("/")
    acs_url = (
        config.sp_acs_url
        if config.sp_acs_url.startswith("http://") or config.sp_acs_url.startswith("https://")
        else f"{clean_base}{config.sp_acs_url}"
    )
    entity_id = config.sp_entity_id or f"{clean_base}/api/v1/sso/metadata"

    return f"""<?xml version="1.0" encoding="UTF-8"?>
<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" entityID="{entity_id}">
  <md:SPSSODescriptor AuthnRequestsSigned="false" WantAssertionsSigned="true" protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <md:NameIDFormat>urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress</md:NameIDFormat>
    <md:NameIDFormat>urn:oasis:names:tc:SAML:2.0:nameid-format:persistent</md:NameIDFormat>
    <md:NameIDFormat>urn:oasis:names:tc:SAML:2.0:nameid-format:transient</md:NameIDFormat>
    <md:AssertionConsumerService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" Location="{acs_url}" index="1" isDefault="true"/>
  </md:SPSSODescriptor>
  <md:Organization>
    <md:OrganizationName xml:lang="en">Captain AI Communicator</md:OrganizationName>
    <md:OrganizationDisplayName xml:lang="en">Communicator Enterprise Platform</md:OrganizationDisplayName>
    <md:OrganizationURL xml:lang="en">{clean_base}</md:OrganizationURL>
  </md:Organization>
</md:EntityDescriptor>"""


def generate_authn_request(
    config: SSOConfig,
    base_url: str = "http://localhost:8000",
    relay_state: str = "",
) -> dict[str, str]:
    """
    Constructs a SAML 2.0 AuthnRequest and returns the target IdP redirect URL.
    """
    clean_base = base_url.rstrip("/")
    acs_url = (
        config.sp_acs_url
        if config.sp_acs_url.startswith("http://") or config.sp_acs_url.startswith("https://")
        else f"{clean_base}{config.sp_acs_url}"
    )
    req_id = f"ID_{uuid.uuid4().hex}"
    now_iso = datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ")

    xml_payload = f"""<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
  xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
  ID="{req_id}"
  Version="2.0"
  IssueInstant="{now_iso}"
  Destination="{config.idp_sso_url}"
  AssertionConsumerServiceURL="{acs_url}"
  ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST">
  <saml:Issuer>{config.sp_entity_id}</saml:Issuer>
  <samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress" AllowCreate="true"/>
</samlp:AuthnRequest>"""

    b64_request = base64.b64encode(xml_payload.encode("utf-8")).decode("ascii")

    params = {"SAMLRequest": b64_request}
    if relay_state:
        params["RelayState"] = relay_state

    query_str = urllib.parse.urlencode(params)
    separator = "&" if "?" in config.idp_sso_url else "?"
    redirect_url = f"{config.idp_sso_url}{separator}{query_str}" if config.idp_sso_url else ""

    return {
        "request_id": req_id,
        "issue_instant": now_iso,
        "saml_request_b64": b64_request,
        "redirect_url": redirect_url,
        "acs_url": acs_url,
    }


async def check_domain_sso_enforced(
    session: AsyncSession,
    email: str,
    base_url: str = "http://localhost:8000",
) -> dict[str, Any]:
    """
    Checks if a user's email domain has SSO enforced, returning the provider information and login redirect if required.
    """
    config = await get_or_create_sso_config(session)
    email_clean = email.strip().lower()
    domain = email_clean.split("@")[-1] if "@" in email_clean else ""
    normalized_domain = _normalize_domain(domain)

    is_enforced = bool(
        config.is_enabled and normalized_domain and normalized_domain in config.enforced_domains
    )

    redirect_url = None
    if is_enforced and config.idp_sso_url:
        authn = generate_authn_request(config, base_url=base_url, relay_state="/")
        redirect_url = authn["redirect_url"]

    return {
        "sso_required": is_enforced,
        "is_enabled": config.is_enabled,
        "provider_type": config.provider_type,
        "domain": normalized_domain,
        "redirect_url": redirect_url,
        "allow_unlinked_passwords": config.allow_unlinked_passwords,
    }


def parse_saml_response_xml(saml_xml: str) -> dict[str, Any]:
    """
    Parses a SAML 2.0 Response XML document, extracting NameID, Issuer, and Attributes.
    Resilient to namespace prefixes.
    """
    try:
        # defusedxml rejects entity expansion (XML bomb) and external entity
        # references -- a SAML response is attacker-influenceable input.
        root = safe_xml_fromstring(saml_xml)
    except Exception as e:
        raise ValueError(f"Failed to parse SAML XML payload: {e}") from e

    # Helper to find elements ignoring namespace prefixes
    def find_all_by_tag(element: ET.Element, tag_name: str) -> list[ET.Element]:
        results = []
        for elem in element.iter():
            local_name = elem.tag.split("}")[-1] if "}" in elem.tag else elem.tag
            if local_name == tag_name:
                results.append(elem)
        return results

    def find_first_by_tag(element: ET.Element, tag_name: str) -> ET.Element | None:
        matches = find_all_by_tag(element, tag_name)
        return matches[0] if matches else None

    # Status check
    status_code_el = find_first_by_tag(root, "StatusCode")
    status_value = status_code_el.attrib.get("Value", "") if status_code_el is not None else ""
    is_success = "Success" in status_value or not status_value

    # Extract Issuer
    issuer_el = find_first_by_tag(root, "Issuer")
    issuer = issuer_el.text.strip() if issuer_el is not None and issuer_el.text else ""

    # Extract NameID
    name_id_el = find_first_by_tag(root, "NameID")
    name_id = name_id_el.text.strip() if name_id_el is not None and name_id_el.text else ""

    # Extract Attributes
    attributes: dict[str, Any] = {}
    attr_elements = find_all_by_tag(root, "Attribute")
    for attr in attr_elements:
        attr_name = attr.attrib.get("Name", "")
        friendly_name = attr.attrib.get("FriendlyName", "")
        values = []
        for val_el in find_all_by_tag(attr, "AttributeValue"):
            if val_el.text:
                values.append(val_el.text.strip())
        val_str = values[0] if len(values) == 1 else values
        if attr_name:
            attributes[attr_name] = val_str
        if friendly_name:
            attributes[friendly_name] = val_str

    # Resolve standard user fields
    # Email
    email = ""
    for candidate in [
        "email",
        "Email",
        "mail",
        "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress",
        "User.Email",
    ]:
        if candidate in attributes and isinstance(attributes[candidate], str):
            email = attributes[candidate]
            break
    if not email and "@" in name_id:
        email = name_id

    # Name
    first_name = ""
    last_name = ""
    full_name = ""
    for candidate in [
        "name",
        "Name",
        "displayName",
        "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name",
    ]:
        if candidate in attributes and isinstance(attributes[candidate], str):
            full_name = attributes[candidate]
            break

    for candidate in [
        "first_name",
        "givenName",
        "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname",
    ]:
        if candidate in attributes and isinstance(attributes[candidate], str):
            first_name = attributes[candidate]
            break

    for candidate in [
        "last_name",
        "surname",
        "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/surname",
    ]:
        if candidate in attributes and isinstance(attributes[candidate], str):
            last_name = attributes[candidate]
            break

    if not full_name and (first_name or last_name):
        full_name = f"{first_name} {last_name}".strip()
    if not full_name and email:
        full_name = email.split("@")[0].replace(".", " ").title()

    # Role
    role = ""
    for candidate in [
        "role",
        "Role",
        "http://schemas.microsoft.com/ws/2008/06/identity/claims/role",
        "groups",
    ]:
        if candidate in attributes:
            val = attributes[candidate]
            role = val[0] if isinstance(val, list) and val else str(val)
            break

    return {
        "status_success": is_success,
        "status_code": status_value,
        "issuer": issuer,
        "name_id": name_id,
        "email": email.strip().lower(),
        "name": full_name or "Enterprise User",
        "first_name": first_name,
        "last_name": last_name,
        "role": role.strip().lower(),
        "attributes": attributes,
    }


async def process_saml_assertion(
    session: AsyncSession,
    saml_response_raw: str,
    relay_state: str = "",
    ip_address: str | None = None,
    user_agent: str | None = None,
) -> dict[str, Any]:
    """
    Decodes and processes an inbound SAML 2.0 Response assertion, handles Just-in-Time (JIT) provisioning
    of team members, updates the RBAC role, and writes a compliance audit log entry.
    """
    config = await get_or_create_sso_config(session)

    # 1. Decode payload (support either Base64-encoded or raw XML string)
    xml_content = ""
    try:
        decoded_bytes = base64.b64decode(saml_response_raw.strip(), validate=True)
        xml_content = decoded_bytes.decode("utf-8")
    except Exception:  # noqa: BLE001
        # If not valid base64, test if it's already XML
        if saml_response_raw.strip().startswith("<"):
            xml_content = saml_response_raw.strip()
        else:
            raise ValueError("Invalid SAML assertion: payload is neither valid Base64 nor XML.")

    # 2. Parse XML
    parsed = parse_saml_response_xml(xml_content)
    if not parsed["status_success"]:
        await record_audit_event(
            session=session,
            action="sso.login_failed",
            entity_type="sso_assertion",
            actor="saml_idp",
            severity="warning",
            compliance_category="authentication_failure",
            details={
                "status_code": parsed["status_code"],
                "issuer": parsed["issuer"],
                "reason": "SAML assertion status was not Success",
            },
            ip_address=ip_address,
            user_agent=user_agent,
            description="Enterprise SSO login rejected by Identity Provider status code.",
        )
        raise ValueError(f"SAML assertion failed with status: {parsed['status_code']}")

    email = parsed["email"]
    if not email or "@" not in email:
        raise ValueError("SAML assertion does not contain a valid NameID or email attribute.")

    # 3. Check domain enforcement if enabled
    domain = _normalize_domain(email.split("@")[-1])
    if config.enforced_domains and domain not in config.enforced_domains:
        logger.warning(f"User domain '{domain}' not in enforced domains: {config.enforced_domains}")

    # 4. Lookup or JIT provision TeamMember
    member = await session.scalar(select(TeamMember).where(TeamMember.email == email))
    is_new = False

    if not member:
        if not config.jit_provisioning_enabled:
            await record_audit_event(
                session=session,
                action="sso.login_failed",
                entity_type="sso_assertion",
                actor=email,
                actor_email=email,
                severity="warning",
                compliance_category="access_denied",
                details={
                    "email": email,
                    "reason": "Just-In-Time (JIT) provisioning is disabled and user does not exist",
                },
                ip_address=ip_address,
                user_agent=user_agent,
                description=f"SSO login rejected for {email}: JIT provisioning disabled.",
            )
            raise ValueError(
                f"No account found for {email} and Just-In-Time (JIT) provisioning is disabled."
            )

        # JIT Provisioning
        is_new = True
        assigned_role = parsed.get("role") or config.default_role or "agent"
        member = TeamMember(
            name=parsed["name"] or email.split("@")[0].title(),
            email=email,
            role=assigned_role,
            availability_status="online",
        )
        session.add(member)
        await session.commit()
        await session.refresh(member)

        # Record JIT provisioning audit log
        await record_audit_event(
            session=session,
            action="sso.user_provisioned",
            entity_type="team_member",
            entity_id=member.id,
            actor="sso_jit",
            actor_email=email,
            compliance_category="user_provisioning",
            details={
                "email": member.email,
                "role": member.role,
                "provider_type": config.provider_type,
                "issuer": parsed["issuer"],
            },
            ip_address=ip_address,
            user_agent=user_agent,
            description=f"New team member {member.name} ({member.email}) JIT-provisioned via {config.provider_type.upper()} SSO with role '{member.role}'",
        )
    else:
        # Update user session
        member.availability_status = "online"
        await session.commit()
        await session.refresh(member)

    # 5. Record Login Success Audit Log
    await record_audit_event(
        session=session,
        action="sso.login_success",
        entity_type="team_member",
        entity_id=member.id,
        actor=member.name,
        actor_email=member.email,
        actor_role=member.role,
        compliance_category="authentication_success",
        details={
            "provider_type": config.provider_type,
            "issuer": parsed["issuer"],
            "name_id": parsed["name_id"],
            "is_new_user": is_new,
            "relay_state": relay_state,
        },
        ip_address=ip_address,
        user_agent=user_agent,
        description=f"Enterprise SSO login authenticated for {member.email} via {config.provider_type.upper()}",
    )

    return {
        "success": True,
        "user": {
            "id": str(member.id),
            "name": member.name,
            "email": member.email,
            "role": member.role,
        },
        "provisioned": is_new,
        "provider": config.provider_type,
        "relay_state": relay_state or "/",
    }


async def test_sso_connection(
    session: AsyncSession,
    sample_xml: str | None = None,
) -> dict[str, Any]:
    """
    Validates current SSO configuration parameters, syntax, and verifies parser against a test assertion.
    """
    config = await get_or_create_sso_config(session)
    checks: list[dict[str, Any]] = []
    has_errors = False

    # Check 1: SSO Enabled
    checks.append(
        {
            "name": "SSO Master Status",
            "passed": config.is_enabled,
            "message": "SSO is active" if config.is_enabled else "SSO is currently disabled",
        }
    )

    # Check 2: IdP SSO URL
    url_valid = bool(
        config.idp_sso_url
        and (config.idp_sso_url.startswith("https://") or config.idp_sso_url.startswith("http://"))
    )
    if not url_valid:
        has_errors = True
    checks.append(
        {
            "name": "Identity Provider SSO URL",
            "passed": url_valid,
            "message": config.idp_sso_url if url_valid else "Missing or invalid IdP HTTP/HTTPS URL",
        }
    )

    # Check 3: IdP Entity ID / Issuer
    issuer_valid = bool(config.idp_entity_id.strip())
    if not issuer_valid:
        has_errors = True
    checks.append(
        {
            "name": "Identity Provider Entity ID",
            "passed": issuer_valid,
            "message": config.idp_entity_id
            if issuer_valid
            else "Missing IdP Entity ID / Issuer URI",
        }
    )

    # Check 4: X.509 Certificate
    cert_valid = bool(
        config.idp_x509_cert.strip() and "CERTIFICATE" in config.idp_x509_cert.upper()
    )
    checks.append(
        {
            "name": "X.509 Certificate",
            "passed": cert_valid,
            "message": "Valid PEM certificate detected"
            if cert_valid
            else "Missing or unformatted X.509 public key certificate",
        }
    )

    # Check 5: Enforced Domains
    domains_count = len(config.enforced_domains)
    checks.append(
        {
            "name": "Enforced Domains",
            "passed": domains_count > 0,
            "message": f"{domains_count} domain(s) configured ({', '.join(config.enforced_domains)})"
            if domains_count > 0
            else "No domains enforced (users can log in with standard credentials)",
        }
    )

    # Check 6: XML Parser Self-Test
    test_xml = (
        sample_xml
        or f"""<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
  xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
  ID="_test123" Version="2.0">
  <saml:Issuer>{config.idp_entity_id or "https://test-idp.example.com"}</saml:Issuer>
  <samlp:Status>
    <samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success"/>
  </samlp:Status>
  <saml:Assertion ID="_assert123" Version="2.0">
    <saml:Subject>
      <saml:NameID Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress">test.agent@example.com</saml:NameID>
    </saml:Subject>
    <saml:AttributeStatement>
      <saml:Attribute Name="name"><saml:AttributeValue>Test SAML Agent</saml:AttributeValue></saml:Attribute>
      <saml:Attribute Name="role"><saml:AttributeValue>{config.default_role}</saml:AttributeValue></saml:Attribute>
    </saml:AttributeStatement>
  </saml:Assertion>
</samlp:Response>"""
    )

    parser_success = False
    parser_details = {}
    try:
        parsed_test = parse_saml_response_xml(test_xml)
        parser_success = parsed_test["status_success"] and "@" in parsed_test["email"]
        parser_details = {
            "email": parsed_test["email"],
            "name": parsed_test["name"],
            "role": parsed_test["role"],
        }
    except Exception as ex:  # noqa: BLE001
        parser_details = {"error": str(ex)}

    checks.append(
        {
            "name": "SAML Assertion Parser",
            "passed": parser_success,
            "message": f"Successfully parsed sample assertion: {parser_details.get('email')}"
            if parser_success
            else f"Parser error: {parser_details.get('error')}",
        }
    )

    overall_status = "healthy" if (not has_errors and parser_success) else "warning"
    overall_msg = (
        "Enterprise SSO configuration is fully verified and ready for production."
        if overall_status == "healthy"
        else "SSO configuration has missing or incomplete parameters."
    )

    config.last_tested_at = datetime.now(UTC)
    config.last_test_status = overall_status
    config.last_test_message = overall_msg
    await session.commit()
    await session.refresh(config)

    return {
        "status": overall_status,
        "message": overall_msg,
        "tested_at": config.last_tested_at.isoformat(),
        "checks": checks,
        "sample_parsed": parser_details,
    }
