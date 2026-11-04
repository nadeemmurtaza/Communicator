import base64, hashlib, hmac, json, os, sqlite3, subprocess, sys, time, urllib.parse, urllib.request

DB = os.path.abspath("smoke_wh.db")
for suffix in ("", "-wal", "-shm"):
    try:
        os.remove(DB + suffix)
    except FileNotFoundError:
        pass

BASE = "http://127.0.0.1:8130"
APP_SECRET = "smoke_app_secret"
TW_TOKEN = "smoke_twilio_token"
POSTMARK_SECRET = "smoke_postmark_secret"
EMAIL_SECRET = "smoke_email_secret"
TIKTOK_SECRET = "smoke_tiktok_secret"
GATEWAY_KEY = "smoke_gateway_key"

env = dict(os.environ, PORT="8130", HOST="127.0.0.1",
           RUST_DATABASE_URL=f"sqlite://{DB}?mode=rwc",
           META_APP_SECRET=APP_SECRET,
           META_WEBHOOK_VERIFY_TOKEN="smoke_verify_token",
           TWILIO_AUTH_TOKEN=TW_TOKEN,
           POSTMARK_INBOUND_WEBHOOK_SECRET=POSTMARK_SECRET,
           EMAIL_INBOUND_WEBHOOK_SECRET=EMAIL_SECRET,
           TIKTOK_APP_SECRET=TIKTOK_SECRET,
           TIKTOK_WEBHOOK_VERIFY_TOKEN="smoke_tiktok_token",
           ECOSYSTEM_GATEWAY_KEY=GATEWAY_KEY)
log = open("smoke_server.log", "w", encoding="utf-8")
proc = subprocess.Popen([r"target\debug\backend-rs.exe"], env=env,
                        stdout=log, stderr=log)

checks = []


def record(label, ok, detail=""):
    checks.append((label, ok, str(detail)))


def request(path, data=None, headers=None):
    req = urllib.request.Request(BASE + path, data=data, headers=headers or {})
    try:
        with urllib.request.urlopen(req) as r:
            return r.status, r.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()
    except Exception as e:  # connection refused, etc.
        return 0, repr(e)


def get(path, headers=None):
    return request(path, None, headers)


def post_json(path, payload, headers=None):
    merged = {"content-type": "application/json", **(headers or {})}
    return request(path, json.dumps(payload).encode(), merged)


def post_form(path, params, headers=None):
    merged = {"content-type": "application/x-www-form-urlencoded", **(headers or {})}
    return request(path, urllib.parse.urlencode(params).encode(), merged)


def meta_sig(payload_bytes, secret=APP_SECRET):
    return "sha256=" + hmac.new(secret.encode(), payload_bytes, hashlib.sha256).hexdigest()


def twilio_sig(path, params, secret=TW_TOKEN):
    url = BASE + path
    data = url + "".join(k + params[k] for k in sorted(params))
    return base64.b64encode(hmac.new(secret.encode(), data.encode(), hashlib.sha1).digest()).decode()


def b64_hmac_sha256(payload_bytes, secret):
    return base64.b64encode(hmac.new(secret.encode(), payload_bytes, hashlib.sha256).digest()).decode()


def hex_hmac_sha256(payload_bytes, secret):
    return hmac.new(secret.encode(), payload_bytes, hashlib.sha256).hexdigest()


for _ in range(60):
    if get("/api/v1/health")[0] == 200:
        break
    time.sleep(0.5)
else:
    print("server never came up -- see smoke_server.log")
    sys.exit(1)

NOW = time.strftime("%Y-%m-%dT%H:%M:%SZ")
con = sqlite3.connect(DB)


def seed_contact(cid, value, channel, name="Contact Placeholder"):
    con.execute("INSERT INTO contacts (id, value, channel, person_name, created_at, updated_at) "
                "VALUES (?, ?, ?, ?, ?, ?)", (cid, value, channel, name, NOW, NOW))


seed_contact("c-smoke-1", "+923331234567", "whatsapp")
seed_contact("c-smoke-2", "+14155551234", "whatsapp")
seed_contact("c-smoke-3", "customer@acme.test", "email")

INBOXES = [
    ("in-fb", "facebook", json.dumps({"appSecret": "fb_app_secret", "webhookVerifyToken": "fb_verify_token"})),
    ("in-ig", "instagram", json.dumps({"appSecret": "ig_app_secret", "webhookVerifyToken": "ig_verify_token"})),
    ("in-tg", "telegram", json.dumps({"secretToken": "tg_secret"})),
    ("in-line", "line", json.dumps({"channelSecret": "line_channel_secret"})),
    ("in-tt", "tiktok", json.dumps({"appSecret": TIKTOK_SECRET, "webhookVerifyToken": "smoke_tiktok_token"})),
    ("in-voice", "voice", json.dumps({
        "ivrEnabled": True, "ivrPrompt": "Press one for sales.", "agentQueue": "smoke-agent",
        "salesQueue": "smoke-sales", "supportQueue": "smoke-support", "voicemailEnabled": True})),
    ("in-mail", "email", json.dumps({"inboundAddress": "support@acme.test"})),
]
for inbox_id, channel, creds in INBOXES:
    con.execute("INSERT INTO inboxes (id, name, channel_type, is_active, credentials, tenant_key, created_at, updated_at) "
                "VALUES (?, ?, ?, 1, ?, 'newax', ?, ?)", (inbox_id, channel + " inbox", channel, creds, NOW, NOW))
con.execute("INSERT INTO live_proof_sessions (id, tenant_id, session_token, status, expires_at, created_at, updated_at) "
            "VALUES ('lp-1', 'newax', 'tok-1', 'pending', ?, ?, ?)", (NOW, NOW, NOW))
con.commit()
con.close()

# ---------------------------------------------------------------- Meta WhatsApp
payload = {
    "object": "whatsapp_business_account",
    "entry": [{"changes": [{"field": "messages", "value": {
        "contacts": [{"wa_id": "923331234567", "profile": {"name": "Ayesha Khan"}}],
        "messages": [{"id": "wamid.SMOKE1", "from": "923331234567", "type": "text",
                      "text": {"body": "What are your rates?"}}]}}]}],
}
raw = json.dumps(payload).encode()
sig = meta_sig(raw)
code, body = request("/api/v1/webhooks/meta/whatsapp", raw, {"x-hub-signature-256": sig, "content-type": "application/json"})
record("whatsapp signed inbound accepted", code == 200 and '"ingested":1' in body, f"{code} {body}")

code, body = request("/api/v1/webhooks/meta/whatsapp", raw, {"x-hub-signature-256": "sha256=" + "0" * 64, "content-type": "application/json"})
record("whatsapp forged signature rejected", code == 403, f"{code} {body}")

code, body = request("/api/v1/webhooks/meta/whatsapp", raw, {"content-type": "application/json"})
record("whatsapp missing signature rejected", code == 403, f"{code} {body}")

code, body = request("/api/v1/webhooks/meta/whatsapp", raw, {"x-hub-signature-256": sig, "content-type": "application/json"})
record("whatsapp duplicate wamid reported", '"duplicates":1' in body, f"{code} {body}")

code, body = get("/api/v1/webhooks/meta/whatsapp?hub.mode=subscribe&hub.verify_token=smoke_verify_token&hub.challenge=12345")
record("whatsapp handshake echoes challenge", code == 200 and body == "12345", f"{code} {body}")

code, body = get("/api/v1/webhooks/meta/whatsapp?hub.mode=subscribe&hub.verify_token=wrong&hub.challenge=12345")
record("whatsapp handshake rejects bad token", code == 403, f"{code} {body}")

stop = {"object": "whatsapp_business_account", "entry": [{"changes": [{"field": "messages", "value": {
    "messages": [{"id": "wamid.SMOKE2", "from": "923331234567", "type": "text",
                  "text": {"body": "please stop texting me"}}]}}]}]}
raw2 = json.dumps(stop).encode()
code, body = request("/api/v1/webhooks/meta/whatsapp", raw2, {"x-hub-signature-256": meta_sig(raw2), "content-type": "application/json"})
record("whatsapp STOP inbound accepted", code == 200 and '"ingested":1' in body, f"{code} {body}")

receipt = {"object": "whatsapp_business_account", "entry": [{"changes": [{"field": "messages", "value": {
    "statuses": [{"id": "wamid.NOPE", "status": "delivered", "timestamp": "1730000000"}]}}]}]}
raw3 = json.dumps(receipt).encode()
code, body = request("/api/v1/webhooks/meta/whatsapp", raw3, {"x-hub-signature-256": meta_sig(raw3), "content-type": "application/json"})
record("whatsapp unknown receipt harmless", code == 200 and '"receipts":1' in body, f"{code} {body}")

quality = {"object": "whatsapp_business_account", "entry": [{"changes": [
    {"field": "phone_number_quality_update", "value": {"phone_number_id": "123"}}]}]}
raw4 = json.dumps(quality).encode()
code, body = request("/api/v1/webhooks/meta/whatsapp", raw4, {"x-hub-signature-256": meta_sig(raw4), "content-type": "application/json"})
record("whatsapp quality update triggers re-sync", code == 200 and '"channel_syncs":1' in body, f"{code} {body}")

# ---------------------------------------------------------------- Twilio
tw_params = {"From": "whatsapp:+14155551234", "Body": "hello there", "MessageSid": "SMfake1"}
code, body = post_form("/api/v1/webhooks/twilio/whatsapp", tw_params, {"x-twilio-signature": twilio_sig("/api/v1/webhooks/twilio/whatsapp", tw_params)})
record("twilio whatsapp signed inbound accepted", code == 200 and '"duplicate":false' in body, f"{code} {body}")

code, body = post_form("/api/v1/webhooks/twilio/whatsapp", tw_params, {"x-twilio-signature": "bogus"})
record("twilio whatsapp forged signature rejected", code == 403, f"{code} {body}")

sms = {"From": "+14155559999", "Body": "unsubscribe", "MessageSid": "SMfake2"}
code, body = post_form("/api/v1/webhooks/twilio/sms", sms, {"x-twilio-signature": twilio_sig("/api/v1/webhooks/twilio/sms", sms)})
record("twilio sms unknown contact acknowledged", code == 200 and '"ignored":"unknown_contact"' in body, f"{code} {body}")

# ---------------------------------------------------------------- Facebook
fb = {"object": "page", "entry": [{"messaging": [{"sender": {"id": "fb-user-1"},
        "message": {"mid": "m_fb_1", "text": "Hi from Messenger"}}]}]}
raw_fb = json.dumps(fb).encode()
code, body = post_json("/api/v1/webhooks/meta/facebook", fb, {"x-hub-signature-256": meta_sig(raw_fb, "fb_app_secret")})
record("facebook signed inbound ingested", code == 200 and '"ingested":1' in body, f"{code} {body}")

code, body = post_json("/api/v1/webhooks/meta/facebook", fb, {"x-hub-signature-256": meta_sig(raw_fb, APP_SECRET)})
record("facebook wrong-secret signature rejected", code == 403, f"{code} {body}")

code, body = post_json("/api/v1/webhooks/meta/facebook", fb, {})
record("facebook unsigned request rejected", code == 403, f"{code} {body}")

code, body = get("/api/v1/webhooks/meta/facebook?hub.mode=subscribe&hub.verify_token=fb_verify_token&hub.challenge=777")
record("facebook handshake uses inbox token", code == 200 and body == "777", f"{code} {body}")

def instagram_inbound(mid, text="Story mention", secret="ig_app_secret"):
    payload = {"object": "instagram", "entry": [{"messaging": [{"sender": {"id": "ig-user-1"},
               "message": {"mid": mid, "text": text}}]}]}
    raw = json.dumps(payload).encode()
    return payload, raw, {"x-hub-signature-256": meta_sig(raw, secret)}


payload, raw, headers = instagram_inbound("m_ig_1")
code, body = request("/api/v1/webhooks/meta/instagram", raw, headers)
record("instagram signed inbound ingested", code == 200 and '"ingested":1' in body, f"{code} {body}")

payload, raw, headers = instagram_inbound("m_ig_2", secret="wrong_secret")
code, body = request("/api/v1/webhooks/meta/instagram", raw, headers)
record("instagram forged signature rejected", code == 403, f"{code} {body}")

# Deactivate the inbox: no creds secret and no META_INSTAGRAM_APP_SECRET -> refuse, never accept.
con = sqlite3.connect(DB)
con.execute("UPDATE inboxes SET is_active = 0 WHERE id = 'in-ig'")
con.commit()
con.close()
payload, raw, headers = instagram_inbound("m_ig_3")
code, body = request("/api/v1/webhooks/meta/instagram", raw, headers)
record("instagram unconfigured returns 503", code == 503, f"{code} {body}")
con = sqlite3.connect(DB)
con.execute("UPDATE inboxes SET is_active = 1 WHERE id = 'in-ig'")
con.commit()
con.close()

# Re-activated but with the app secret stripped from its credentials: still must refuse inbound.
con = sqlite3.connect(DB)
con.execute("UPDATE inboxes SET credentials = ? WHERE id = 'in-ig'",
            (json.dumps({"webhookVerifyToken": "ig_verify_token"}),))
con.commit()
con.close()
payload, raw, headers = instagram_inbound("m_ig_4")
code, body = request("/api/v1/webhooks/meta/instagram", raw, headers)
record("instagram inbox without app secret refuses", code == 503, f"{code} {body}")

con = sqlite3.connect(DB)
con.execute("UPDATE inboxes SET credentials = ? WHERE id = 'in-ig'",
            (json.dumps({"appSecret": "ig_app_secret", "webhookVerifyToken": "ig_verify_token"}),))
con.commit()
con.close()
payload, raw, headers = instagram_inbound("m_ig_5")
code, body = request("/api/v1/webhooks/meta/instagram", raw, headers)
record("instagram ingests once credentials restored", code == 200 and '"ingested":1' in body, f"{code} {body}")

# ---------------------------------------------------------------- Telegram
tg = {"message": {"message_id": 501, "chat": {"id": 99001}, "from": {"first_name": "Bilal"},
                  "text": "send me the catalogue"}}
raw_tg = json.dumps(tg).encode()
code, body = post_json("/api/v1/webhooks/telegram", tg, {"x-telegram-bot-api-secret-token": "tg_secret"})
record("telegram signed inbound ingested", code == 200 and '"ingested":1' in body, f"{code} {body}")

code, body = post_json("/api/v1/webhooks/telegram", {"message": {"message_id": 502, "chat": {"id": 99002}, "text": "x"}},
                       {"x-telegram-bot-api-secret-token": "wrong"})
record("telegram bad secret token rejected", code == 403, f"{code} {body}")

code, body = post_json("/api/v1/webhooks/telegram", tg, {})
record("telegram missing secret token rejected", code == 403, f"{code} {body}")

cb = {"callback_query": {"id": "cb1", "data": "select_plan_basic", "message": {"chat": {"id": 99001}},
                         "from": {"first_name": "Bilal"}}}
code, body = post_json("/api/v1/webhooks/telegram", cb, {"x-telegram-bot-api-secret-token": "tg_secret"})
record("telegram button callback ingested", code == 200 and '"ingested":1' in body, f"{code} {body}")

# ---------------------------------------------------------------- LINE
ln = {"events": [
    {"type": "message", "source": {"userId": "line-user-1"}, "message": {"id": "lm1", "type": "text", "text": "Where is my order?"}},
    {"type": "postback", "source": {"userId": "line-user-1"}, "timestamp": 1730000000,
     "postback": {"data": "menu_orders"}},
    {"type": "message", "source": {"userId": "line-user-2"}, "message": {"id": "lm2", "type": "image"}},
]}
raw_ln = json.dumps(ln).encode()
code, body = post_json("/api/v1/webhooks/line", ln, {"x-line-signature": b64_hmac_sha256(raw_ln, "line_channel_secret")})
record("line signed events ingested", code == 200 and '"ingested":3' in body, f"{code} {body}")

code, body = post_json("/api/v1/webhooks/line", ln, {"x-line-signature": b64_hmac_sha256(raw_ln, "wrong_secret")})
record("line forged signature rejected", code == 403, f"{code} {body}")

code, body = post_json("/api/v1/webhooks/line", ln, {})
record("line missing signature rejected", code == 403, f"{code} {body}")

# ---------------------------------------------------------------- TikTok
code, body = get("/api/v1/webhooks/tiktok?hub.mode=subscribe&hub.verify_token=smoke_tiktok_token&hub.challenge=tt-challenge")
record("tiktok handshake echoes challenge", code == 200 and body == "tt-challenge", f"{code} {body}")

code, body = get("/api/v1/webhooks/tiktok?hub.mode=subscribe&hub.verify_token=nope&hub.challenge=tt-challenge")
record("tiktok handshake rejects bad token", code == 403, f"{code} {body}")

tt = {"event": "im.direct_message.receive", "content": {"open_id": "tt-user-1", "text": "Is this available in black?",
                                                        "message_id": "ttm1"},
      "user": {"nickname": "Sana"}}
raw_tt = json.dumps(tt).encode()
code, body = post_json("/api/v1/webhooks/tiktok", tt, {"x-tiktok-signature": hex_hmac_sha256(raw_tt, TIKTOK_SECRET)})
record("tiktok signed dm ingested", code == 200 and '"ingested":1' in body, f"{code} {body}")

code, body = post_json("/api/v1/webhooks/tiktok", tt, {"x-tiktok-signature": "deadbeef"})
record("tiktok forged signature rejected", code == 403, f"{code} {body}")

cm = {"event": "comment.create", "content": {"from_user_id": "tt-user-2", "text": "price?",
                                             "comment_id": "cmt1", "video_id": "V999"}}
raw_cm = json.dumps(cm).encode()
code, body = post_json("/api/v1/webhooks/tiktok", cm, {"x-tiktok-signature": hex_hmac_sha256(raw_cm, TIKTOK_SECRET)})
record("tiktok comment flagged as comment", code == 200 and '"isComment":true' in body, f"{code} {body}")

# ---------------------------------------------------------------- Voice
voice = {"CallSid": "CA-smoke-1", "From": "+15550001111", "To": "+15550009999", "Digits": ""}
code, body = post_form("/api/v1/webhooks/voice/inbound", voice,
                       {"x-twilio-signature": twilio_sig("/api/v1/webhooks/voice/inbound", voice)})
record("voice ivr returns configured prompt",
       code == 200 and "<Response>" in body and "Press one for sales." in body and "smoke-agent" in body, f"{code} {body}")
record("voice prompt is not a canned brand string", "Bay Works" not in body, body[:80])

sales = {"CallSid": "CA-smoke-2", "From": "+15550002222", "To": "+15550009999", "Digits": "1"}
code, body = post_form("/api/v1/webhooks/voice/inbound", sales,
                       {"x-twilio-signature": twilio_sig("/api/v1/webhooks/voice/inbound", sales)})
record("voice ivr digit routes to sales queue", code == 200 and "smoke-sales" in body, f"{code} {body}")

code, body = post_form("/api/v1/webhooks/voice/inbound", sales, {"x-twilio-signature": "bogus"})
record("voice status forged signature rejected", code == 403, f"{code} {body}")

code, body = post_json("/api/v1/webhooks/voice/inbound", sales, {"x-twilio-signature": "whatever"})
record("voice json payload refused so signing is mandatory", code == 415, f"{code} {body}")

status_p = {"CallSid": "CA-smoke-1", "CallStatus": "completed", "CallDuration": "94"}
code, body = post_form("/api/v1/webhooks/voice/status", status_p,
                       {"x-twilio-signature": twilio_sig("/api/v1/webhooks/voice/status", status_p)})
record("voice status updates the call record", code == 200 and '"duration":94' in body, f"{code} {body}")

rec_p = {"CallSid": "CA-smoke-1", "RecordingUrl": "https://api.twilio.com/rec1", "RecordingSid": "XR1",
         "RecordingDuration": "42"}
code, body = post_form("/api/v1/webhooks/voice/recording", rec_p,
                       {"x-twilio-signature": twilio_sig("/api/v1/webhooks/voice/recording", rec_p)})
record("voice recording attached", code == 200 and "https://api.twilio.com/rec1" in body, f"{code} {body}")

tr_p = {"CallSid": "CA-smoke-2", "TranscriptionText": "I need a refund", "TranscriptionStatus": "completed"}
code, body = post_form("/api/v1/webhooks/voice/transcription", tr_p,
                       {"x-twilio-signature": twilio_sig("/api/v1/webhooks/voice/transcription", tr_p)})
record("voice transcription attached", code == 200 and '"transcriptionStatus":"completed"' in body, f"{code} {body}")

# ---------------------------------------------------------------- Email
email = {
    "From": "Alice Example <Alice@Acme.test>",
    "Subject": "Invoice 4471",
    "TextBody": "Please send the corrected invoice.\n\nOn Mon, Alice wrote:\n> older quoted text",
    "MessageID": "<inv4471@acme.test>",
    "To": "support@acme.test",
}
code, body = post_json("/api/v1/webhooks/email/inbound", email, {"x-webhook-token": EMAIL_SECRET})
record("email inbound ingested", code == 200 and '"from":"alice@acme.test"' in body, f"{code} {body}")

code, body = post_json("/api/v1/webhooks/email/inbound", email, {})
record("email missing token rejected", code == 403, f"{code} {body}")

code, body = post_json("/api/v1/webhooks/email/inbound", {"From": "no-address", "TextBody": "hi"},
                       {"x-webhook-token": EMAIL_SECRET})
record("email without sender refused", code == 400, f"{code} {body}")

code, body = post_json("/api/v1/webhooks/email/inbound", {"From": "a@b.test", "TextBody": "hi"},
                       {"x-webhook-token": EMAIL_SECRET})
record("email without Message-ID refused", code == 400, f"{code} {body}")

repeat = dict(email, MessageID="<inv4471@acme.test>")
code, body = post_json("/api/v1/webhooks/email/inbound", repeat, {"x-webhook-token": EMAIL_SECRET})
record("email duplicate stored once", code == 200 and "<inv4471@acme.test>" in body, f"{code} {body}")

# ---------------------------------------------------------------- Postmark
pm = {"From": "customer@acme.test", "TextBody": "following up on my order", "MessageID": "pm-1"}
code, body = post_json(f"/api/v1/webhooks/postmark/inbound?token={POSTMARK_SECRET}", pm, {})
record("postmark token accepted ingest", code == 200 and '"ingested":1' in body, f"{code} {body}")

code, body = post_json("/api/v1/webhooks/postmark/inbound?token=wrong", pm, {})
record("postmark bad token rejected", code == 403, f"{code} {body}")

# ---------------------------------------------------------------- Ecosystem
ev = {"name": "refund.decision", "tenantId": "newax", "payload": {"sessionId": "lp-1", "decision": "approved"}}
raw_ev = json.dumps(ev).encode()
code, body = request("/api/v1/webhooks/ecosystem/events", raw_ev,
                     {"content-type": "application/json", "x-gateway-signature": hex_hmac_sha256(raw_ev, GATEWAY_KEY)})
record("ecosystem refund decision applied", code == 200 and '"action":"live_proof_verified"' in body, f"{code} {body}")

code, body = request("/api/v1/webhooks/ecosystem/events", raw_ev,
                     {"content-type": "application/json", "x-gateway-signature": "nope"})
record("ecosystem forged signature rejected", code == 403, f"{code} {body}")

ev2 = {"name": "order.placed", "tenantId": "newax", "payload": {"orderId": "O-9"}}
code, body = post_json("/api/v1/webhooks/ecosystem/events", ev2, {"x-webhook-token": GATEWAY_KEY})
record("ecosystem unported workflow reported as deferred",
       code == 200 and '"action":"deferred_no_automation:order.placed"' in body, f"{code} {body}")

code, body = post_json("/api/v1/webhooks/ecosystem/events", {"payload": {}}, {"x-webhook-token": GATEWAY_KEY})
record("ecosystem event without name refused", code == 400, f"{code} {body}")

# ---------------------------------------------------------------- Row-level truth
con = sqlite3.connect(DB)
rows = con.execute("SELECT direction, body, provider, provider_message_id, status FROM messages ORDER BY sent_at").fetchall()
bodies = [r[1] for r in rows]
dedup = con.execute("SELECT COUNT(*) FROM messages WHERE provider_message_id='wamid.SMOKE1'").fetchone()[0]
email_dedup = con.execute("SELECT COUNT(*) FROM messages WHERE provider_message_id='<inv4471@acme.test>'").fetchone()[0]
name = con.execute("SELECT person_name FROM contacts WHERE id='c-smoke-1'").fetchone()[0]
conv = con.execute("SELECT state FROM conversations WHERE contact_id='c-smoke-1'").fetchall()
consent = con.execute("SELECT status, source FROM consent_records ORDER BY recorded_at").fetchall()
provisioned = con.execute("SELECT COUNT(*) FROM contacts WHERE channel IN ('facebook','instagram','telegram','line','tiktok','voice','email')").fetchone()[0]
leads = con.execute("SELECT COUNT(*) FROM leads").fetchone()[0]
stored_events = con.execute("SELECT COUNT(*) FROM ecosystem_events").fetchone()[0]
proof = con.execute("SELECT status FROM live_proof_sessions WHERE id='lp-1'").fetchone()[0]
quote_kept = [b for b in bodies if "older quoted text" in b]
email_stored = [b for b in bodies if "corrected invoice" in b]
caller_names = con.execute("SELECT person_name FROM contacts WHERE channel = 'telegram'").fetchall()
rec = con.execute("SELECT body, provider_metadata FROM messages WHERE provider_message_id='CA-smoke-1'").fetchone()
tts = con.execute("SELECT body FROM messages WHERE provider_message_id='cmt1'").fetchone()
line_img = con.execute("SELECT body FROM messages WHERE provider_message_id='lm2'").fetchone()
con.close()

record("whatsapp duplicate wamid stored once", dedup == 1, dedup)
record("email duplicate stored once", email_dedup == 1, email_dedup)
record("whatsapp sender profile name captured", name == "Ayesha Khan", name)
record("conversation went engaged then opted_out", [c[0] for c in conv] == ["opted_out"], conv)
record("stop keyword consent recorded", ("opted_out", "inbound_stop_keyword") in consent, consent)
record("provisioned channels write opt-in consent", ("opted_in", "facebook_webhook_inbound") in consent, consent)
record("social senders auto-provisioned", provisioned >= 6, provisioned)
record("leads created for provisioned contacts", leads >= 6, leads)
record("telegram sender name from payload", "Bilal" in [c[0] for c in caller_names], caller_names)
record("quoted email reply stripped", not quote_kept, quote_kept)
record("email body and subject stored", any("Subject: Invoice 4471" in b for b in email_stored), email_stored)
record("tiktok comment labelled with video id", tts and "V999" in tts[0], tts)
record("line non-text event labelled", line_img and line_img[0] == "Image received", line_img)
rec_meta = json.loads(rec[1]) if rec and rec[1] else {}
record("voice call row carries duration and recording",
       rec and "Call completed (94s)" in rec[0] and rec_meta.get("duration") == 94
       and rec_meta.get("call_status") == "completed"
       and rec_meta.get("recording_url") == "https://api.twilio.com/rec1"
       and rec_meta.get("recording_duration") == 42, (rec, rec_meta))
record("ecosystem events persisted", stored_events == 2, stored_events)
record("live proof session verified", proof == "verified", proof)
record("all channels produced message rows", len(rows) >= 16, f"{len(rows)} rows")

proc.terminate()
try:
    proc.wait(timeout=10)
except Exception:
    proc.kill()
log.flush()
log.close()

failed = [c for c in checks if not c[1]]
for label, ok, detail in checks:
    text = f"{'PASS' if ok else 'FAIL'}  {label}"
    if not ok or len(detail) < 100:
        text += f"  -> {detail}"
    print(text)
print(f"\n{len(checks) - len(failed)}/{len(checks)} passed")
if failed:
    print("\nserver log tail:\n" + open("smoke_server.log", encoding="utf-8").read()[-3000:])
sys.exit(1 if failed else 0)
