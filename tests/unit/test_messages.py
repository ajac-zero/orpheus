import pytest

from orpheus.models import Message, ToolCall, Part


def test_system_message():
    message = Message.System(content="You are a helpful assistant.")

    assert message is not None
    assert message.role == "system"
    assert message.content == "You are a helpful assistant."


def test_system_message_validation():
    # Raises error if no content is provided
    with pytest.raises(TypeError):
        Message.System()  # type: ignore argument

    # Raises error if different kwarg is provided
    with pytest.raises(TypeError):
        Message.System(message="hello!")  # type: ignore argument


def test_user_message():
    message = Message.User(content="hi!")

    assert message is not None
    assert message.role == "user"
    assert message.content == "hi!"


def test_user_complex_message():
    message = Message.User(
        content=[
            Part.Text(text="hi!"),
            Part.Image(url="https://example.com/image.png"),
        ]
    )

    assert len(message.content) == 2

    assert isinstance(message.content[0], Part.Text)
    assert message.content[0].text == "hi!"

    assert isinstance(message.content[1], Part.Image)
    assert message.content[1].url == "https://example.com/image.png"


def test_user_message_validation():
    # Raises error if no content is provided
    with pytest.raises(TypeError):
        Message.User()  # type: ignore argument

    # Raises error if different kwarg is provided
    with pytest.raises(TypeError):
        Message.User(message="hello!")  # type: ignore argument


def test_assistant_message():
    message = Message.Assistant(
        content="I will look it up for you",
        tool_calls=[
            ToolCall(
                id="123",
                name="look_up",
                arguments={"resource": "wikipedia", "query": "rust"},
            ),
        ],
    )

    assert message is not None
    assert message.role == "assistant"
    assert message.content == "I will look it up for you"

    assert message.tool_calls is not None
    assert len(message.tool_calls) == 1
    assert message.tool_calls[0].id == "123"
    assert message.tool_calls[0].function.name == "look_up"
    assert message.tool_calls[0].function.arguments == {
        "resource": "wikipedia",
        "query": "rust",
    }


def test_conversation():
    messages = [
        Message.System(content="You are a helpful assistant."),
        Message.User(content="hi!"),
        Message.Assistant(content="hello!"),
        Message.User(content="bye!"),
    ]

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
