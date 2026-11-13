# Shopify Store OTP Integration — Setup Guide

## Overview

This guide explains how to connect a **Shopify store** (the actual storefront built on Shopify) to the Communicator OTP engine. When a customer creates an account on your Shopify store, the OTP is generated and sent via your preferred channels (WhatsApp, SMS, Email).

**Key concept:** This connects to the **store website** where customers shop, not the Shopify admin panel. The brand admin controls which channels are used for OTP delivery.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Customer Journey                            │
│                                                                 │
│  1. Customer visits mystore.myshopify.com/account/register      │
│  2. Fills in name, email, phone, password                       │
│  3. Clicks "Create Account"                                     │
│         ↓                                                       │
│  4. Communicator OTP Widget intercepts the form                 │
│     → Sends customer data to Communicator API                   │
│         ↓                                                       │
│  5. Communicator generates 12-char OTP                          │
│     → Sends via brand-preferred channel (WhatsApp/SMS/Email)    │
│         ↓                                                       │
│  6. Customer receives OTP on WhatsApp/SMS/Email                 │
│     → Enters OTP on the verification widget                     │
│         ↓                                                       │
│  7. Communicator verifies OTP                                   │
│     → Returns session token                                     │
│         ↓                                                       │
│  8. Customer account created in Shopify                         │
│     → Customer can now log in                                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Step 1: Brand Admin — Register Your Shopify Store

The brand admin registers the Shopify store in the Communicator dashboard.

### API Call

```bash
POST /api/v1/shopify/connect
Content-Type: application/json

{
  "brand_id": "brand_abc123",
  "store_domain": "mystore.myshopify.com",
  "store_name": "My Clothing Store",
  "shopify_api_key": "shpat_xxxx",           // optional: for Admin API
  "shopify_api_secret": "shpss_xxxx",        // optional: for Admin API
  "shopify_access_token": "shpat_xxxx",      // optional: for auto customer creation
  "auto_create_customer": true,              // default: true
  "customer_tag": "otp_verified"             // default: "otp_verified"
}
```

### Response

```json
{
  "success": true,
  "id": "shop_550e8400-e29b-41d4-a716-446655440000",
  "store_domain": "mystore.myshopify.com",
  "public_key": "pk_shop_abc123_xyz789",
  "webhook_url": "/api/v1/shopify/webhook/shop_550e8400-e29b-41d4-a716-446655440000",
  "message": "Shopify store connected successfully. Use the public key to embed the OTP widget on your store."
}
```

**Save the `public_key`** — you'll need it to configure the widget.

---

## Step 2: Configure Channel Preferences

The brand admin chooses which channels are used for OTP delivery and their priority order.

### Get Current Preferences

```bash
GET /api/v1/shopify/{connection_id}/channels
```

### Update Preferences

```bash
PUT /api/v1/shopify/{connection_id}/channels
Content-Type: application/json

{
  "whatsapp_enabled": true,
  "email_enabled": true,
  "sms_enabled": false,
  "in_app_enabled": false,
  "default_channel": "whatsapp",
  "priority_order": ["whatsapp", "email", "sms"]
}
```

**Channel Selection Logic:**
1. If customer provides a phone number → WhatsApp (if enabled) → SMS (if enabled) → Email
2. If customer only provides email → Email
3. Priority order determines which channel is tried first

---

## Step 3: Shopify Store Owner — Embed the Widget

### Option A: Add to Shopify Theme (Recommended)

1. Go to your Shopify Admin → Online Store → Themes → Edit Code
2. Open `customer/account/register.liquid` (or `customer/templates/register.liquid`)
3. Add the following before `</body>`:

```liquid
<!-- Communicator OTP Widget -->
<script src="https://your-communicator.com/assets/shopify-otp-widget.js"></script>
<script>
  CommunicatorOTP.init({
    publicKey: 'pk_shop_abc123_xyz789',
    apiUrl: 'https://your-communicator.com/api/v1',
    storeDomain: 'mystore.myshopify.com',
    theme: {
      primaryColor: '#008060',  // Match your store's brand color
      fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      borderRadius: '8px'
    },
    messages: {
      sending: 'Sending verification code...',
      verifyButton: 'Verify & Create Account',
      resendButton: 'Resend Code'
    }
  });
</script>
```

4. Save the file

### Option B: Custom Domain / Headless Shopify

If you're using a custom domain or headless Shopify, add the widget to your account registration page:

```html
<script src="https://your-communicator.com/assets/shopify-otp-widget.js"></script>
<script>
  CommunicatorOTP.init({
    publicKey: 'pk_shop_abc123_xyz789',
    apiUrl: 'https://your-communicator.com/api/v1',
    storeDomain: 'mystore.myshopify.com'
  });
</script>
```

---

## Step 4: How It Works for the Customer

### Customer Account Creation Flow

1. **Customer visits** `mystore.myshopify.com/account/register`
2. **Fills the form** with name, email, phone, password
3. **Clicks "Create Account"**
4. **Widget intercepts** the form submission
5. **OTP is sent** via the brand's preferred channel (WhatsApp/SMS/Email)
6. **Widget shows** the OTP input screen
7. **Customer enters** the OTP code
8. **Widget verifies** the code with Communicator
9. **Account is created** in Shopify
10. **Customer is redirected** to the login page

### What the Customer Sees

```
┌────────────────────────────────────┐
│         🔒                         │
│     Verify Your Account            │
│                                    │
│ We've sent a verification code     │
│ to your WhatsApp                   │
│                                    │
│   ┌──────────────────────────┐    │
│   │   A7xK9mP2qR5t          │    │
│   └──────────────────────────┘    │
│                                    │
│   [  Verify & Create Account  ]    │
│                                    │
│  Didn't receive the code?          │
│  Resend Code (60s)                 │
└────────────────────────────────────┘
```

---

## API Reference

### Store Connection Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/shopify/connect` | Register a new Shopify store |
| `GET` | `/api/v1/shopify/connections/{brand_id}` | List all connected stores |
| `GET` | `/api/v1/shopify/{connection_id}/channels` | Get channel preferences |
| `PUT` | `/api/v1/shopify/{connection_id}/channels` | Update channel preferences |

### Account Creation Flow

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/shopify/{store_domain}/account/create` | Initiate account creation |
| `POST` | `/api/v1/shopify/account/verify` | Verify OTP |
| `POST` | `/api/v1/shopify/account/resend` | Resend OTP |

### Webhooks

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/shopify/webhook/{connection_id}` | Receive Shopify webhooks |

---

## API Examples

### Initiate Account Creation

```bash
POST /api/v1/shopify/mystore.myshopify.com/account/create
Content-Type: application/json

{
  "email": "customer@example.com",
  "phone": "+1234567890",
  "first_name": "John",
  "last_name": "Doe",
  "channel": "whatsapp"  // optional: override default
}
```

**Response:**
```json
{
  "success": true,
  "pending_id": "pending_550e8400-e29b-41d4-a716-446655440000",
  "otp_id": "otp_550e8400-e29b-41d4-a716-446655440000",
  "channel": "whatsapp",
  "message": "OTP sent via WhatsApp. Please enter the code to complete your account creation.",
  "expires_in_seconds": 300
}
```

### Verify OTP

```bash
POST /api/v1/shopify/account/verify
Content-Type: application/json

{
  "pending_id": "pending_550e8400-e29b-41d4-a716-446655440000",
  "code": "A7xK9mP2qR5t"
}
```

**Response:**
```json
{
  "success": true,
  "verified": true,
  "message": "Account verified and created successfully!",
  "customer_created": true,
  "session_token": "otp_session_xxx"
}
```

### Resend OTP

```bash
POST /api/v1/shopify/account/resend
Content-Type: application/json

{
  "pending_id": "pending_550e8400-e29b-41d4-a716-446655440000"
}
```

---

## Widget Configuration Options

```javascript
CommunicatorOTP.init({
  // Required
  publicKey: 'pk_shop_abc123_xyz789',
  apiUrl: 'https://your-communicator.com/api/v1',
  storeDomain: 'mystore.myshopify.com',

  // Optional: Theme customization
  theme: {
    primaryColor: '#008060',        // Button and accent color
    fontFamily: 'inherit',          // Font family
    borderRadius: '8px',            // Border radius
    fontSize: '14px'                // Base font size
  },

  // Optional: Custom messages
  messages: {
    sending: 'Sending verification code...',
    sent: 'Code sent! Check your {channel}.',
    verifyPlaceholder: 'Enter the verification code',
    verifyButton: 'Verify & Create Account',
    resendButton: 'Resend Code',
    success: 'Account verified! Redirecting...',
    error: 'Something went wrong. Please try again.',
    expired: 'Code expired. Please request a new one.',
    invalidCode: 'Invalid code. Please try again.',
    tooManyAttempts: 'Too many attempts. Please request a new code.',
    loading: 'Loading...'
  }
});
```

---

## Security Features

- **OTP codes**: 12-character alphanumeric, cryptographically generated (non-guessable)
- **HMAC-SHA256 hashing**: OTP codes are hashed with per-OTP salt; plaintext never stored
- **Constant-time verification**: Prevents timing attacks
- **IP/device binding**: Verification rejected if source IP changes
- **Rate limiting**: Per-recipient and per-IP rate limits
- **Max attempts**: 5 attempts before OTP is invalidated
- **TTL**: 5-minute expiry by default
- **Session tokens**: Short-lived token returned after verification

---

## Database Migration

Run the migration to create the required tables:

```bash
psql -d communicator -f backend-rs/migrations/0056_shopify_store_integration.sql
```

This creates:
- `shopify_store_connections` — Store connection configuration
- `shopify_pending_customers` — Pending account creation sessions
- `shopify_webhook_events` — Incoming Shopify webhook log
- `shopify_channel_preferences` — Per-store channel preferences

---

## Troubleshooting

### Widget Not Appearing

1. Check that the script URL is correct and accessible
2. Verify `publicKey`, `apiUrl`, and `storeDomain` are set correctly
3. Check browser console for errors

### OTP Not Being Sent

1. Verify the store connection is active in Communicator
2. Check that at least one channel is enabled in channel preferences
3. Verify WhatsApp/Email/SMS credentials are configured for the brand
4. Check Communicator logs for error messages

### Verification Failing

1. Ensure the OTP code is entered correctly (case-sensitive)
2. Check that the OTP hasn't expired (5-minute default TTL)
3. Verify the customer hasn't exceeded max attempts (5 default)
4. Check that the `pending_id` matches the original account creation request

### CORS Errors

If the widget can't reach the API due to CORS, ensure your Communicator server has CORS configured to allow requests from your Shopify store domain.

---

## Integration with Other Platforms

This same OTP engine works with:
- **WooCommerce**: Use the generic external OTP API
- **Custom e-commerce**: Use the generic external OTP API
- **Any platform**: The API is language-agnostic — just HTTP POST

See `docs/EXTERNAL_OTP_INTEGRATION.md` for the generic integration guide.
