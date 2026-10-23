from __future__ import annotations

import io
import ipaddress
import re
import socket
from urllib.parse import urlparse

import httpx
from bs4 import BeautifulSoup
from pypdf import PdfReader
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.knowledge_base import KnowledgeBaseEntry


class IngestionError(Exception):
    """Raised when URL or document ingestion fails."""


def _clean_text(raw: str) -> str:
    """Collapses excess blank lines, escapes, and whitespace."""
    raw = re.sub(r"[\r\t\f\v ]+", " ", raw)
    lines = [line.strip() for line in raw.splitlines()]
    non_empty = [line for line in lines if line]
    return "\n\n".join(non_empty)


def _resolve_public_ip(hostname: str) -> str:
    """
    Blocks SSRF: resolves the host once and refuses any private, loopback,
    link-local, or other non-public IP (internal services, cloud metadata
    endpoints like 169.254.169.254, etc), returning the vetted address.

    The caller connects directly to this returned IP (rather than letting
    the HTTP client re-resolve the hostname) so a DNS answer that changes
    between this check and the actual connection -- a DNS-rebinding attack --
    cannot bypass the guard. This is an admin-only knowledge-base import
    feature, but the URL itself is still attacker-influenceable input.
    """
    try:
        infos = socket.getaddrinfo(hostname, None)
    except OSError as e:
        raise IngestionError(f"Could not resolve host: {hostname}") from e

    for info in infos:
        ip = ipaddress.ip_address(info[4][0])
        if (
            ip.is_private
            or ip.is_loopback
            or ip.is_link_local
            or ip.is_reserved
            or ip.is_multicast
            or ip.is_unspecified
        ):
            raise IngestionError(f"Refusing to fetch non-public address: {hostname} -> {ip}")
        return str(ip)

    raise IngestionError(f"Could not resolve host: {hostname}")


async def ingest_url(session: AsyncSession, url: str) -> KnowledgeBaseEntry:
    """
    Crawls an external help center or documentation URL (e.g., helpcenter.com),
    extracts main article body text, and saves it as a KnowledgeBaseEntry.
    """
    url = url.strip()
    if not url.startswith(("http://", "https://")):
        url = "https://" + url

    parsed = urlparse(url)
    if not parsed.netloc:
        raise IngestionError(f"Invalid URL: {url}")
    if parsed.username or parsed.password:
        # A userinfo component (https://user:pass@host/...) is never needed
        # for a public help-center URL, and allowing it both leaks
        # credentials into the Host header below and (because the pinned
        # authority was previously built via a string replace of the
        # hostname inside netloc) could leave the request targeting the
        # un-pinned original host if the replace matched inside userinfo
        # instead of the host.
        raise IngestionError(f"Refusing URL with embedded credentials: {url}")

    hostname = parsed.hostname or ""
    safe_ip = _resolve_public_ip(hostname)
    # Rebuild the authority explicitly from parts (safe_ip[:port]) rather
    # than string-replacing hostname inside netloc, which -- being a plain
    # first-occurrence, case-sensitive replace -- could silently no-op for a
    # mixed-case hostname and leave the request pointed at the un-pinned,
    # un-vetted original host.
    pinned_netloc = safe_ip if parsed.port is None else f"{safe_ip}:{parsed.port}"
    pinned_url = parsed._replace(netloc=pinned_netloc).geturl()

    headers = {
        "Host": parsed.netloc,
        "User-Agent": (
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
            "AppleWebKit/537.36 (KHTML, like Gecko) "
            "Chrome/124.0.0.0 Safari/537.36 (KnowledgeBaseIngestBot/1.0)"
        ),
    }

    try:
        # follow_redirects=False: a public URL redirecting to a private/
        # internal address is the classic SSRF-via-redirect bypass, so
        # redirects are rejected outright rather than silently followed.
        # The request is made against safe_ip (the vetted address), not the
        # hostname, so the HTTP client cannot re-resolve DNS and land on a
        # different (possibly private) address than the one that was checked.
        async with httpx.AsyncClient(timeout=25.0, follow_redirects=False) as client:
            resp = await client.get(
                pinned_url, headers=headers, extensions={"sni_hostname": hostname}
            )
            if resp.is_redirect:
                raise IngestionError(f"Refusing to follow redirect from {url}")
            resp.raise_for_status()
            html = resp.text
    except IngestionError:
        raise
    except Exception as e:
        raise IngestionError(f"Failed to fetch {url}: {e}") from e

    soup = BeautifulSoup(html, "html.parser")

    # Remove scripts, styles, navbars, footers, forms, cookie banners, ads to keep crisp text
    for tag in soup(
        [
            "script",
            "style",
            "noscript",
            "svg",
            "nav",
            "footer",
            "aside",
            "header",
            "form",
            "iframe",
            "button",
        ]
    ):
        tag.decompose()

    for el in soup.find_all(
        class_=re.compile(r"(cookie|consent|banner|newsletter|popup|modal|ad-)", re.IGNORECASE)
    ):
        el.decompose()

    # Extract title
    title = ""
    if soup.title and soup.title.string:
        title = soup.title.string.strip()
    elif soup.h1:
        title = soup.h1.get_text(strip=True)
    if not title:
        title = f"Documentation from {parsed.netloc}"

    # Extract body content
    main_node = soup.find("main") or soup.find("article") or soup.find("body") or soup
    raw_text = main_node.get_text(separator="\n")
    content = _clean_text(raw_text)

    if not content:
        raise IngestionError(f"No readable content could be extracted from {url}")

    # Truncate if exceptionally huge to stay within LLM context
    if len(content) > 30000:
        content = content[:30000] + "\n\n...[Content truncated for length]..."

    entry = KnowledgeBaseEntry(
        title=title[:250],
        content=content,
        source_type="url",
        source_url=url,
    )
    session.add(entry)
    await session.flush()
    return entry


def ingest_document_bytes(
    session: AsyncSession, filename: str, content_bytes: bytes
) -> KnowledgeBaseEntry:
    """
    Extracts text from uploaded documents (PDF, TXT, MD) and adds a KnowledgeBaseEntry.
    """
    clean_filename = filename.strip()
    lower_name = clean_filename.lower()
    extracted_text = ""

    if lower_name.endswith(".pdf"):
        try:
            reader = PdfReader(io.BytesIO(content_bytes))
            pages_text = []
            for i, page in enumerate(reader.pages):
                text = page.extract_text()
                if text and text.strip():
                    pages_text.append(f"--- Page {i + 1} ---\n{text.strip()}")
            extracted_text = "\n\n".join(pages_text)
        except Exception as e:
            raise IngestionError(f"Failed to parse PDF {clean_filename}: {e}") from e
    else:
        # Treat as plain text / markdown / json
        try:
            extracted_text = content_bytes.decode("utf-8")
        except UnicodeDecodeError:
            extracted_text = content_bytes.decode("latin-1", errors="replace")

    content = _clean_text(extracted_text)
    if not content:
        raise IngestionError(f"The document {clean_filename} contains no readable text.")

    if len(content) > 30000:
        content = content[:30000] + "\n\n...[Content truncated for length]..."

    # Create readable title from filename (e.g., "refund-policy.pdf" -> "Refund Policy")
    name_no_ext = re.sub(r"\.[^.]+$", "", clean_filename)
    title = name_no_ext.replace("-", " ").replace("_", " ").title()

    entry = KnowledgeBaseEntry(
        title=title[:250],
        content=content,
        source_type="document",
        file_name=clean_filename,
    )
    session.add(entry)
    return entry
