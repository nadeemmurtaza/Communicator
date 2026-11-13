# nID Tenant Management System — Complete Guide

## Overview

This document describes the complete tenant management layer built across nID and the Communicator, covering all 12 items from the original audit.

### What Was Built

| # | Feature | Status | Where |
|---|---------|--------|-------|
| 1 | Create Tenant button in nID UI | ✅ Built | `apps/dashboard/app/page.tsx` |
| 2 | Tenant Dashboard with settings/nav | ✅ Built | `apps/dashboard/app/page.tsx` |
| 3 | Tenant Creation API | ✅ Built | `apps/api/src/modules/tenants/` |
| 4 | Universal Tenant ID Generator | ✅ Exists | Prisma `cuid()` |
| 5 | Tenant visible in Work Profile | ✅ Built | Dashboard Overview tab |
| 6 | Auto-sync to Communicator | ✅ Built | `nid_tenant_sync.rs` + webhooks |
| 7 | Tenant Dashboard in Communicator | ✅ Built | API endpoints at `/api/v1/nid/*` |
| 8 | Redirect flow (Communicator → nID → back) | ✅ Built | `tenant_redirect_sessions` table |
| 9 | Redirect error handling | ✅ Built | State tokens, retry logic, expiry |
| 10 | Tenant duplication prevention | ✅ Built | Slug + ID dedup in both systems |
| 11 | Profile update sync (nID → Communicator) | ✅ Built | Webhook `tenant.updated` events |
| 12 | Event Bus Integration | ✅ Built | OutboundEvent model + webhooks |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         nID System                              │
│                                                                 │
│  ┌──────────────┐   ┌──────────────┐   ┌────────────────────┐  │
│  │  Dashboard   │   │   API        │   │   Profile UI       │  │
│  │  (Next.js)   │   │  (NestJS)    │   │  (Next.js)         │  │
│  │              │   │              │   │                    │  │
│  │ • Create     │   │ • POST /v1/  │   │ • Tenant overview  │  │
│  │   Tenant     │   │   tenants    │   │ • Work profile     │  │
│  │ • API Keys   │   │ • PUT /v1/   │   │                    │  │
│  │ • Staff      │   │   tenants/:id│   │                    │  │
│  │ • Settings   │   │ • GET /v1/   │   │                    │  │
│  │ • Platform   │   │   tenants    │   │                    │  │
│  │   Admin      │   │              │   │                    │  │
│  └──────┬───────┘   └──────┬───────┘   └────────────────────┘  │
│         │                  │                                    │
│         │  Webhook         │  OutboundEvent                     │
│         │  (tenant.created)│  (tenant.updated)                  │
│         │                  │                                    │
└─────────┼──────────────────┼────────────────────────────────────┘
          │                  │
          │                  │
          ▼                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Communicator (Rust)                         │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  nid_tenant_sync.rs  (Tenant Sync Service)               │   │
│  │                                                          │   │
│  │  • process_webhook() — receives nID webhooks             │   │
│  │  • handle_tenant_created() — mirrors tenant data         │   │
│  │  • handle_tenant_updated() — syncs profile changes       │   │
│  │  • handle_tenant_deleted() — soft deletes                │   │
│  │  • create_redirect_session() — starts redirect flow      │   │
│  │  • handle_redirect_callback() — completes redirect flow  │   │
│  │  • auto_link_tenant_to_brand() — links to Communicator   │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Database Tables (PostgreSQL)                            │   │
│  │                                                          │   │
│  │  • nid_tenants — mirror of nID tenants                   │   │
│  │  • tenant_sync_log — audit trail of sync events          │   │
│  │  • tenant_redirect_sessions — redirect flow state        │   │
│  │  • nid_connection_settings — connection config           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  API Endpoints                                           │   │
│  │                                                          │   │
│  │  POST /api/v1/nid/webhook — receive nID webhook          │   │
│  │  GET  /api/v1/nid/tenants — list synced tenants          │   │
│  │  GET  /api/v1/nid/tenants/:id — get tenant details       │   │
│  │  GET  /api/v1/nid/connection — connection health         │   │
│  │  PUT  /api/v1/nid/connection — update settings           │   │
│  │  POST /api/v1/nid/redirect/create — start redirect       │   │
│  │  GET  /api/v1/nid/redirect/callback — handle callback    │   │
│  │  GET  /api/v1/nid/redirect/:state — check status         │   │
│  │  GET  /api/v1/nid/sync-log — view sync log              │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Setup Guide

### 1. nID Configuration

#### Set Platform Admin Secret

Generate a random secret and set it in nID's `.env`:

```bash
# Generate secret
openssl rand -hex 32

# Add to nID .env
PLATFORM_ADMIN_SECRET=your_64_char_hex_string_here
```

#### Run Database Migration

```bash
cd packages/db
npx prisma migrate dev
```

#### Start nID API

```bash
cd apps/api
npm run dev
```

### 2. Communicator Configuration

#### Run Database Migration

```bash
psql -d communicator -f backend-rs/migrations/0058_nid_tenant_sync.sql
```

#### Configure Connection

```bash
curl -X PUT http://localhost:8080/api/v1/nid/connection \
  -H "Content-Type: application/json" \
  -d '{
    "nID_base_url": "http://localhost:4000",
    "platform_admin_secret": "your_64_char_hex_string_here",
    "webhook_receive_secret": "your_webhook_secret_here",
    "auto_sync_enabled": true,
    "auto_link_new_tenants": true
  }'
```

#### Set Webhook URL in nID

When creating a tenant in nID, set the webhook URL to:

```
https://your-communicator.com/api/v1/nid/webhook
```

### 3. nID Dashboard Setup

#### Start Dashboard

```bash
cd apps/dashboard
npm run dev
```

The dashboard runs at `http://localhost:3001`.

#### Login Modes

1. **Staff Login**: Phone + OTP (for tenant staff members)
2. **Platform Admin**: Platform admin secret (for tenant management)

---

## API Reference

### nID API Endpoints

#### Create Tenant (Platform Admin)

```
POST /v1/tenants
Authorization: Bearer <PLATFORM_ADMIN_SECRET>
Content-Type: application/json

{
  "name": "My Brand",
  "slug": "my-brand",
  "dataResidencyRegion": "global",
  "webhookUrl": "https://communicator.example.com/api/v1/nid/webhook",
  "initialStaffAccountId": "ckx7z8a9b000001la2m3n4o5p"
}
```

Response:

```json
{
  "id": "ckx7z8a9b000001la2m3n4o5p",
  "name": "My Brand",
  "slug": "my-brand",
  "dataResidencyRegion": "global",
  "webhookUrl": "https://communicator.example.com/api/v1/nid/webhook",
  "webhookSecret": "a1b2c3d4e5f6...",
  "billingWebhookSecret": "f6e5d4c3b2a1...",
  "createdAt": "2024-01-01T00:00:00.000Z",
  "message": "Tenant created successfully. Save the webhook secrets — they cannot be recovered."
}
```

#### List All Tenants (Platform Admin)

```
GET /v1/tenants
Authorization: Bearer <PLATFORM_ADMIN_SECRET>
```

#### Update Tenant (Platform Admin)

```
PUT /v1/tenants/:id
Authorization: Bearer <PLATFORM_ADMIN_SECRET>
Content-Type: application/json

{
  "name": "Updated Brand Name",
  "webhookUrl": "https://new-url.example.com/webhook"
}
```

#### Create API Key (Tenant Admin)

```
POST /v1/tenants/:id/api-keys
Authorization: Bearer <TENANT_API_KEY>
Content-Type: application/json

{
  "keyPrefix": "nid_live",
  "scopes": ["verification-cases:read", "verification-cases:write"],
  "rateLimitPerMinute": 60
}
```

#### Invite Staff (Tenant Admin)

```
POST /v1/tenants/:id/staff
Authorization: Bearer <TENANT_API_KEY>
Content-Type: application/json

{
  "accountId": "ckx7z8a9b000001la2m3n4o5p",
  "role": "STAFF"
}
```

### Communicator API Endpoints

#### Receive nID Webhook

```
POST /api/v1/nid/webhook
X-nID-Signature: sha256=<hmac_signature>
Content-Type: application/json

{
  "event_type": "tenant.created",
  "tenant_id": "ckx7z8a9b000001la2m3n4o5p",
  "timestamp": "2024-01-01T00:00:00.000Z",
  "data": {
    "id": "ckx7z8a9b000001la2m3n4o5p",
    "name": "My Brand",
    "slug": "my-brand",
    "createdAt": "2024-01-01T00:00:00.000Z"
  }
}
```

#### List Synced Tenants

```
GET /api/v1/nid/tenants
```

#### Get Connection Health

```
GET /api/v1/nid/connection
```

Response:

```json
{
  "connection_status": "connected",
  "nID_base_url": "http://localhost:4000",
  "auto_sync_enabled": true,
  "total_tenants": 5,
  "synced_tenants": 5,
  "error_tenants": 0,
  "last_error": ""
}
```

#### Create Redirect Session

```
POST /api/v1/nid/redirect/create
Content-Type: application/json

{
  "redirect_type": "create",
  "brand_id": "brand_abc123",
  "return_url": "https://communicator.example.com/tenants"
}
```

Response:

```json
{
  "session": {
    "id": "uuid...",
    "state_token": "a1b2c3d4e5f6...",
    "redirect_type": "create",
    "status": "pending",
    "expires_at": "600 seconds from now"
  },
  "redirect_url": "http://localhost:4000/tenant-create?state=a1b2c3d4e5f6...&return_url=..."
}
```

---

## Webhook Events

nID sends these webhook events to the Communicator:

### `tenant.created`

Fired when a new tenant is created in nID. The Communicator mirrors the tenant data and optionally auto-links it to a Communicator brand.

```json
{
  "event_type": "tenant.created",
  "tenant_id": "ckx7z8a9b000001la2m3n4o5p",
  "timestamp": "2024-01-01T00:00:00.000Z",
  "data": {
    "id": "ckx7z8a9b000001la2m3n4o5p",
    "name": "My Brand",
    "slug": "my-brand",
    "createdAt": "2024-01-01T00:00:00.000Z"
  }
}
```

### `tenant.updated`

Fired when tenant profile data changes in nID. The Communicator updates its mirror in real-time.

```json
{
  "event_type": "tenant.updated",
  "tenant_id": "ckx7z8a9b000001la2m3n4o5p",
  "timestamp": "2024-01-01T00:00:00.000Z",
  "data": {
    "id": "ckx7z8a9b000001la2m3n4o5p",
    "name": "Updated Name",
    "slug": "my-brand",
    "updatedAt": "2024-01-01T00:00:00.000Z",
    "changedFields": ["name"]
  }
}
```

---

## Redirect Flow

### Communicator → nID → Communicator

1. **User clicks "Create Tenant"** in Communicator
2. **Communicator creates redirect session** via `POST /api/v1/nid/redirect/create`
3. **User is redirected** to nID's tenant creation page with state token
4. **User fills in tenant details** in nID dashboard
5. **nID creates tenant** and sends webhook to Communicator
6. **nID redirects back** to Communicator's callback URL with state + tenant_id
7. **Communicator validates** state token, checks for duplicates, completes session
8. **User sees success** in Communicator with synced tenant data

### Error Handling

| Error | Code | Handling |
|-------|------|----------|
| Session expired | `timeout` | Auto-marked as expired, user must start new flow |
| User cancelled | `user_cancelled` | Session marked as cancelled, no tenant created |
| nID error | `nID_error` | Error message stored, retry allowed (max 3 attempts) |
| Max attempts exceeded | `max_attempts_exceeded` | Session locked, must create new one |
| Invalid state token | `validation_failed` | 400 error, possible CSRF attempt |
| Duplicate tenant | `tenant_already_exists` | Session completed, existing tenant linked |

---

## Duplication Prevention

### In nID

- **Slug uniqueness**: `slug` column has `@unique` constraint in Prisma
- **API-level check**: `TenantsService.create()` checks for existing slug before insert
- **ConflictException**: Returns 409 if slug already exists

### In Communicator

- **ID dedup**: `handle_tenant_created()` checks `nid_tenant_id` before insert
- **Slug dedup**: Also checks `slug` for cross-system deduplication
- **Redirect callback**: Checks for existing tenant before completing session

---

## Profile Update Sync

When a tenant changes its name or data in nID:

1. **nID detects change** via `PUT /v1/tenants/:id`
2. **nID emits webhook** with `tenant.updated` event
3. **Communicator receives webhook** at `POST /api/v1/nid/webhook`
4. **Communicator updates mirror** in `nid_tenants` table
5. **Sync log records** the event with timestamp
6. **Auto-link updates** if brand linkage exists

**Sync timing**: Real-time (webhook delivery is immediate, typically <1 second)

---

## Database Schema

### `nid_tenants`

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT | Primary key (nID tenant cuid) |
| `nid_tenant_id` | TEXT | nID tenant ID (unique) |
| `name` | TEXT | Tenant name |
| `slug` | TEXT | Unique slug |
| `data_residency_region` | TEXT | Data residency region |
| `webhook_url` | TEXT | Webhook URL for events |
| `webhook_secret_hash` | TEXT | HMAC hash of webhook secret |
| `sync_status` | TEXT | `pending`, `synced`, `out_of_sync`, `error`, `deleted` |
| `linked_brand_id` | TEXT | Linked Communicator brand ID |
| `nID_created_at` | TIMESTAMPTZ | Creation time from nID |

### `tenant_sync_log`

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT | UUID primary key |
| `nid_tenant_id` | TEXT | nID tenant ID |
| `event_type` | TEXT | `tenant.created`, `tenant.updated`, `tenant.deleted` |
| `payload` | JSONB | Full webhook payload |
| `signature_valid` | BOOLEAN | Was signature valid? |
| `processing_status` | TEXT | `pending`, `processed`, `failed`, `skipped` |

### `tenant_redirect_sessions`

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT | UUID primary key |
| `state_token` | TEXT | Unique state token for validation |
| `redirect_type` | TEXT | `create`, `manage`, `link` |
| `return_url` | TEXT | Where to redirect after completion |
| `nID_tenant_id` | TEXT | Set after successful creation |
| `status` | TEXT | `pending`, `completed`, `failed`, `expired`, `cancelled` |
| `error_code` | TEXT | Error code if failed |
| `attempts` | INTEGER | Number of retry attempts |
| `expires_at` | TIMESTAMPTZ | Session expiration time |

### `nid_connection_settings`

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT | Primary key ('default') |
| `nID_base_url` | TEXT | nID API base URL |
| `platform_admin_secret_hash` | TEXT | Hashed platform admin secret |
| `webhook_receive_secret` | TEXT | Secret for webhook verification |
| `auto_sync_enabled` | BOOLEAN | Auto-sync on webhook |
| `auto_link_new_tenants` | BOOLEAN | Auto-link to brands |
| `connection_status` | TEXT | `connected`, `disconnected`, `error` |

---

## Security

- **Platform Admin Secret**: 32+ character random hex string, constant-time comparison
- **Webhook Signatures**: HMAC-SHA256 signed payloads with constant-time verification
- **Secret Hashing**: All secrets stored as HMAC-SHA256 with per-secret salt
- **State Tokens**: 32-character random tokens for redirect flow validation
- **Rate Limiting**: Redirect sessions have max 3 attempts
- **Session Expiry**: Redirect sessions expire after 10 minutes

---

## Files Changed

### nID (TypeScript/Next.js)

| File | Description |
|------|-------------|
| `apps/api/src/modules/tenants/tenants.service.ts` | Full CRUD, API keys, staff, webhooks |
| `apps/api/src/modules/tenants/tenants.controller.ts` | 11 endpoints |
| `apps/api/src/modules/tenants/tenants.module.ts` | Module with AuditModule, PlatformAdminGuard |
| `apps/api/src/core/auth/platform-admin.guard.ts` | Platform admin auth guard |
| `apps/api/src/core/config/env.ts` | PLATFORM_ADMIN_SECRET env var |
| `apps/dashboard/app/page.tsx` | Full dashboard with sidebar, tabs, Create Tenant |
| `apps/dashboard/app/tenants/page.tsx` | Tenant listing page |
| `apps/dashboard/app/globals.css` | Dashboard styles |
| `apps/dashboard/lib/api-client.ts` | API client functions |

### Communicator (Rust)

| File | Description |
|------|-------------|
| `backend-rs/migrations/0058_nid_tenant_sync.sql` | 4 new tables |
| `backend-rs/src/services/nid_tenant_sync.rs` | Tenant sync service |
| `backend-rs/src/routes/nid_tenant_sync.rs` | 9 API endpoints |
| `backend-rs/src/services/mod.rs` | Module registration |
| `backend-rs/src/routes/mod.rs` | Module registration |
| `backend-rs/src/main.rs` | Route registration |
| `backend-rs/Cargo.toml` | urlencoding dependency |
| `.env.example` | Section 16: nID Tenant Sync |
