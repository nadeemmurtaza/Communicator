from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.channels.base import ChannelProvider, SendResult
from communicator.channels.email import UniversalEmailProvider
from communicator.channels.line import LineProvider
from communicator.channels.live_chat import WebLiveChatProvider
from communicator.channels.messenger import InstagramLoginProvider, MetaMessengerProvider
from communicator.channels.sms import TwilioSMSProvider
from communicator.channels.telegram import TelegramProvider
from communicator.channels.tiktok import TikTokProvider
from communicator.channels.voice import VoiceProvider
from communicator.channels.whatsapp import MetaWhatsAppProvider, TwilioWhatsAppProvider
from communicator.models.contact import ChannelType
from communicator.models.inbox import Inbox
from communicator.services.integration_settings import get_whatsapp_config

_PROVIDERS: dict[ChannelType, type[ChannelProvider]] = {
    ChannelType.EMAIL: UniversalEmailProvider,
    ChannelType.SMS: TwilioSMSProvider,
    ChannelType.WEBSITE: WebLiveChatProvider,
    ChannelType.FACEBOOK: MetaMessengerProvider,
    ChannelType.INSTAGRAM: MetaMessengerProvider,
    ChannelType.TELEGRAM: TelegramProvider,
    ChannelType.LINE: LineProvider,
    ChannelType.VOICE: VoiceProvider,
    ChannelType.TIKTOK: TikTokProvider,
}


async def get_provider(channel: ChannelType, session: AsyncSession) -> ChannelProvider:
    if channel == ChannelType.WHATSAPP:
        config = await get_whatsapp_config(session)
        if config.provider == "meta":
            return MetaWhatsAppProvider(
                access_token=config.access_token, phone_number_id=config.phone_number_id
            )
        if config.provider == "twilio":
            return TwilioWhatsAppProvider()
        raise ValueError(f"Unknown whatsapp_provider setting: {config.provider!r}")

    if channel == ChannelType.EMAIL:
        stmt = select(Inbox).where(Inbox.channel_type == ChannelType.EMAIL)
        inbox = (await session.execute(stmt)).scalars().first()
        smtp_host = None
        smtp_port = None
        smtp_username = None
        smtp_password = None
        smtp_encryption = "tls"
        from_address = None
        from_name = None
        reply_to = None
        postmark_api_key = None
        if inbox and inbox.credentials:
            creds = inbox.credentials
            smtp_host = creds.get("smtpHost")
            smtp_port = int(creds.get("smtpPort", 587)) if creds.get("smtpPort") else 587
            smtp_username = creds.get("smtpUsername")
            smtp_password = creds.get("smtpPassword")
            smtp_encryption = creds.get("smtpEncryption", "tls")
            from_address = creds.get("emailFromAddress")
            from_name = creds.get("emailFromName")
            reply_to = creds.get("emailReplyTo")
            postmark_api_key = creds.get("postmarkApiKey")
        return UniversalEmailProvider(
            smtp_host=smtp_host,
            smtp_port=smtp_port,
            smtp_username=smtp_username,
            smtp_password=smtp_password,
            smtp_encryption=smtp_encryption,
            from_address=from_address,
            from_name=from_name,
            reply_to=reply_to,
            postmark_api_key=postmark_api_key,
        )

    if channel == ChannelType.SMS:
        stmt = select(Inbox).where(Inbox.channel_type == ChannelType.SMS)
        sms_inbox = (await session.execute(stmt)).scalars().first()
        if sms_inbox and sms_inbox.credentials:
            creds = sms_inbox.credentials
            account_sid = creds.get("accountSid")
            auth_token = creds.get("authToken")
            from_number = sms_inbox.phone_number or creds.get("phoneNumber")
            messaging_service_sid = creds.get("messagingServiceSid")
            if account_sid and auth_token and (from_number or messaging_service_sid):
                return TwilioSMSProvider(
                    account_sid=account_sid,
                    auth_token=auth_token,
                    from_number=from_number,
                    messaging_service_sid=messaging_service_sid,
                )

    if channel == ChannelType.INSTAGRAM:
        stmt = select(Inbox).where(Inbox.channel_type == ChannelType.INSTAGRAM)
        inbox = (await session.execute(stmt)).scalars().first()
        creds = inbox.credentials if inbox and inbox.credentials else {}
        if creds.get("igId") and not creds.get("pageId"):
            # Connected via Instagram Business Login (instagram_auth.py) --
            # that token is only valid against graph.instagram.com, not the
            # Facebook Page messaging endpoint below.
            return InstagramLoginProvider(
                access_token=creds.get("accessToken"), ig_user_id=creds.get("igId")
            )
        return MetaMessengerProvider(
            page_access_token=creds.get("pageAccessToken") or creds.get("accessToken"),
            page_id=creds.get("pageId"),
            channel_name="instagram",
        )

    if channel == ChannelType.FACEBOOK:
        stmt = select(Inbox).where(Inbox.channel_type == ChannelType.FACEBOOK)
        inbox = (await session.execute(stmt)).scalars().first()
        token = None
        page_id = None
        if inbox and inbox.credentials:
            token = inbox.credentials.get("pageAccessToken") or inbox.credentials.get("accessToken")
            page_id = inbox.credentials.get("pageId")
        return MetaMessengerProvider(
            page_access_token=token,
            page_id=page_id,
            channel_name="facebook",
        )

    if channel == ChannelType.TELEGRAM:
        stmt = select(Inbox).where(Inbox.channel_type == ChannelType.TELEGRAM)
        inbox = (await session.execute(stmt)).scalars().first()
        token = None
        username = None
        if inbox and inbox.credentials:
            token = inbox.credentials.get("botToken")
            username = inbox.credentials.get("botUsername")
        return TelegramProvider(bot_token=token, bot_username=username)

    if channel == ChannelType.LINE:
        stmt = select(Inbox).where(Inbox.channel_type == ChannelType.LINE)
        inbox = (await session.execute(stmt)).scalars().first()
        token = None
        channel_id = None
        secret = None
        if inbox and inbox.credentials:
            token = inbox.credentials.get("channelAccessToken")
            channel_id = inbox.credentials.get("channelId")
            secret = inbox.credentials.get("channelSecret")
        return LineProvider(
            channel_access_token=token, channel_id=channel_id, channel_secret=secret
        )

    if channel == ChannelType.VOICE:
        stmt = select(Inbox).where(Inbox.channel_type == ChannelType.VOICE)
        inbox = (await session.execute(stmt)).scalars().first()
        account_sid = None
        auth_token = None
        voice_number = None
        caller_id = None
        twiml_app_sid = None
        sip_domain = None
        recording_enabled = True
        transcription_enabled = True
        if inbox and inbox.credentials:
            account_sid = inbox.credentials.get("accountSid")
            auth_token = inbox.credentials.get("authToken")
            voice_number = inbox.phone_number or inbox.credentials.get("voiceNumber")
            caller_id = inbox.credentials.get("callerId") or voice_number
            twiml_app_sid = inbox.credentials.get("twimlAppSid")
            sip_domain = inbox.credentials.get("sipDomain")
            recording_enabled = inbox.credentials.get("recordingEnabled", True)
            transcription_enabled = inbox.credentials.get("transcriptionEnabled", True)
        return VoiceProvider(
            account_sid=account_sid,
            auth_token=auth_token,
            voice_number=voice_number,
            caller_id=caller_id,
            twiml_app_sid=twiml_app_sid,
            sip_domain=sip_domain,
            recording_enabled=recording_enabled,
            transcription_enabled=transcription_enabled,
        )

    if channel == ChannelType.TIKTOK:
        stmt = select(Inbox).where(Inbox.channel_type == ChannelType.TIKTOK)
        inbox = (await session.execute(stmt)).scalars().first()
        access_token = None
        app_id = None
        app_secret = None
        business_account_id = None
        if inbox and inbox.credentials:
            access_token = inbox.credentials.get("accessToken")
            app_id = inbox.credentials.get("appId")
            app_secret = inbox.credentials.get("appSecret")
            business_account_id = inbox.credentials.get("businessAccountId")
        return TikTokProvider(
            access_token=access_token,
            app_id=app_id,
            app_secret=app_secret,
            business_account_id=business_account_id,
        )

    try:
        provider_cls = _PROVIDERS[channel]
    except KeyError:
        raise ValueError(f"No channel provider configured for {channel}.") from None
    return provider_cls()


__all__ = [
    "ChannelProvider",
    "LineProvider",
    "MetaMessengerProvider",
    "SendResult",
    "TelegramProvider",
    "TikTokProvider",
    "TwilioSMSProvider",
    "UniversalEmailProvider",
    "VoiceProvider",
    "WebLiveChatProvider",
    "get_provider",
]
