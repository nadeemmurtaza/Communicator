from __future__ import annotations

import re
import uuid
from html import escape
from typing import Any

# Website tokens are always generated as "web_live_<16 hex chars>" (see
# below) or the "web_token_default" fallback -- never free text -- so any
# value outside this shape is rejected rather than reflected into HTML/JS.
_SAFE_TOKEN_RE = re.compile(r"^[A-Za-z0-9_-]{1,64}$")


def _sanitize_token(token: str) -> str:
    """Prevents reflected XSS: a token embedded raw in HTML/inline-JS must
    not be able to break out of its context (query-string input is
    attacker-controlled)."""
    return token if _SAFE_TOKEN_RE.match(token) else "web_token_default"

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import ChannelType
from communicator.models.inbox import Inbox

DEFAULT_WIDGET_CONFIG: dict[str, Any] = {
    "widget_color": "#1570EF",
    "theme_mode": "light",  # light | dark | system
    "launcher_style": "round_bubble",  # round_bubble | pill_with_text | minimal_tab
    "launcher_icon": "chat_bubble",  # chat_bubble | sparkle_ai | headset_support | envelope
    "launcher_text": "Chat with us",
    "widget_position": "right",  # right | left
    "greeting_title": "Hi there! 👋",
    "greeting_tagline": "We typically reply in under 2 minutes.",
    "avatar_url": "https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=128&h=128&fit=crop",
    "border_radius": "rounded_2xl",  # rounded_lg (12px) | rounded_2xl (16px) | rounded_full (24px)
    "play_sound": True,
    "show_branding": True,
    "allowed_domains": "bayworks.com, localhost, *.bayworks.com",
    "pre_chat_form_enabled": True,
    "identity_validation": False,
    "hmac_secret": "",
}


async def get_or_create_website_inbox(session: AsyncSession) -> Inbox:
    stmt = select(Inbox).where(Inbox.channel_type == ChannelType.WEBSITE)
    inbox = (await session.execute(stmt)).scalars().first()
    if not inbox:
        token = f"web_live_{uuid.uuid4().hex[:16]}"
        inbox = Inbox(
            id=uuid.uuid4(),
            name="Website Live Chat",
            channel_type=ChannelType.WEBSITE,
            greeting_enabled=True,
            greeting_message="👋 Welcome to our live chat! How can our support team help you today?",
            credentials={
                "websiteToken": token,
                "websiteDomain": "bayworks.com",
                **DEFAULT_WIDGET_CONFIG,
            },
        )
        session.add(inbox)
        await session.commit()
        await session.refresh(inbox)
    return inbox


def generate_embed_codes(token: str, base_url: str, styling: dict[str, Any]) -> dict[str, str]:
    token = _sanitize_token(token)
    color = styling.get("widget_color", "#1570EF")
    pos = styling.get("widget_position", "right")

    html_script = f"""<!-- Captain AI / Newax Live Chat Embed Code -->
<script>
  (function(d, t) {{
    var BASE_URL = "{base_url}";
    var g = d.createElement(t), s = d.getElementsByTagName(t)[0];
    g.src = BASE_URL + "/packs/js/sdk.js";
    g.defer = true;
    g.async = true;
    s.parentNode.insertBefore(g, s);
    g.onload = function() {{
      window.captainChatSettings = {{
        websiteToken: "{token}",
        baseUrl: BASE_URL,
        color: "{color}",
        position: "{pos}"
      }};
      window.captainChat.init();
    }};
  }})(document, "script");
</script>"""

    react_snippet = f"""// React / Next.js Component
import {{ useEffect }} from 'react';

export function LiveChatWidget() {{
  useEffect(() => {{
    const script = document.createElement('script');
    script.src = "{base_url}/packs/js/sdk.js";
    script.async = true;
    script.onload = () => {{
      window.captainChatSettings = {{
        websiteToken: "{token}",
        baseUrl: "{base_url}",
        color: "{color}",
        position: "{pos}"
      }};
      window.captainChat?.init();
    }};
    document.body.appendChild(script);
  }}, []);

  return null;
}}"""

    gtm_snippet = f"""<!-- Google Tag Manager (Custom HTML Tag) -->
<script>
  window.captainChatSettings = {{
    websiteToken: "{token}",
    baseUrl: "{base_url}",
    color: "{color}",
    position: "{pos}"
  }};
  var s = document.createElement('script');
  s.src = "{base_url}/packs/js/sdk.js";
  s.async = true;
  document.head.appendChild(s);
</script>"""

    return {
        "html_script": html_script,
        "react_snippet": react_snippet,
        "gtm_snippet": gtm_snippet,
    }


def generate_preview_page_html(token: str, base_url: str, styling: dict[str, Any]) -> str:
    token = _sanitize_token(token)
    color = styling.get("widget_color", "#1570EF")
    pos = styling.get("widget_position", "right")
    greeting_title = escape(str(styling.get("greeting_title", "Hi there! 👋")))
    greeting_tagline = escape(
        str(styling.get("greeting_tagline", "We typically reply in under 2 minutes."))
    )
    launcher_text = escape(str(styling.get("launcher_text", "Chat with us")))
    launcher_style = styling.get("launcher_style", "round_bubble")

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Live Chat Widget Test Environment</title>
  <style>
    body {{
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      margin: 0;
      padding: 0;
      background: #f8fafc;
      color: #0f172a;
    }}
    .hero {{
      max-width: 900px;
      margin: 60px auto;
      padding: 40px;
      background: #ffffff;
      border-radius: 16px;
      box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
      border: 1px solid #e2e8f0;
      text-align: center;
    }}
    h1 {{ font-size: 2rem; margin-bottom: 12px; }}
    p {{ color: #64748b; font-size: 1.1rem; line-height: 1.6; max-width: 650px; margin: 0 auto 24px; }}
    .badge {{
      display: inline-block;
      padding: 4px 12px;
      border-radius: 9999px;
      background: #eff8ff;
      color: #175cd3;
      font-weight: 600;
      font-size: 0.875rem;
      margin-bottom: 16px;
    }}
    .status-card {{
      display: inline-flex;
      align-items: center;
      gap: 8px;
      padding: 8px 16px;
      border-radius: 8px;
      background: #ecfdf5;
      color: #065f46;
      font-size: 0.875rem;
      font-weight: 500;
    }}
  </style>
</head>
<body>
  <div class="hero">
    <div class="badge">Live Chat Widget Playground</div>
    <h1>Your Website Storefront Preview</h1>
    <p>This is a live test environment demonstrating your customized embeddable chat widget. Notice the floating launcher at the bottom {pos}.</p>
    <div class="status-card">
      <span>●</span>
      <span>Widget Token: <code>{token}</code></span>
    </div>
  </div>

  <!-- Interactive Mock Chat Widget Script -->
  <script>
    window.captainChatSettings = {{
      websiteToken: "{token}",
      baseUrl: "{base_url}",
      color: "{color}",
      position: "{pos}",
      greetingTitle: "{greeting_title}",
      greetingTagline: "{greeting_tagline}",
      launcherText: "{launcher_text}",
      launcherStyle: "{launcher_style}"
    }};
  </script>
</body>
</html>"""


async def get_customizer_config(
    session: AsyncSession, base_url: str = "http://localhost:8000"
) -> dict[str, Any]:
    inbox = await get_or_create_website_inbox(session)
    creds = inbox.credentials or {}
    token = creds.get("websiteToken") or "web_token_default"

    styling = {
        "widget_color": creds.get("widgetColor", DEFAULT_WIDGET_CONFIG["widget_color"]),
        "theme_mode": creds.get("themeMode", DEFAULT_WIDGET_CONFIG["theme_mode"]),
        "launcher_style": creds.get("launcherStyle", DEFAULT_WIDGET_CONFIG["launcher_style"]),
        "launcher_icon": creds.get("launcherIcon", DEFAULT_WIDGET_CONFIG["launcher_icon"]),
        "launcher_text": creds.get("launcherText", DEFAULT_WIDGET_CONFIG["launcher_text"]),
        "widget_position": creds.get("widgetPosition", DEFAULT_WIDGET_CONFIG["widget_position"]),
        "greeting_title": creds.get("greetingTitle", DEFAULT_WIDGET_CONFIG["greeting_title"]),
        "greeting_tagline": creds.get("greetingTagline", DEFAULT_WIDGET_CONFIG["greeting_tagline"]),
        "avatar_url": creds.get("avatarUrl", DEFAULT_WIDGET_CONFIG["avatar_url"]),
        "border_radius": creds.get("borderRadius", DEFAULT_WIDGET_CONFIG["border_radius"]),
        "play_sound": creds.get("playSound", DEFAULT_WIDGET_CONFIG["play_sound"]),
        "show_branding": creds.get("showBranding", DEFAULT_WIDGET_CONFIG["show_branding"]),
        "allowed_domains": creds.get("allowedDomains", DEFAULT_WIDGET_CONFIG["allowed_domains"]),
        "pre_chat_form_enabled": creds.get(
            "preChatFormEnabled", DEFAULT_WIDGET_CONFIG["pre_chat_form_enabled"]
        ),
        "identity_validation": creds.get(
            "identityValidation", DEFAULT_WIDGET_CONFIG["identity_validation"]
        ),
        "hmac_secret": creds.get("hmacSecret", DEFAULT_WIDGET_CONFIG["hmac_secret"]),
    }

    embed_codes = generate_embed_codes(token, base_url, styling)

    return {
        "inbox_id": str(inbox.id),
        "inbox_name": inbox.name,
        "website_token": token,
        "website_domain": creds.get("websiteDomain", "bayworks.com"),
        "styling": styling,
        "embed_codes": embed_codes,
        "preview_url": f"/api/v1/widget/preview-page?token={token}",
    }


async def update_customizer_config(
    session: AsyncSession, payload: dict[str, Any], base_url: str = "http://localhost:8000"
) -> dict[str, Any]:
    inbox = await get_or_create_website_inbox(session)
    creds = dict(inbox.credentials or {})

    # Map snake_case payload to credentials
    mapping = {
        "widget_color": "widgetColor",
        "theme_mode": "themeMode",
        "launcher_style": "launcherStyle",
        "launcher_icon": "launcherIcon",
        "launcher_text": "launcherText",
        "widget_position": "widgetPosition",
        "greeting_title": "greetingTitle",
        "greeting_tagline": "greetingTagline",
        "avatar_url": "avatarUrl",
        "border_radius": "borderRadius",
        "play_sound": "playSound",
        "show_branding": "showBranding",
        "allowed_domains": "allowedDomains",
        "pre_chat_form_enabled": "preChatFormEnabled",
        "identity_validation": "identityValidation",
        "hmac_secret": "hmacSecret",
        "website_domain": "websiteDomain",
    }

    for key, cred_key in mapping.items():
        if key in payload:
            creds[cred_key] = payload[key]

    if "inbox_name" in payload:
        inbox.name = payload["inbox_name"]
    if "avatar_url" in payload:
        inbox.avatar_url = payload["avatar_url"]

    inbox.credentials = creds
    await session.commit()
    await session.refresh(inbox)

    return await get_customizer_config(session, base_url=base_url)


async def reset_customizer_defaults(
    session: AsyncSession, base_url: str = "http://localhost:8000"
) -> dict[str, Any]:
    return await update_customizer_config(session, DEFAULT_WIDGET_CONFIG, base_url=base_url)
