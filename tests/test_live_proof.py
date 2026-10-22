import hashlib
import json

import pytest
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.live_proof import (
    LiveProofStatus,
)
from communicator.services.live_proof import (
    create_live_proof_session,
)


@pytest.mark.asyncio
async def test_create_live_proof_session(db_session: AsyncSession):
    session, url = await create_live_proof_session(
        session=db_session,
        tenant_id="tenant-enterprise-01",
        organization_id="org-pk-south",
        order_id="ORD-9988",
        customer_identifier="+923001234567",
        claim_type="refund",
        max_video_seconds=30,
        ttl_minutes=15,
        base_url="https://verify.newax.io",
    )

    assert session.id is not None
    assert session.tenant_id == "tenant-enterprise-01"
    assert session.organization_id == "org-pk-south"
    assert session.order_id == "ORD-9988"
    assert session.status == LiveProofStatus.PENDING
    assert session.max_video_seconds == 30
    assert len(session.session_token) >= 32
    assert len(session.anti_spoof_nonce) >= 16
    assert f"https://verify.newax.io/verify/proof/{session.id}?token={session.session_token}" == url


@pytest.mark.asyncio
async def test_live_proof_ui_rendering(test_client: AsyncClient, db_session: AsyncSession):
    session, _ = await create_live_proof_session(
        session=db_session,
        tenant_id="tenant-enterprise-01",
        order_id="ORD-1001",
        claim_type="damaged_item",
    )

    # Valid token -> returns live camera HUD
    resp = await test_client.get(f"/verify/proof/{session.id}?token={session.session_token}")
    assert resp.status_code == 200
    assert "Live Product Verification" in resp.text
    assert "Real-Time Live Camera Only" in resp.text
    assert "Order #ORD-1001" in resp.text
    assert "snap-photo-btn" in resp.text
    assert "record-video-btn" in resp.text

    # Invalid token -> returns friendly error page
    bad_resp = await test_client.get(f"/verify/proof/{session.id}?token=invalid-token-here")
    assert bad_resp.status_code == 200
    assert "Session Expired or Unavailable" in bad_resp.text


@pytest.mark.asyncio
async def test_ingest_live_photo_and_video(
    test_client: AsyncClient, db_session: AsyncSession, tmp_path, monkeypatch
):
    monkeypatch.setattr("communicator.config.settings.live_proof_storage_dir", str(tmp_path))

    session, _ = await create_live_proof_session(
        session=db_session,
        tenant_id="tenant-enterprise-01",
        order_id="ORD-2002",
        claim_type="refund",
    )

    # 1. Upload live photo with WebRTC telemetry
    fake_photo_bytes = b"\xff\xd8\xff\xe0\x00\x10JFIF" + b"live_photo_payload_123"
    expected_photo_hash = hashlib.sha256(fake_photo_bytes).hexdigest()

    photo_telemetry = {
        "is_live_stream": True,
        "capture_source": "live_webrtc_camera",
        "facing_mode": "environment",
        "resolution": "1920x1080",
        "frame_rate": 30,
    }

    photo_resp = await test_client.post(
        f"/api/v1/live-proof/sessions/{session.id}/evidence?token={session.session_token}",
        data={
            "media_type": "live_photo",
            "watermark_nonce": session.anti_spoof_nonce,
            "telemetry": json.dumps(photo_telemetry),
        },
        files={"file": ("live_photo.jpg", fake_photo_bytes, "image/jpeg")},
    )

    assert photo_resp.status_code == 200
    photo_data = photo_resp.json()
    assert photo_data["sha256_hash"] == expected_photo_hash
    assert photo_data["media_type"] == "live_photo"
    assert photo_data["watermark_verified"] is True
    assert photo_data["verification_score"] == 1.0

    # 2. Upload live video
    fake_video_bytes = b"\x1a\x45\xdf\xa3" + b"live_video_stream_recording_456"
    expected_video_hash = hashlib.sha256(fake_video_bytes).hexdigest()

    video_telemetry = {
        "is_live_stream": True,
        "capture_source": "live_webrtc_camera",
        "duration_seconds": 12.5,
        "resolution": "1280x720",
    }

    video_resp = await test_client.post(
        f"/api/v1/live-proof/sessions/{session.id}/evidence?token={session.session_token}",
        data={
            "media_type": "live_video",
            "watermark_nonce": session.anti_spoof_nonce,
            "telemetry": json.dumps(video_telemetry),
        },
        files={"file": ("live_video.webm", fake_video_bytes, "video/webm")},
    )

    assert video_resp.status_code == 200
    video_data = video_resp.json()
    assert video_data["sha256_hash"] == expected_video_hash
    assert video_data["media_type"] == "live_video"
    assert video_data["duration_seconds"] == 12.5

    # 3. Retrieve session details
    detail_resp = await test_client.get(
        f"/api/v1/live-proof/sessions/{session.id}?token={session.session_token}"
    )
    assert detail_resp.status_code == 200
    detail = detail_resp.json()
    assert detail["status"] == "captured"
    assert len(detail["evidences"]) == 2

    # 4. Stream media back
    media_resp = await test_client.get(
        f"/api/v1/live-proof/sessions/{session.id}/evidence/{photo_data['id']}/media"
    )
    assert media_resp.status_code == 200
    assert media_resp.content == fake_photo_bytes


@pytest.mark.asyncio
async def test_rejects_gallery_file_upload(
    test_client: AsyncClient, db_session: AsyncSession, tmp_path, monkeypatch
):
    monkeypatch.setattr("communicator.config.settings.live_proof_storage_dir", str(tmp_path))

    session, _ = await create_live_proof_session(
        session=db_session,
        tenant_id="tenant-enterprise-01",
        order_id="ORD-3003",
    )

    # Attempt to upload a regular file without live stream telemetry (e.g. from gallery)
    resp = await test_client.post(
        f"/api/v1/live-proof/sessions/{session.id}/evidence?token={session.session_token}",
        data={
            "media_type": "live_photo",
            "watermark_nonce": session.anti_spoof_nonce,
            "telemetry": json.dumps({"capture_source": "file_picker"}),
        },
        files={"file": ("gallery_pic.jpg", b"fake_gallery_image_bytes", "image/jpeg")},
    )

    assert resp.status_code == 400
    assert "File upload from phone gallery is strictly prohibited" in resp.json()["detail"]
