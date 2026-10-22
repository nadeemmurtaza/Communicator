import uuid
import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.models.contact import ChannelType, Contact
from communicator.models.conversation import Conversation
from communicator.models.lead import Lead
from communicator.models.message import Message, MessageDirection
from communicator.services import translation_service


@pytest.mark.asyncio
async def test_detect_language():
    # Spanish
    res_es = translation_service.detect_language("Hola, necesito ayuda con mi factura por favor")
    assert res_es["language"] == "es"
    assert res_es["confidence"] >= 0.85

    # German
    res_de = translation_service.detect_language("Hallo, wie kann ich mein Passwort ändern?")
    assert res_de["language"] == "de"
    assert res_de["confidence"] >= 0.85

    # Russian (Cyrillic script)
    res_ru = translation_service.detect_language("Привет, как дела?")
    assert res_ru["language"] == "ru"
    assert res_ru["script"] == "Cyrillic"

    # Arabic (Arabic script)
    res_ar = translation_service.detect_language("مرحبا كيف حالك")
    assert res_ar["language"] == "ar"
    assert res_ar["script"] == "Arabic"

    # Japanese
    res_ja = translation_service.detect_language("こんにちは、サポートが必要です")
    assert res_ja["language"] == "ja"

    # English
    res_en = translation_service.detect_language("Hello, could you please help me with my account?")
    assert res_en["language"] == "en"


@pytest.mark.asyncio
async def test_translate_text():
    # Spanish to English
    res_es_en = await translation_service.translate_text(
        text="Hola, necesito ayuda con mi factura",
        target_language="en",
    )
    assert res_es_en["target_language"] == "en"
    assert res_es_en["source_language"] == "es"
    assert "Hello" in res_es_en["translated_text"]
    assert "invoice" in res_es_en["translated_text"]

    # English to Spanish
    res_en_es = await translation_service.translate_text(
        text="Thank you for contacting support",
        target_language="es",
    )
    assert res_en_es["target_language"] == "es"
    assert res_en_es["source_language"] == "en"
    assert "Gracias" in res_en_es["translated_text"]

    # French to English
    res_fr_en = await translation_service.translate_text(
        text="Bonjour, où est ma commande ?",
        target_language="en",
    )
    assert res_fr_en["target_language"] == "en"
    assert "order" in res_fr_en["translated_text"].lower()


@pytest.mark.asyncio
async def test_translate_message_persistence(db_session):
    # Setup test Lead, Contact, Conversation, Message
    lead = Lead(name="Carlos Santana", harvester_institution_id="inst_carlos_123")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.WHATSAPP,
        value="+34612345678",
        person_name="Carlos",
        language="es",
    )
    db_session.add(contact)
    await db_session.flush()

    conv = Conversation(contact_id=contact.id)
    db_session.add(conv)
    await db_session.flush()

    msg = Message(
        conversation_id=conv.id,
        direction=MessageDirection.INBOUND,
        channel=ChannelType.WHATSAPP,
        body="Hola, necesito ayuda con mi factura",
    )
    db_session.add(msg)
    await db_session.commit()

    # Translate the message
    trans = await translation_service.translate_message(db_session, msg.id, target_language="en")
    assert trans is not None
    assert trans["target_language"] == "en"
    assert trans["source_language"] == "es"
    assert "invoice" in trans["translated_text"]

    # Verify message in DB updated
    await db_session.refresh(msg)
    assert msg.provider_metadata.get("is_translated") is True
    assert msg.provider_metadata["translation"]["translated_text"] == trans["translated_text"]


@pytest.mark.asyncio
async def test_prepare_outbound_translation(db_session):
    lead = Lead(name="Mateo Silva", harvester_institution_id="inst_mateo_123")
    db_session.add(lead)
    await db_session.flush()

    contact = Contact(
        lead_id=lead.id,
        channel=ChannelType.EMAIL,
        value="mateo@acme.es",
        language="es",
    )
    db_session.add(contact)
    await db_session.flush()

    conv = Conversation(contact_id=contact.id)
    db_session.add(conv)
    await db_session.commit()

    # Outbound translation preparation
    prepared = await translation_service.prepare_outbound_translation(
        session=db_session,
        conversation_id=conv.id,
        text="Thank you for contacting support",
    )
    assert prepared["customer_language"] == "es"
    assert "Gracias" in prepared["translated_text"]


@pytest.mark.asyncio
async def test_translation_api_endpoints(db_session, override_database_dependency):
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # 1. GET /api/v1/translation/languages
        r_langs = await client.get("/api/v1/translation/languages")
        assert r_langs.status_code == 200
        langs = r_langs.json()["languages"]
        assert len(langs) >= 20
        assert any(l["code"] == "es" for l in langs)
        assert any(l["code"] == "fr" for l in langs)

        # 2. GET /api/v1/translation/settings
        r_get_settings = await client.get("/api/v1/translation/settings")
        assert r_get_settings.status_code == 200
        settings = r_get_settings.json()["settings"]
        assert settings["is_enabled"] is True

        # 3. PUT /api/v1/translation/settings
        r_put_settings = await client.put(
            "/api/v1/translation/settings",
            json={"auto_translate_inbound": True, "agent_primary_language": "en"},
        )
        assert r_put_settings.status_code == 200

        # 4. POST /api/v1/translation/detect
        r_detect = await client.post(
            "/api/v1/translation/detect",
            json={"text": "Bonjour, comment ça va ?"},
        )
        assert r_detect.status_code == 200
        assert r_detect.json()["language"] == "fr"

        # 5. POST /api/v1/translation/translate
        r_trans = await client.post(
            "/api/v1/translation/translate",
            json={"text": "Hola, necesito ayuda con mi factura", "target_language": "en"},
        )
        assert r_trans.status_code == 200
        assert "Hello" in r_trans.json()["translated_text"]
