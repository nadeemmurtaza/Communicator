from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    database_url: str = "postgresql+asyncpg://postgres:development-only@localhost:5432/communicator"
    test_database_url: str = (
        "postgresql+asyncpg://communicator_test:TEST_PASSWORD@127.0.0.1:5432/communicator_test"
    )
    app_env: str = "development"

    # Harvester integration -- Communicator only ever talks to Harvester's
    # API, never its database directly. See docs/ARCHITECTURE.md.
    harvester_api_base_url: str = "http://localhost:8000/api/v1"
    harvester_api_token: str = ""

    # Reply drafting
    anthropic_api_key: str = ""

    # Channel providers
    twilio_account_sid: str = ""
    twilio_auth_token: str = ""
    twilio_sms_from: str = ""
    twilio_whatsapp_from: str = ""
    email_provider_api_key: str = ""
    email_from_address: str = ""

    # Shared secret Postmark's inbound-webhook URL must include as
    # ?token=... -- Postmark doesn't sign inbound webhook requests, unlike
    # Twilio, so this is the only thing standing between the endpoint and
    # anyone who finds the URL.
    postmark_inbound_webhook_secret: str = ""

    dev_auth_enabled: bool = False

    model_config = SettingsConfigDict(env_file=".env", env_file_encoding="utf-8", extra="ignore")


def validate_test_database_url(test_url: str, production_url: str) -> None:
    if test_url == production_url:
        raise RuntimeError("TEST_DATABASE_URL must not match DATABASE_URL.")
    if "test" not in test_url.lower():
        raise RuntimeError("Test database name must contain 'test'.")


settings = Settings()
