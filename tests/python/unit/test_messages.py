import pytest

from orpheus.models import Messages, Message

def test_user_messages():
    assert Message(role="user", content="hi!") is not None

def test_user_messages_validation_no_content():
    with pytest.raises(ValueError):
        assert Message(role="user") is not None # type: ignore

def test_none_conversation():
    with pytest.raises(TypeError):
        assert Messages() is not None

def test_empty_conversation():
    assert Messages([]) is not None

def test_conversation():
    assert Messages([Message(role="user", content="hi!")]) is not None
