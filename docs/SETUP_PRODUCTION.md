# Production Setup Guide — Communicator + nID

## Overview

This guide gets both databases (Communicator + nID) running on PostgreSQL with real nID identity provider connections, sign-up, and sign-in.

## Prerequisites

- **PostgreSQL 16+** running on `localhost:5432`
- **Rust 1.75+** (for the Communicator backend)
- **Node.js 20+** (for the nID API)
- **Docker** (optional, for running PostgreSQL via docker-compose)

---

## Step 1: Start PostgreSQL

### Option A: Docker (Recommended)

From the nID directory:
```powershell
cd C:\Users\nommi\Downloads\scraper\nID
docker-compose up -d postgres
```

This starts PostgreSQL on `localhost:5432` with the `nid` database.

### Option B: Local PostgreSQL

Install PostgreSQL and create two databases:
```sql
CREATE DATABASE nid;
CREATE DATABASE communicator;

-- For nID (least-privilege app role)
CREATE ROLE nid_app WITH LOGIN PASSWORD 'nid_app';
GRANT CONNECT ON DATABASE nid TO nid_app;

-- For Communicator
CREATE ROLE communicator_app WITH LOGIN PASSWORD 'C0mm@nicat0r!2026#Secure@localhost:5432/communicator';
GRANT CONNECT ON DATABASE communicator TO communicator_app;
```

---

## Step 2: Set Up nID Database

```powershell
cd C:\Users\nommi\Downloads\scraper\nID

# Install dependencies
pnpm install

# Copy and configure environment
Copy-Item .env.example .env
# Edit .env — ensure DATABASE_URL points to your PostgreSQL

# Run migrations and seed (creates OAuth clients including Communicator)
cd packages\db
npx prisma db push
npx prisma db seed
```

**Important:** The seed script prints the Communicator OAuth client credentials:
```
Communicator OAuth client: client_comm_XXXXXXXX / <secret> (redirect: http://localhost:8001/api/v1/auth/nid/callback)
```

Copy these values — you need them for Step 3.

---

## Step 3: Configure Communicator

```powershell
cd C:\Users\nommi\Downloads\scraper\communicator

# The .env file is already created — fill in the nID credentials:
# Edit .env and set:
NID_CLIENT_ID=client_comm_XXXXXXXX    # from nID seed output
NID_CLIENT_SECRET=<secret>            # from nID seed output
NID_BASE_URL=http://localhost:4000
```

---

## Step 4: Start nID API

```powershell
cd C:\Users\nommi\Downloads\scraper\nID
pnpm --filter @nid/api dev
```

nID API starts on `http://localhost:4000`.

---

## Step 5: Apply Communicator Migrations

```powershell
cd C:\Users\nommi\Downloads\scraper\communicator\backend-rs

# Run sqlx migrations (creates the Communicator schema)
cargo run --bin sqlx migrate run --database-url "postgresql://communicator_app:C0mm@nicat0r!2026#Secure@localhost:5432/communicator"
```

---

## Step 6: Start Communicator Backend

```powershell
cd C:\Users\nommi\Downloads\scraper\communicator\backend-rs
cargo run
```

Communicator starts on `http://localhost:8000`.

---

## API Endpoints

### Sign Up (Create nID Account)

```
POST /api/v1/auth/sign-up/start
Body: { "phone": "+1234567890" }
→ Returns: { "status": "otp_sent", "phone": "+1234567890" }

POST /api/v1/auth/sign-up/verify
Body: { "phone": "+1234567890", "code": "123456" }
→ Returns: { "status": "signed_up", "token": "...", "account_id": "...", "phone": "..." }
```

### Sign In (Login with nID)

```
GET  /api/v1/auth/nid/start
→ Returns: { "authorize_url": "...", "poll_token": "...", "state": "..." }

GET  /api/v1/auth/nid/callback?code=XXX&state=XXX
→ Returns: { "token": "...", "account_id": "...", "email": "...", "phone": "..." }
```

### Profile

```
GET /api/v1/auth/me
Header: Authorization: Bearer <token>
→ Returns: { "status": "authenticated", "account_id": "...", ... }
```

### WhatsApp

```
GET  /api/v1/inboxes/channels/whatsapp
POST /api/v1/inboxes/channels/whatsapp
POST /api/v1/inboxes/channels/whatsapp/verify
POST /api/v1/inboxes/channels/whatsapp/test-message
```

### Share / Broadcasts

```
GET  /api/v1/broadcasts
POST /api/v1/broadcasts
POST /api/v1/broadcasts/{id}/dispatch
```

---

## Database Architecture

| Database | Service | Connection |
|----------|---------|------------|
| `nid` | nID (Identity Provider) | `postgresql://nid_app:nid_app@localhost:5432/nid` |
| `communicator` | Communicator (Messaging) | `postgresql://communicator_app:***@localhost:5432/communicator` |

Both databases run on the same PostgreSQL server but are completely separate. nID handles identity; Communicator handles conversations, messages, and broadcasts.

---

## WhatsApp Configuration

To enable real WhatsApp messaging:

1. Go to [Meta for Business](https://business.facebook.com/settings/)
2. Create a WhatsApp Business Account
3. Get your Access Token and Phone Number ID
4. Set in `.env`:
   ```
   META_WHATSAPP_ACCESS_TOKEN=<your_token>
   META_WHATSAPP_PHONE_NUMBER_ID=<your_phone_id>
   META_WEBHOOK_VERIFY_TOKEN=<your_verify_token>
   META_APP_SECRET=<your_app_secret>
   ```
5. Configure webhook URL: `https://your-domain/api/v1/webhooks/meta/whatsapp`

---

## Troubleshooting

### nID not reachable
- Ensure nID API is running on port 4000
- Check `NID_BASE_URL` in Communicator's `.env`

### Database connection failed
- Verify PostgreSQL is running: `docker ps` or check service status
- Check credentials in `.env` match what you created

### OAuth client not found
- Re-run `npx prisma db seed` in the nID directory
- Ensure `NID_CLIENT_ID` and `NID_CLIENT_SECRET` match the seed output

### Mock auth still active
- Remove or set `NID_ALLOW_MOCK_AUTH=false` in `.env`
- Restart the Communicator backend
