import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.services import widget_customizer_service


@pytest.fixture
def anyio_backend():
    return "asyncio"


@pytest.mark.asyncio
async def test_get_and_update_customizer_config(db_session):
    # 1. Get default config
    cfg = await widget_customizer_service.get_customizer_config(db_session, base_url="https://app.test.com")
    assert cfg is not None
    assert "styling" in cfg
    assert cfg["styling"]["widget_color"] == "#1570EF"
    assert "embed_codes" in cfg
    assert "html_script" in cfg["embed_codes"]
    assert "react_snippet" in cfg["embed_codes"]
    assert "gtm_snippet" in cfg["embed_codes"]

    # 2. Update styling
    updated = await widget_customizer_service.update_customizer_config(
        db_session,
        {
            "widget_color": "#7F56D9",
            "launcher_style": "pill_with_text",
            "launcher_text": "Need help? Ask Captain AI",
            "launcher_icon": "sparkle_ai",
            "theme_mode": "dark",
            "widget_position": "left",
            "greeting_title": "Welcome to Pro Support! ✨",
            "allowed_domains": "test.com, app.test.com",
        },
        base_url="https://app.test.com",
    )
    assert updated["styling"]["widget_color"] == "#7F56D9"
    assert updated["styling"]["launcher_style"] == "pill_with_text"
    assert updated["styling"]["launcher_text"] == "Need help? Ask Captain AI"
    assert updated["styling"]["theme_mode"] == "dark"
    assert updated["styling"]["widget_position"] == "left"
    assert "#7F56D9" in updated["embed_codes"]["html_script"]
    assert "left" in updated["embed_codes"]["html_script"]


@pytest.mark.asyncio
async def test_reset_customizer_defaults(db_session):
    # Change color
    await widget_customizer_service.update_customizer_config(
        db_session, {"widget_color": "#E11D48"}, base_url="https://app.test.com"
    )

    # Reset
    reset_cfg = await widget_customizer_service.reset_customizer_defaults(
        db_session, base_url="https://app.test.com"
    )
    assert reset_cfg["styling"]["widget_color"] == "#1570EF"
    assert reset_cfg["styling"]["launcher_style"] == "round_bubble"


@pytest.mark.asyncio
async def test_generate_preview_html():
    html = widget_customizer_service.generate_preview_page_html(
        token="web_live_test_tok",
        base_url="https://app.test.com",
        styling={
            "widget_color": "#10B981",
            "widget_position": "right",
            "launcher_text": "Live Support",
            "launcher_style": "round_bubble",
        },
    )
    assert "<!DOCTYPE html>" in html
    assert "web_live_test_tok" in html
    assert "#10B981" in html
    assert "window.captainChatSettings" in html


@pytest.mark.asyncio
async def test_customizer_api_endpoints(db_session, override_database_dependency):
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. GET /api/v1/widget/customizer
        res = await client.get("/api/v1/widget/customizer")
        assert res.status_code == 200
        data = res.json()
        assert "website_token" in data
        assert "styling" in data

        # 2. POST /api/v1/widget/customizer
        res = await client.post(
            "/api/v1/widget/customizer",
            json={
                "widget_color": "#039855",
                "launcher_style": "pill_with_text",
                "launcher_text": "Chat with our team",
            },
        )
        assert res.status_code == 200
        updated = res.json()
        assert updated["styling"]["widget_color"] == "#039855"
        assert updated["styling"]["launcher_style"] == "pill_with_text"

        # 3. GET /api/v1/widget/customizer/embed-code
        res = await client.get("/api/v1/widget/customizer/embed-code")
        assert res.status_code == 200
        codes = res.json()
        assert "html_script" in codes["embed_codes"]
        assert "react_snippet" in codes["embed_codes"]
        assert "gtm_snippet" in codes["embed_codes"]

        # 4. GET /api/v1/widget/preview-page
        res = await client.get("/api/v1/widget/preview-page")
        assert res.status_code == 200
        assert "text/html" in res.headers["content-type"]
        assert "Live Chat Widget Playground" in res.text

        # 5. Public /api/v1/widget/config includes extended customizer attributes
        token = data["website_token"]
        res = await client.get(f"/api/v1/widget/config?token={token}")
        assert res.status_code == 200
        pub_cfg = res.json()
        assert "launcherStyle" in pub_cfg
        assert "launcherText" in pub_cfg
        assert "themeMode" in pub_cfg
        assert "borderRadius" in pub_cfg
        assert "showBranding" in pub_cfg
