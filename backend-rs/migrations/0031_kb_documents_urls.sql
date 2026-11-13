-- Phase 23: Document Upload & URL Crawling for Knowledge Base
-- Enables uploading PDF/DOCX files and crawling webpages to automatically
-- populate the knowledge base. Mirrors Intercom's document upload and URL
-- crawling capabilities.

-- Uploaded documents (PDF, DOCX, TXT, etc.)
CREATE TABLE IF NOT EXISTS kb_documents (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    file_name TEXT NOT NULL,
    file_type TEXT NOT NULL, -- 'pdf' | 'docx' | 'txt' | 'html' | 'md'
    file_size INTEGER NOT NULL DEFAULT 0,
    file_hash TEXT, -- SHA256 hash for duplicate detection
    storage_path TEXT, -- path to stored file
    status TEXT NOT NULL DEFAULT 'processing', -- 'processing' | 'ready' | 'error' | 'deleted'
    error_message TEXT,
    pages_count INTEGER,
    chunks_count INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_kb_documents_org ON kb_documents(organization_id);
CREATE INDEX IF NOT EXISTS idx_kb_documents_status ON kb_documents(status);
CREATE INDEX IF NOT EXISTS idx_kb_documents_hash ON kb_documents(file_hash);

-- Document chunks (extracted text segments that become KB entries)
CREATE TABLE IF NOT EXISTS kb_document_chunks (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    page_number INTEGER,
    kb_entry_id TEXT, -- linked knowledge_base entry ID
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_kb_doc_chunks_doc ON kb_document_chunks(document_id);
CREATE INDEX IF NOT EXISTS idx_kb_doc_chunks_kb_entry ON kb_document_chunks(kb_entry_id);

-- URL sources (webpages crawled for knowledge)
CREATE TABLE IF NOT EXISTS kb_url_sources (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    url TEXT NOT NULL,
    title TEXT,
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending' | 'crawling' | 'ready' | 'error' | 'deleted'
    error_message TEXT,
    content_length INTEGER,
    chunks_count INTEGER,
    last_crawled_at TEXT,
    next_crawl_at TEXT,
    crawl_interval_hours INTEGER NOT NULL DEFAULT 168, -- default 7 days
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_kb_url_sources_org ON kb_url_sources(organization_id);
CREATE INDEX IF NOT EXISTS idx_kb_url_sources_status ON kb_url_sources(status);
CREATE INDEX IF NOT EXISTS idx_kb_url_sources_url ON kb_url_sources(url);

-- URL crawl logs (track each crawl operation)
CREATE TABLE IF NOT EXISTS kb_url_crawl_logs (
    id TEXT PRIMARY KEY,
    url_source_id TEXT NOT NULL,
    status_code INTEGER,
    content_length INTEGER,
    chunks_created BOOLEAN NOT NULL DEFAULT false,
    status TEXT NOT NULL DEFAULT 'running', -- 'running' | 'completed' | 'failed'
    error_message TEXT,
    started_at TEXT NOT NULL DEFAULT (NOW()),
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_kb_url_crawl_logs_source ON kb_url_crawl_logs(url_source_id);
CREATE INDEX IF NOT EXISTS idx_kb_url_crawl_logs_started ON kb_url_crawl_logs(started_at);

-- URL crawl chunks (extracted text segments from crawled pages)
CREATE TABLE IF NOT EXISTS kb_url_chunks (
    id TEXT PRIMARY KEY,
    url_source_id TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    section_heading TEXT,
    kb_entry_id TEXT, -- linked knowledge_base entry ID
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_kb_url_chunks_source ON kb_url_chunks(url_source_id);
CREATE INDEX IF NOT EXISTS idx_kb_url_chunks_kb_entry ON kb_url_chunks(kb_entry_id);

-- KB import settings (per-organization configuration)
CREATE TABLE IF NOT EXISTS kb_import_settings (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL UNIQUE,
    max_file_size_mb INTEGER NOT NULL DEFAULT 10,
    allowed_file_types TEXT NOT NULL DEFAULT '["pdf","docx","txt","html","md"]',
    chunk_size INTEGER NOT NULL DEFAULT 2000, -- characters per chunk
    chunk_overlap INTEGER NOT NULL DEFAULT 200, -- overlap between chunks
    auto_create_kb_entries BOOLEAN NOT NULL DEFAULT true, -- auto-create KB entries from chunks
    crawl_interval_hours INTEGER NOT NULL DEFAULT 168, -- default crawl interval
    max_url_depth INTEGER NOT NULL DEFAULT 3, -- max crawl depth
    user_agent TEXT NOT NULL DEFAULT 'Communicator-KB-Crawler/1.0',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Default settings
INSERT INTO kb_import_settings (id, organization_id, max_file_size_mb, chunk_size, chunk_overlap)
VALUES ('kb-settings-default', 'org-default', 10, 2000, 200);
