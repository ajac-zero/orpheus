import pytest

from orpheus.models import Messages, Message


def test_generic_messages():
    message = Message(role="user", content="hi!")

    assert message is not None
    assert message.role == "user"
    assert message.content == "hi!"


def test_user_messages():
    message = Message.User(content="hi!")

    assert message is not None
    assert message.role == "user"
    assert message.content == "hi!"


def test_user_messages_validation_no_content():
    with pytest.raises(ValueError):
        assert Message(role="user") is not None  # type: ignore


def test_none_conversation():
    with pytest.raises(TypeError):
        assert Messages() is not None


def test_empty_conversation():
    assert Messages([]) is not None


def test_conversation():
    assert Messages([Message(role="user", content="hi!")]) is not None
