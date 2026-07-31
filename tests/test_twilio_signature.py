"""
The two vectors below (auth token, URL, params, expected signature) were
cross-checked against Twilio's own `twilio.request_validator.RequestValidator`
(from the official `twilio` PyPI package) before being hardcoded here --
that package isn't a project dependency, so the check isn't repeated at
test time, but the fixed values are a real reference implementation's
output, not invented.
"""

from communicator.webhooks.twilio_signature import (
    compute_twilio_signature,
    validate_twilio_signature,
)

AUTH_TOKEN = "12345"
URL = "https://mycompany.com/myapp.php?foo=1&bar=2"
PARAMS = {
    "CallSid": "CA1234567890ABCDE",
    "Caller": "+14158675308",
    "Digits": "1234",
    "From": "+14158675308",
    "To": "+18005551212",
}
EXPECTED_SIGNATURE = "RzzcfEcya7CLlEkW9rAuegtzCH4="


def test_matches_reference_implementation():
    assert compute_twilio_signature(AUTH_TOKEN, URL, PARAMS) == EXPECTED_SIGNATURE


def test_validates_matching_signature():
    assert validate_twilio_signature(AUTH_TOKEN, URL, PARAMS, EXPECTED_SIGNATURE) is True


def test_rejects_wrong_auth_token():
    assert validate_twilio_signature("wrong-token", URL, PARAMS, EXPECTED_SIGNATURE) is False


def test_rejects_tampered_params():
    tampered = dict(PARAMS, Digits="0000")
    assert validate_twilio_signature(AUTH_TOKEN, URL, tampered, EXPECTED_SIGNATURE) is False


def test_rejects_tampered_url():
    assert (
        validate_twilio_signature(AUTH_TOKEN, URL + "&extra=1", PARAMS, EXPECTED_SIGNATURE) is False
    )


def test_round_trip_for_arbitrary_params():
    url = "https://communicator.example.com/api/v1/webhooks/twilio/sms"
    params = {
        "From": "+923001234567",
        "To": "+15005550006",
        "Body": "STOP",
        "MessageSid": "SMxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    }
    signature = compute_twilio_signature("some-auth-token", url, params)
    assert validate_twilio_signature("some-auth-token", url, params, signature) is True
