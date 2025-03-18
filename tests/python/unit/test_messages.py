from orpheus.models import Conversation, Message

def test_user_messages():
    assert Message(role="user", content="hi!") is not None

def test_user_messages_validation_no_content():
    assert Message(role="user") is not None

def test_conversation():
    assert Conversation([Message(role="user", content="hi!")]) is not None
