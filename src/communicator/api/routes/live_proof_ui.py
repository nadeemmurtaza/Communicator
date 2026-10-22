from __future__ import annotations

import uuid
from typing import Annotated

from fastapi import APIRouter, Depends, HTTPException, Query, Request
from fastapi.responses import HTMLResponse
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.live_proof import LiveProofStatus
from communicator.services.live_proof import get_and_validate_session

router = APIRouter(tags=["live-proof-ui"])


def render_error_page(title: str, message: str) -> HTMLResponse:
    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
  <title>{title} - LiveProof</title>
  <style>
    :root {{
      --bg: #0f172a;
      --card: #1e293b;
      --border: #334155;
      --text: #f8fafc;
      --muted: #94a3b8;
      --danger: #ef4444;
      --accent: #3b82f6;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      background: var(--bg);
      color: var(--text);
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      min-height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 20px;
    }}
    .card {{
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: 16px;
      max-width: 480px;
      width: 100%;
      padding: 32px 24px;
      text-align: center;
      box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5);
    }}
    .icon {{ font-size: 48px; margin-bottom: 16px; }}
    h1 {{ font-size: 20px; margin-bottom: 12px; font-weight: 600; }}
    p {{ color: var(--muted); font-size: 15px; line-height: 1.5; margin-bottom: 24px; }}
    .badge {{
      display: inline-block;
      background: rgba(239, 68, 68, 0.15);
      color: var(--danger);
      padding: 6px 14px;
      border-radius: 9999px;
      font-size: 13px;
      font-weight: 500;
      margin-bottom: 16px;
    }}
  </style>
</head>
<body>
  <div class="card">
    <div class="icon">🔒</div>
    <div class="badge">Verification Notice</div>
    <h1>{title}</h1>
    <p>{message}</p>
  </div>
</body>
</html>"""
    return HTMLResponse(content=html, status_code=200)


@router.get("/verify/proof/{session_id}", response_class=HTMLResponse)
async def live_proof_capture_page(
    session_id: uuid.UUID,
    token: Annotated[str, Query(description="Security token")],
    request: Request,
    db: AsyncSession = Depends(get_db_session),
) -> HTMLResponse:
    """
    Renders the Verilo-style mobile-first live camera capture application.
    Enforces real-time camera stream only.
    Strictly forbids photo gallery / file uploads.
    """
    try:
        proof_session = await get_and_validate_session(db, session_id, token)
    except (HTTPException, ValueError) as exc:
        detail = getattr(exc, "detail", str(exc))
        return render_error_page("Session Expired or Unavailable", str(detail))

    if proof_session.status == LiveProofStatus.CAPTURED:
        return render_error_page(
            "Proof Already Submitted",
            f"Live evidence for order #{proof_session.order_id or 'claim'} has already been captured and is being processed by the support team.",
        )

    order_ref = proof_session.order_id or "Claim"
    tenant_name = proof_session.tenant_id
    anti_spoof_nonce = proof_session.anti_spoof_nonce
    max_video_seconds = proof_session.max_video_seconds

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
  <title>Live Product Verification - {tenant_name}</title>
  <style>
    :root {{
      --bg: #090d16;
      --card: #131b2e;
      --border: #23314f;
      --primary: #2563eb;
      --primary-hover: #1d4ed8;
      --accent: #10b981;
      --danger: #ef4444;
      --text: #f1f5f9;
      --muted: #94a3b8;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; user-select: none; }}
    body {{
      background: var(--bg);
      color: var(--text);
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      min-height: 100vh;
      display: flex;
      flex-direction: column;
      overflow-x: hidden;
    }}
    header {{
      background: rgba(19, 27, 46, 0.85);
      backdrop-filter: blur(12px);
      border-bottom: 1px solid var(--border);
      padding: 14px 20px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      position: sticky;
      top: 0;
      z-index: 50;
    }}
    .brand {{ font-weight: 700; font-size: 16px; letter-spacing: -0.5px; display: flex; align-items: center; gap: 8px; }}
    .brand-tag {{
      background: rgba(37, 99, 235, 0.2);
      color: #60a5fa;
      font-size: 10px;
      text-transform: uppercase;
      padding: 2px 8px;
      border-radius: 4px;
      font-weight: 600;
    }}
    .pill {{
      font-size: 12px;
      background: #1e293b;
      padding: 4px 10px;
      border-radius: 9999px;
      border: 1px solid var(--border);
      color: var(--muted);
    }}
    main {{
      flex: 1;
      display: flex;
      flex-direction: column;
      max-width: 600px;
      margin: 0 auto;
      width: 100%;
      padding: 16px;
      gap: 16px;
    }}
    .notice {{
      background: rgba(37, 99, 235, 0.1);
      border: 1px solid rgba(37, 99, 235, 0.3);
      border-radius: 12px;
      padding: 12px 16px;
      font-size: 13px;
      line-height: 1.4;
      color: #93c5fd;
      display: flex;
      align-items: flex-start;
      gap: 10px;
    }}
    .camera-container {{
      position: relative;
      background: #000;
      border-radius: 16px;
      overflow: hidden;
      aspect-ratio: 3/4;
      width: 100%;
      display: flex;
      align-items: center;
      justify-content: center;
      border: 2px solid var(--border);
      box-shadow: 0 10px 25px rgba(0,0,0,0.6);
    }}
    video {{
      width: 100%;
      height: 100%;
      object-fit: cover;
    }}
    .hud-overlay {{
      position: absolute;
      top: 0; left: 0; right: 0; bottom: 0;
      pointer-events: none;
      display: flex;
      flex-direction: column;
      justify-content: space-between;
      padding: 14px;
      background: linear-gradient(180deg, rgba(0,0,0,0.6) 0%, transparent 20%, transparent 80%, rgba(0,0,0,0.8) 100%);
    }}
    .hud-top {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      font-family: monospace;
      font-size: 11px;
      color: #10b981;
      text-shadow: 0 1px 3px rgba(0,0,0,0.9);
    }}
    .watermark-banner {{
      background: rgba(0, 0, 0, 0.65);
      border: 1px solid rgba(16, 185, 129, 0.4);
      padding: 4px 10px;
      border-radius: 6px;
      display: flex;
      align-items: center;
      gap: 6px;
    }}
    .pulse-dot {{
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: #ef4444;
      animation: blink 1s infinite;
    }}
    @keyframes blink {{
      0%, 100% {{ opacity: 1; }}
      50% {{ opacity: 0.3; }}
    }}
    .hud-center-guide {{
      position: absolute;
      top: 50%; left: 50%;
      transform: translate(-50%, -50%);
      width: 75%;
      height: 60%;
      border: 1.5px dashed rgba(255, 255, 255, 0.3);
      border-radius: 12px;
      display: flex;
      align-items: center;
      justify-content: center;
      pointer-events: none;
    }}
    .hud-guide-text {{
      color: rgba(255, 255, 255, 0.6);
      font-size: 12px;
      text-align: center;
      padding: 6px 12px;
      background: rgba(0, 0, 0, 0.4);
      border-radius: 6px;
    }}
    .controls-panel {{
      display: flex;
      flex-direction: column;
      gap: 12px;
    }}
    .capture-buttons {{
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 12px;
    }}
    .btn {{
      padding: 14px 20px;
      border-radius: 12px;
      font-size: 15px;
      font-weight: 600;
      border: none;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 8px;
      transition: all 0.2s ease;
    }}
    .btn-photo {{
      background: #ffffff;
      color: #0f172a;
    }}
    .btn-photo:active {{ background: #e2e8f0; }}
    .btn-video {{
      background: var(--danger);
      color: #ffffff;
    }}
    .btn-video:active {{ background: #dc2626; }}
    .btn-switch {{
      background: var(--card);
      color: var(--text);
      border: 1px solid var(--border);
      padding: 10px;
      font-size: 13px;
    }}
    .preview-modal {{
      display: none;
      position: fixed;
      top: 0; left: 0; right: 0; bottom: 0;
      background: rgba(0,0,0,0.92);
      z-index: 100;
      flex-direction: column;
      padding: 20px;
    }}
    .preview-body {{
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      margin-bottom: 20px;
    }}
    .preview-media {{
      max-width: 100%;
      max-height: 70vh;
      border-radius: 12px;
      box-shadow: 0 10px 30px rgba(0,0,0,0.8);
    }}
    .success-card {{
      display: none;
      background: var(--card);
      border: 1px solid var(--accent);
      border-radius: 16px;
      padding: 32px 20px;
      text-align: center;
      animation: fadeIn 0.4s ease;
    }}
    .receipt-box {{
      background: #090d16;
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 14px;
      margin: 16px 0;
      font-family: monospace;
      font-size: 12px;
      color: #38bdf8;
      text-align: left;
      word-break: break-all;
    }}
    @keyframes fadeIn {{
      from {{ opacity: 0; transform: translateY(10px); }}
      to {{ opacity: 1; transform: translateY(0); }}
    }}
  </style>
</head>
<body>
  <header>
    <div class="brand">
      <span>🛡️ LiveProof</span>
      <span class="brand-tag">{tenant_name}</span>
    </div>
    <div class="pill">Order #{order_ref}</div>
  </header>

  <main id="capture-interface">
    <div class="notice">
      <span>🔒</span>
      <div>
        <strong>Real-Time Live Camera Only</strong><br>
        To prevent return fraud, this system streams directly from your device camera. Pre-recorded gallery uploads are strictly disabled.
      </div>
    </div>

    <div class="camera-container" id="camera-box">
      <video id="live-video" autoplay playsinline muted></video>
      <div class="hud-overlay">
        <div class="hud-top">
          <div class="watermark-banner">
            <span class="pulse-dot"></span>
            <span>LIVE CAM • <span id="clock-display">00:00:00.000</span></span>
          </div>
          <div class="watermark-banner" id="rec-indicator" style="display: none; color: #ef4444; border-color: #ef4444;">
            <span class="pulse-dot"></span>
            <span id="rec-timer">00:00</span>
          </div>
        </div>
        <div class="hud-center-guide">
          <div class="hud-guide-text">Position damaged item & barcode in frame</div>
        </div>
      </div>
    </div>

    <div class="controls-panel">
      <div class="capture-buttons">
        <button id="snap-photo-btn" class="btn btn-photo">
          📸 Take Live Photo
        </button>
        <button id="record-video-btn" class="btn btn-video">
          🎥 Record Live Video
        </button>
      </div>
      <button id="switch-cam-btn" class="btn btn-switch">
        🔄 Flip Camera
      </button>
    </div>
  </main>

  <!-- Review Modal -->
  <div class="preview-modal" id="preview-modal">
    <h2 style="font-size: 18px; margin-bottom: 12px; text-align: center;">Review Live Capture</h2>
    <div class="preview-body" id="preview-container"></div>
    <div style="display: flex; gap: 12px;">
      <button id="retake-btn" class="btn btn-switch" style="flex: 1;">Retake</button>
      <button id="submit-proof-btn" class="btn btn-photo" style="flex: 2; background: var(--accent); color: white;">
        ✅ Submit Live Proof
      </button>
    </div>
  </div>

  <!-- Success View -->
  <div id="success-view" class="card success-card" style="margin: 20px;">
    <div style="font-size: 48px; margin-bottom: 16px;">🎉</div>
    <h1 style="font-size: 22px; margin-bottom: 8px;">Proof Verified & Submitted</h1>
    <p style="color: var(--muted); font-size: 14px; line-height: 1.5;">
      Your live camera capture has been cryptographically signed and submitted for your claim on order #{order_ref}.
    </p>
    <div class="receipt-box" id="receipt-details">
      <!-- SHA-256 and proof receipt populated via JS -->
    </div>
    <p style="font-size: 13px; color: #94a3b8;">
      The AI support agent in your chat session has been notified and will proceed with your refund immediately. You may close this tab.
    </p>
  </div>

  <canvas id="hidden-canvas" style="display: none;"></canvas>

  <script>
    const SESSION_ID = "{session_id}";
    const TOKEN = "{token}";
    const NONCE = "{anti_spoof_nonce}";
    const MAX_REC_SECONDS = {max_video_seconds};

    let stream = null;
    let facingMode = 'environment';
    let mediaRecorder = null;
    let recordedChunks = [];
    let isRecording = false;
    let recInterval = null;
    let recSeconds = 0;
    let activeMediaBlob = null;
    let activeMediaType = null; // 'live_photo' or 'live_video'

    const videoEl = document.getElementById('live-video');
    const clockEl = document.getElementById('clock-display');
    const recIndicator = document.getElementById('rec-indicator');
    const recTimer = document.getElementById('rec-timer');
    const hiddenCanvas = document.getElementById('hidden-canvas');
    const previewModal = document.getElementById('preview-modal');
    const previewContainer = document.getElementById('preview-container');

    // Live clock HUD
    function updateClock() {{
      const d = new Date();
      const pad = (n, s=2) => String(n).padStart(s, '0');
      clockEl.textContent = `${{pad(d.getUTCHours())}}:${{pad(d.getUTCMinutes())}}:${{pad(d.getUTCSeconds())}}.${{pad(d.getUTCMilliseconds(), 3)}} UTC`;
      requestAnimationFrame(updateClock);
    }}
    requestAnimationFrame(updateClock);

    // Initialize Camera Stream
    async function startCamera() {{
      if (stream) {{
        stream.getTracks().forEach(t => t.stop());
      }}
      try {{
        const constraints = {{
          video: {{
            facingMode: {{ ideal: facingMode }},
            width: {{ ideal: 1920 }},
            height: {{ ideal: 1080 }}
          }},
          audio: true
        }};
        stream = await navigator.mediaDevices.getUserMedia(constraints);
        videoEl.srcObject = stream;
      }} catch (err) {{
        console.error("Camera access failed", err);
        alert("Camera permission is required for live return verification. Please enable camera access.");
      }}
    }}
    startCamera();

    // Flip Camera
    document.getElementById('switch-cam-btn').addEventListener('click', () => {{
      facingMode = facingMode === 'environment' ? 'user' : 'environment';
      startCamera();
    }});

    // Capture Live Photo
    document.getElementById('snap-photo-btn').addEventListener('click', () => {{
      if (!stream) return;
      const track = stream.getVideoTracks()[0];
      const settings = track.getSettings();
      const w = videoEl.videoWidth || 1280;
      const h = videoEl.videoHeight || 720;

      hiddenCanvas.width = w;
      hiddenCanvas.height = h;
      const ctx = hiddenCanvas.getContext('2d');
      ctx.drawImage(videoEl, 0, 0, w, h);

      // Add hard live watermark onto frame
      ctx.fillStyle = 'rgba(0,0,0,0.6)';
      ctx.fillRect(0, h - 36, w, 36);
      ctx.fillStyle = '#10b981';
      ctx.font = '14px monospace';
      ctx.fillText(`VERIFIED LIVE • SESSION: ${{SESSION_ID.slice(0,8)}} • ${{new Date().toISOString()}}`, 16, h - 14);

      hiddenCanvas.toBlob((blob) => {{
        activeMediaBlob = blob;
        activeMediaType = 'live_photo';
        showPreview('photo', URL.createObjectURL(blob));
      }}, 'image/jpeg', 0.92);
    }});

    // Record Live Video
    const recBtn = document.getElementById('record-video-btn');
    recBtn.addEventListener('click', () => {{
      if (!isRecording) {{
        startRecording();
      }} else {{
        stopRecording();
      }}
    }});

    function startRecording() {{
      if (!stream) return;
      recordedChunks = [];
      recSeconds = 0;
      const options = {{ mimeType: 'video/webm;codecs=vp8,opus' }};
      try {{
        mediaRecorder = new MediaRecorder(stream, options);
      }} catch (e) {{
        mediaRecorder = new MediaRecorder(stream);
      }}

      mediaRecorder.ondataavailable = (e) => {{
        if (e.data && e.data.size > 0) {{
          recordedChunks.push(e.data);
        }}
      }};

      mediaRecorder.onstop = () => {{
        activeMediaBlob = new Blob(recordedChunks, {{ type: mediaRecorder.mimeType || 'video/webm' }});
        activeMediaType = 'live_video';
        showPreview('video', URL.createObjectURL(activeMediaBlob));
      }};

      mediaRecorder.start(1000);
      isRecording = true;
      recBtn.textContent = '⏹️ Stop Recording';
      recBtn.style.background = '#dc2626';
      recIndicator.style.display = 'flex';

      recInterval = setInterval(() => {{
        recSeconds++;
        const mins = String(Math.floor(recSeconds / 60)).padStart(2, '0');
        const secs = String(recSeconds % 60).padStart(2, '0');
        recTimer.textContent = `${{mins}}:${{secs}}`;

        if (recSeconds >= MAX_REC_SECONDS) {{
          stopRecording();
        }}
      }}, 1000);
    }}

    function stopRecording() {{
      if (!isRecording) return;
      isRecording = false;
      clearInterval(recInterval);
      mediaRecorder.stop();
      recBtn.textContent = '🎥 Record Live Video';
      recBtn.style.background = 'var(--danger)';
      recIndicator.style.display = 'none';
    }}

    // Preview
    function showPreview(type, url) {{
      previewContainer.innerHTML = '';
      if (type === 'photo') {{
        const img = document.createElement('img');
        img.src = url;
        img.className = 'preview-media';
        previewContainer.appendChild(img);
      }} else {{
        const vid = document.createElement('video');
        vid.src = url;
        vid.controls = true;
        vid.autoplay = true;
        vid.className = 'preview-media';
        previewContainer.appendChild(vid);
      }}
      previewModal.style.display = 'flex';
    }}

    document.getElementById('retake-btn').addEventListener('click', () => {{
      previewModal.style.display = 'none';
      activeMediaBlob = null;
      activeMediaType = null;
    }});

    // Submit Proof
    const submitBtn = document.getElementById('submit-proof-btn');
    submitBtn.addEventListener('click', async () => {{
      if (!activeMediaBlob || !activeMediaType) return;

      submitBtn.disabled = true;
      submitBtn.textContent = '⏳ Verifying & Uploading...';

      const formData = new FormData();
      formData.append('file', activeMediaBlob, activeMediaType === 'live_photo' ? 'live_capture.jpg' : 'live_capture.webm');
      formData.append('media_type', activeMediaType);
      formData.append('watermark_nonce', NONCE);

      const track = stream ? stream.getVideoTracks()[0] : null;
      const settings = track ? track.getSettings() : {{}};
      const telemetry = {{
        is_live_stream: true,
        capture_source: 'live_webrtc_camera',
        facing_mode: facingMode,
        resolution: `${{settings.width || videoEl.videoWidth || 1280}}x${{settings.height || videoEl.videoHeight || 720}}`,
        frame_rate: settings.frameRate || 30,
        duration_seconds: activeMediaType === 'live_video' ? recSeconds : null,
        client_timestamp: new Date().toISOString()
      }};
      formData.append('telemetry', JSON.stringify(telemetry));

      try {{
        const url = `/api/v1/live-proof/sessions/${{SESSION_ID}}/evidence?token=${{TOKEN}}`;
        const res = await fetch(url, {{
          method: 'POST',
          body: formData
        }});

        if (!res.ok) {{
          const errData = await res.json().catch(() => ({{}}));
          throw new Error(errData.detail || 'Upload failed');
        }}

        const data = await res.json();

        // Show Success
        previewModal.style.display = 'none';
        document.getElementById('capture-interface').style.display = 'none';
        const receipt = document.getElementById('receipt-details');
        receipt.innerHTML = `
          <strong>STATUS:</strong> VERIFIED LIVE PROOF<br>
          <strong>SHA-256 HASH:</strong> ${{data.sha256_hash}}<br>
          <strong>MEDIA TYPE:</strong> ${{data.media_type.toUpperCase()}}<br>
          <strong>CAPTURE RECEIPT ID:</strong> ${{data.id}}<br>
          <strong>SECURITY SCORE:</strong> ${{Math.round(data.verification_score * 100)}}% LIVE AUTHENTICATED
        `;
        document.getElementById('success-view').style.display = 'block';

        if (stream) {{
          stream.getTracks().forEach(t => t.stop());
        }}
      }} catch (err) {{
        alert("Upload error: " + err.message);
        submitBtn.disabled = false;
        submitBtn.textContent = '✅ Submit Live Proof';
      }}
    }});
  </script>
</body>
</html>"""
    return HTMLResponse(content=html, status_code=200)
