import pytest

from communicator.compliance.optout import is_opt_in_message, is_opt_out_message


@pytest.mark.parametrize("body", ["STOP", "stop", "  Stop  ", "unsubscribe", "opt out", "cancel", "remove me"])
def test_recognizes_opt_out_keywords(body):
    assert is_opt_out_message(body) is True


@pytest.mark.parametrize(
    "body",
    [
        "please help me stop paying too much",
        "when does the free trial stop?",
        "Tell me more about your services",
        "",
    ],
)
def test_does_not_false_positive_on_opt_out(body):
    assert is_opt_out_message(body) is False


@pytest.mark.parametrize("body", ["START", "subscribe", "opt in", "yes"])
def test_recognizes_opt_in_keywords(body):
    assert is_opt_in_message(body) is True


def test_opt_in_and_opt_out_are_mutually_exclusive_for_keywords():
    for body in ["stop", "unsubscribe", "cancel"]:
        assert is_opt_in_message(body) is False
    for body in ["start", "subscribe", "yes"]:
        assert is_opt_out_message(body) is False
