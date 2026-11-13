# Known Limitations & Production Readiness

This document tracks features that are implemented but use simulated/stub responses instead of real external API integration, as well as deferred features from the original Intercom gap analysis.

## Simulated Implementations (Need Production Integration)

These features have full backend routes, database tables, and frontend dashboards, but the actual external API calls are simulated. They need production integration to work with real services.

| Feature | Phase | What's Simulated | Production Integration Needed |
|---------|-------|-----------------|-------------------------------|
| CRM Sync (Salesforce/HubSpot/Pipedrive) | 22 | Sync operations return simulated results | OAuth flow, real API calls to Salesforce/HubSpot/Pipedrive REST APIs |
| MCP Protocol tool calling | 28 | Tool discovery infers from URL patterns; tool calling simulates responses | Real MCP server connections via stdio or HTTP transport |
| Procedure action execution | 18 | HTTP/Stripe/internal API calls return simulated responses | Real HTTP client with auth headers for Stripe, internal APIs |
| Workflow step execution | 20 | Only `assign` step updates DB; other steps (send_message, notify, create_ticket) simulate | Email/SMS/webhook integration for each step type |
| Scheduled task execution | 40 | Task execution simulates responses | Real handler dispatch for each task type (metric collection, alert evaluation, etc.) |
| Broadcast sending | 46 | One-click send creates records but simulates actual channel delivery | Real API calls to WhatsApp, Email, SMS, Instagram, Facebook, Telegram, Discord, Slack |
| Voice channel | 13 | Routes exist but Twilio integration is stub | Real Twilio Voice API integration with WebRTC |
| CSAT survey sending | 15 | Survey send simulates delivery | Real email/SMS/WhatsApp delivery for survey links |
| Notification broadcast | 31 | WebSocket registration works but broadcast simulates delivery | Real WebSocket upgrade handler and message fan-out |
| ERP Fulfillment | 39 | Shipping rate calculation and manifest dispatch simulate | Real carrier API integration (FedEx, UPS, DHL) |

## Deferred Features

These features from the original Intercom research are intentionally deferred as premature optimization.

| Feature | Reason | Effort | Impact |
|---------|--------|--------|--------|
| Custom-Trained Models | Using generic Claude LLM is sufficient for current scale; custom models needed only at enterprise scale | Very High | Low — premature optimization |
| Real-time WebSocket Upgrade Handler | Axum WS feature is enabled in Cargo.toml; handler not yet implemented | Medium | Medium — needed for real-time notifications |
| Background Worker Daemon | Scheduled tasks are triggered on-demand via API; no cron-driven background process | Medium | Medium — needed for truly automated recurring tasks |
| PDF Extraction Quality | Uses `lopdf` for basic PDF parsing; production may need `pdf-extract` or similar | Low | Low — current extraction works for simple PDFs |
| DOCX Parsing | Basic ZIP+XML extraction; production may want a dedicated library | Low | Low — current extraction works for simple DOCX files |

## HTML-to-Channel Conversion Limitations

The HTML auto-formatting in `omni_broadcast.rs` uses rule-based regex conversion without an HTML parser:

- **Handles:** `<b>`, `<strong>`, `<i>`, `<em>`, `<s>`, `<del>`, `<code>`, `<a>`, block elements, HTML entities
- **Doesn't handle:** `<table>`, `<ul>/<ol>`, `<img>` extraction, deeply nested tags, CSS classes
- **Recommendation:** For production, use a proper HTML parser (e.g., `html5ever`) instead of regex-based tag stripping

## No Integration Tests

The codebase has unit tests (serialization, prompt building, heuristic categorization) but no integration tests. Consider adding:

- End-to-end broadcast flow tests
- Migration application order tests
- Route handler response tests
- Database constraint enforcement tests

## Frontend Pages Without Dedicated Dashboards

The following backend routes exist but don't have dedicated frontend dashboard pages. They are managed through existing pages or API-only:

| Backend Route | Managed Through |
|--------------|-----------------|
| `/otp/*` | OTP engine is used programmatically during auth flows |
| `/external-otp/*` | External OTP integration is configured via API |
| `/shopify/*` | Shopify store management via Commerce dashboard |
| `/store/*` | Store connection management via Commerce dashboard |
| `/nid/tenants/*` | Tenant dashboard at `/tenants` |
| `/nid/org/*` | Organization sync via System Health dashboard |
| `/nid/brand/*` | Brand sync via Brand Profile dashboard |
