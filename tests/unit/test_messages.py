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


def test_empty_conversation():
    messages = Messages()

    assert messages is not None
    assert len(messages) == 0


def test_conversation():
    messages = Messages(
        Message.System(content="You are a helpful assistant."),
        Message(role="user", content="hi!"),
        Message.Assistant(content="hello!"),
        Message.User(content="bye!"),
    )

    assert messages is not None
    assert len(messages) == 4

    assert messages[0].role == "system"
    assert messages[0].content == "You are a helpful assistant."

    assert messages[1].role == "user"
    assert messages[1].content == "hi!"

    assert messages[2].role == "assistant"
    assert messages[2].content == "hello!"

    assert messages[3].role == "user"
    assert messages[3].content == "bye!"
