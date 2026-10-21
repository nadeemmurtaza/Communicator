from __future__ import annotations

import io
import re
from urllib.parse import urlparse

import httpx
from bs4 import BeautifulSoup
from pypdf import PdfReader
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.knowledge_base import KnowledgeBaseEntry


class IngestionError(Exception):
    """Raised when URL or document ingestion fails."""


def _clean_text(raw: str) -> str:
    """Collapses excess blank lines and whitespace."""
    lines = [line.strip() for line in raw.splitlines()]
    non_empty = [line for line in lines if line]
    return "\n\n".join(non_empty)


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

    headers = {
        "User-Agent": (
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
            "AppleWebKit/537.36 (KHTML, like Gecko) "
            "Chrome/124.0.0.0 Safari/537.36 (KnowledgeBaseIngestBot/1.0)"
        )
    }

    try:
        async with httpx.AsyncClient(timeout=25.0, follow_redirects=True) as client:
            resp = await client.get(url, headers=headers)
            resp.raise_for_status()
            html = resp.text
    except Exception as e:
        raise IngestionError(f"Failed to fetch {url}: {e}") from e

    soup = BeautifulSoup(html, "html.parser")

    # Remove script, style, nav, footer, ads, header tags to keep clean text
    for tag in soup(["script", "style", "noscript", "svg", "nav", "footer", "aside", "header"]):
        tag.decompose()

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
