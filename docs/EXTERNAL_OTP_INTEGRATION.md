# External OTP Integration Guide

This guide explains how external e-commerce platforms (Shopify, custom stores, WooCommerce, Magento, etc.) connect to the Communicator OTP engine for OTP generation and verification.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                     BRAND'S E-COMMERCE PLATFORM                     │
│  (Shopify / Custom Store / WooCommerce / Any Sales Channel)         │
│                                                                     │
│  ┌──────────────┐         ┌──────────────┐         ┌──────────────┐│
│  │  Frontend JS  │         │  Backend API  │         │  Webhook     ││
│  │  (OTP Input)  │────────▶│  (Secret Key) │         │  Receiver    ││
│  │               │◀────────│               │◀────────│  (Callbacks) ││
│  └──────────────┘         └──────────────┘         └──────────────┘│
└───────────────┬───────────────────┬───────────────────┬─────────────┘
                │                   │                   │
                │ POST /generate    │ POST /verify      │ Webhook POST
                │ (Bearer sk_...)   │ (Bearer sk_...)   │ (HMAC signed)
                ▼                   ▼                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      COMMUNICATOR SERVER                            │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  External OTP API Layer                                      │  │
│  │  • Auth: Secret key verification (HMAC-SHA256)               │  │
│  │  • Rate limiting per integration                             │  │
│  │  • Purpose/channel permission check                          │  │
│  └────────────────────┬─────────────────────────────────────────┘  │
│                       │                                             │
│  ┌────────────────────▼─────────────────────────────────────────┐  │
│  │  OTP Engine (Banking-Grade)                                  │  │
│  │  • 12-char alphanumeric OTP from CSPRNG                      │  │
│  │  • HMAC-SHA256 hash storage                                  │  │
│  │  • IP/device binding                                         │  │
│  │  • Per-recipient + per-IP rate limiting                      │  │
│  └────────────────────┬─────────────────────────────────────────┘  │
│                       │                                             │
│  ┌────────────────────▼─────────────────────────────────────────┐  │
│  │  Channel Dispatch                                            │  │
│  │  • WhatsApp (Meta Cloud API)                                 │  │
│  │  • Email (SMTP)                                              │  │
│  │  • SMS (Twilio)                                              │  │
│  │  • In-App Messaging                                          │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

## Data Flow: Complete OTP Lifecycle

### Step 1: Brand Admin Creates Integration

The brand admin logs into the Communicator dashboard and creates an external integration for their e-commerce platform.

**API Call:**
```
POST https://communicator.example.com/api/v1/external-otp/integrations
Content-Type: application/json

{
  "brand_id": "brand_abc123",
  "platform_name": "shopify",
  "platform_url": "https://mystore.myshopify.com",
  "display_name": "My Shopify Store",
  "webhook_url": "https://mystore.myshopify.com/apps/communicator/webhook"
}
```

**Response:**
```json
{
  "success": true,
  "id": "ext_550e8400-e29b-41d4-a716-446655440000",
  "public_key": "pk_brand_ab123456",
  "secret_key": "sk_a1b2c3d4e5f6...",
  "webhook_secret": "f6e5d4c3b2a1...",
  "message": "Integration created successfully. Store the secret key securely — it will not be shown again."
}
```

**Important:** The `secret_key` is returned **only once**. Store it securely in your e-commerce platform's environment variables. The `webhook_secret` is used to verify incoming webhook signatures.

### Step 2: User Initiates Action on Brand Website

A user on the brand's Shopify store clicks "Sign Up" or "Verify Phone Number." The frontend shows an OTP input form.

**Frontend (JavaScript):**
```javascript
// Show OTP input form
// The public_key identifies the integration (safe to expose in frontend)
const publicKey = 'pk_brand_ab123456';

// Fetch integration info to show allowed channels
const info = await fetch(`/api/otp-info?public_key=${publicKey}`);
const integration = await info.json();
// integration.allowed_channels → ['whatsapp', 'email']
```

### Step 3: Backend Requests OTP Generation

The e-commerce platform's backend calls the Communicator to generate and send an OTP.

**API Call:**
```
POST https://communicator.example.com/api/v1/external-otp/generate
Authorization: Bearer sk_a1b2c3d4e5f6...
Content-Type: application/json

{
  "recipient": "+1234567890",
  "recipient_type": "phone",
  "purpose": "sign_up",
  "channel": "whatsapp",
  "metadata": {
    "user_id": "shopify_customer_123",
    "session_id": "sess_abc123"
  }
}
```

**Response:**
```json
{
  "success": true,
  "otp_id": "otp_550e8400-e29b-41d4-a716-446655440000",
  "channel": "whatsapp",
  "message": "OTP sent via WhatsApp to +1234567890",
  "expires_in_seconds": 300
}
```

The OTP is now sent to the user via WhatsApp (or the selected channel). The `otp_id` is returned to the e-commerce platform — it needs this for verification.

### Step 4: User Enters OTP on Brand Website

The user receives the OTP on WhatsApp and enters it on the brand's website.

**Frontend:**
```html
<!-- OTP Input Form on Shopify Store -->
<div id="otp-form">
  <input type="text" id="otp-code" placeholder="Enter 12-character code" maxlength="12" />
  <button id="otp-submit">Verify</button>
  <p id="otp-timer">Expires in: 5:00</p>
</div>

<input type="hidden" id="otp-id" value="otp_550e8400-..." />
```

### Step 5: Backend Verifies OTP

The e-commerce platform's backend sends the OTP code to Communicator for verification.

**API Call:**
```
POST https://communicator.example.com/api/v1/external-otp/verify
Authorization: Bearer sk_a1b2c3d4e5f6...
Content-Type: application/json

{
  "otp_id": "otp_550e8400-e29b-41d4-a716-446655440000",
  "code": "A7xK9mP2qR5t"
}
```

**Response (Success):**
```json
{
  "success": true,
  "verified": true,
  "message": "OTP verified successfully",
  "attempts_remaining": 4,
  "session_token": "otp_session_550e8400:a1b2c3d4e5f6:1700000000"
}
```

**Response (Failure):**
```json
{
  "success": false,
  "verified": false,
  "message": "Invalid OTP code",
  "attempts_remaining": 3,
  "error": "Code mismatch"
}
```

### Step 6: Webhook Callback

After verification (success or failure), Communicator sends a webhook to the e-commerce platform's configured `webhook_url`.

**Webhook Payload:**
```
POST https://mystore.myshopify.com/apps/communicator/webhook
Content-Type: application/json
X-Communicator-Event: otp_verified
X-Communicator-Timestamp: 2024-11-10T16:30:00Z
X-Communicator-Signature: sha256a1b2c3d4e5f6...

{
  "event_id": "evt_550e8400-e29b-41d4-a716-446655440000",
  "event_type": "otp_verified",
  "brand_id": "brand_abc123",
  "otp_id": "otp_550e8400-e29b-41d4-a716-446655440000",
  "recipient": "+1234567890",
  "channel": "whatsapp",
  "purpose": "sign_up",
  "timestamp": "2024-11-10T16:30:00Z",
  "data": {
    "verified": true,
    "attempts_remaining": 4,
    "session_token": "otp_session_550e8400:a1b2c3d4e5f6:1700000000",
    "error": null
  }
}
```

**Webhook Signature Verification:**
```javascript
// Verify the webhook signature in your backend
const crypto = require('crypto');

function verifyWebhookSignature(payload, signature, webhookSecret) {
  const expected = 'sha256' + crypto
    .createHmac('sha256', webhookSecret)
    .update(payload)
    .digest('hex');
  
  // Constant-time comparison
  return crypto.timingSafeEqual(
    Buffer.from(signature),
    Buffer.from(expected)
  );
}

// Express.js example
app.post('/apps/communicator/webhook', (req, res) => {
  const signature = req.headers['x-communicator-signature'];
  const payload = JSON.stringify(req.body);
  
  if (!verifyWebhookSignature(payload, signature, process.env.COMMUNICATOR_WEBHOOK_SECRET)) {
    return res.status(401).send('Invalid signature');
  }
  
  // Handle the event
  const event = req.body;
  if (event.event_type === 'otp_verified') {
    // Complete the user's sign-up flow
    completeSignUp(event.data);
  } else if (event.event_type === 'otp_failed') {
    // Handle failed verification
    handleFailedVerification(event.data);
  }
  
  res.status(200).send('OK');
});
```

---

## API Reference

### Authentication

All external OTP endpoints require the `Authorization: Bearer {secret_key}` header.

```
Authorization: Bearer sk_a1b2c3d4e5f6...
```

### Endpoints

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| `POST` | `/api/v1/external-otp/integrations` | Create integration key pair | Admin |
| `GET` | `/api/v1/external-otp/integrations/{brand_id}` | List integrations | Admin |
| `DELETE` | `/api/v1/external-otp/integrations/{id}` | Deactivate integration | Admin |
| `POST` | `/api/v1/external-otp/generate` | Generate and send OTP | Secret Key |
| `POST` | `/api/v1/external-otp/verify` | Verify OTP code | Secret Key |
| `GET` | `/api/v1/external-otp/info?public_key=pk_...` | Get integration info | None |

### Generate OTP

```
POST /api/v1/external-otp/generate
Authorization: Bearer {secret_key}

{
  "recipient": "+1234567890",           // Required: phone or email
  "recipient_type": "phone",            // Optional: "phone" or "email"
  "purpose": "sign_up",                 // Optional: defaults to "sign_up"
  "channel": "whatsapp",               // Optional: uses brand default
  "metadata": {}                        // Optional: additional context
}
```

**Purposes:** `sign_up`, `sign_in`, `password_reset`, `phone_verify`, `email_verify`, `custom`

**Channels:** `whatsapp`, `email`, `sms`, `in_app`

### Verify OTP

```
POST /api/v1/external-otp/verify
Authorization: Bearer {secret_key}

{
  "otp_id": "otp_...",    // Required: returned from generate
  "code": "A7xK9mP2qR5t"  // Required: the OTP code entered by user
}
```

### Webhook Events

| Event Type | Description |
|------------|-------------|
| `otp_generated` | OTP was generated and sent |
| `otp_verified` | OTP was successfully verified |
| `otp_failed` | OTP verification failed |
| `otp_expired` | OTP expired without verification |

---

## Platform-Specific Integration Examples

### Shopify Integration

**1. Create a Shopify App** that acts as the backend proxy between your store and Communicator.

**2. Store Credentials:**
```
# Shopify App .env
COMMUNICATOR_BASE_URL=https://communicator.example.com
COMMUNICATOR_SECRET_KEY=sk_a1b2c3d4e5f6...
COMMUNICATOR_WEBHOOK_SECRET=f6e5d4c3b2a1...
```

**3. Shopify Backend (Node.js):**
```javascript
const express = require('express');
const axios = require('axios');
const app = express();

app.post('/api/otp/generate', async (req, res) => {
  const { phone, purpose } = req.body;
  
  const response = await axios.post(
    `${process.env.COMMUNICATOR_BASE_URL}/api/v1/external-otp/generate`,
    {
      recipient: phone,
      recipient_type: 'phone',
      purpose: purpose || 'sign_up',
      channel: 'whatsapp'
    },
    {
      headers: {
        'Authorization': `Bearer ${process.env.COMMUNICATOR_SECRET_KEY}`,
        'Content-Type': 'application/json'
      }
    }
  );
  
  res.json(response.data);
});

app.post('/api/otp/verify', async (req, res) => {
  const { otpId, code } = req.body;
  
  const response = await axios.post(
    `${process.env.COMMUNICATOR_BASE_URL}/api/v1/external-otp/verify`,
    {
      otp_id: otpId,
      code: code
    },
    {
      headers: {
        'Authorization': `Bearer ${process.env.COMMUNICATOR_SECRET_KEY}`,
        'Content-Type': 'application/json'
      }
    }
  );
  
  res.json(response.data);
});

app.post('/webhooks/communicator', express.json(), (req, res) => {
  // Verify signature (see above)
  const event = req.body;
  
  if (event.event_type === 'otp_verified') {
    // Complete customer creation in Shopify
    // using the session_token from event.data.session_token
  }
  
  res.status(200).send('OK');
});
```

**4. Shopify Frontend (Liquid/JavaScript):**
```javascript
// In your Shopify theme or app
async function requestOtp(phone) {
  const response = await fetch('/api/otp/generate', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ phone, purpose: 'sign_up' })
  });
  
  const data = await response.json();
  
  if (data.success) {
    // Show OTP input form
    showOtpInput(data.otp_id, data.expires_in_seconds);
  }
}

async function verifyOtp(otpId, code) {
  const response = await fetch('/api/otp/verify', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ otpId, code })
  });
  
  const data = await response.json();
  
  if (data.verified) {
    // Redirect to dashboard or complete sign-up
    window.location.href = '/account/dashboard';
  } else {
    // Show error
    showError(data.message);
  }
}
```

### Custom E-Commerce Store (Python/Flask)

```python
import requests
import os

COMMUNICATOR_URL = os.environ['COMMUNICATOR_BASE_URL']
SECRET_KEY = os.environ['COMMUNICATOR_SECRET_KEY']

def generate_otp(recipient, purpose='sign_up', channel='whatsapp'):
    response = requests.post(
        f'{COMMUNICATOR_URL}/api/v1/external-otp/generate',
        json={
            'recipient': recipient,
            'recipient_type': 'phone' if recipient.startswith('+') else 'email',
            'purpose': purpose,
            'channel': channel
        },
        headers={
            'Authorization': f'Bearer {SECRET_KEY}',
            'Content-Type': 'application/json'
        }
    )
    return response.json()

def verify_otp(otp_id, code):
    response = requests.post(
        f'{COMMUNICATOR_URL}/api/v1/external-otp/verify',
        json={
            'otp_id': otp_id,
            'code': code
        },
        headers={
            'Authorization': f'Bearer {SECRET_KEY}',
            'Content-Type': 'application/json'
        }
    )
    return response.json()
```

### WooCommerce Integration (PHP)

```php
<?php
function communicator_generate_otp($recipient, $purpose = 'sign_up', $channel = 'whatsapp') {
    $response = wp_remote_post(get_env('COMMUNICATOR_BASE_URL') . '/api/v1/external-otp/generate', [
        'headers' => [
            'Authorization' => 'Bearer ' . get_env('COMMUNICATOR_SECRET_KEY'),
            'Content-Type'  => 'application/json'
        ],
        'body' => wp_json_encode([
            'recipient'      => $recipient,
            'recipient_type' => strpos($recipient, '+') === 0 ? 'phone' : 'email',
            'purpose'        => $purpose,
            'channel'        => $channel
        ])
    ]);
    
    return json_decode(wp_remote_retrieve_body($response), true);
}

function communicator_verify_otp($otp_id, $code) {
    $response = wp_remote_post(get_env('COMMUNICATOR_BASE_URL') . '/api/v1/external-otp/verify', [
        'headers' => [
            'Authorization' => 'Bearer ' . get_env('COMMUNICATOR_SECRET_KEY'),
            'Content-Type'  => 'application/json'
        ],
        'body' => wp_json_encode([
            'otp_id' => $otp_id,
            'code'   => $code
        ])
    ]);
    
    return json_decode(wp_remote_retrieve_body($response), true);
}
?>
```

---

## Security Considerations

### Secret Key Storage
- **Never** store the secret key in frontend JavaScript
- Store it in your backend's environment variables
- The secret key is returned only once at creation time
- If compromised, deactivate the integration and create a new one

### Webhook Signature Verification
- Always verify the `X-Communicator-Signature` header
- Use constant-time comparison to prevent timing attacks
- The signature is HMAC-SHA256 of the raw request body using your `webhook_secret`

### IP Binding
- OTPs are bound to the source IP address at generation time
- Verification from a different IP will be rejected
- If behind a load balancer, ensure `X-Forwarded-For` is properly handled

### Rate Limiting
- Per-integration rate limits: 10 requests/minute, 100/hour (configurable)
- Per-recipient rate limits: 3/minute, 10/hour (from OTP engine config)
- Per-IP rate limits: prevents SMS bombing attacks

---

## Troubleshooting

### "Invalid secret key" Error
- Verify the `Authorization` header format: `Bearer sk_...`
- Check that the integration is still active
- Recreate the integration if the key was compromised

### "Purpose not allowed" Error
- Check the `allowed_purposes` array in your integration config
- Update the integration to include the required purpose

### "Channel not allowed" Error
- Check the `allowed_channels` array in your integration config
- Ensure the brand has the channel enabled in OTP channel config

### Webhook Not Received
- Verify the `webhook_url` is accessible from the internet
- Check the webhook delivery log in Communicator admin
- Verify the webhook signature is being computed correctly

### OTP Verification Fails
- Check that the same IP address is used for generate and verify
- Verify the OTP hasn't expired (default 5 minutes)
- Check `attempts_remaining` — max 5 attempts by default

---

## Migration Setup

To enable external OTP integration, run the migration:

```bash
# Apply migration 0055
psql -h localhost -U communicator -d communicator -f backend-rs/migrations/0055_external_otp_integration.sql
```

Then create your first integration:

```bash
curl -X POST http://localhost:3000/api/v1/external-otp/integrations \
  -H "Content-Type: application/json" \
  -d '{
    "brand_id": "brand_abc123",
    "platform_name": "shopify",
    "platform_url": "https://mystore.myshopify.com",
    "display_name": "My Shopify Store",
    "webhook_url": "https://mystore.myshopify.com/apps/communicator/webhook"
  }'
```
