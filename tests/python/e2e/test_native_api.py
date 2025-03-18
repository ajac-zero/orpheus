from orpheus import Orpheus
from orpheus.models import Conversation, Message

def test_native_chat_completion(orpheus: Orpheus):
    response = orpheus.chat.completions.create(
        model="gpt5",
        messages=Conversation(messages=[Message(role="user", content="hello")]),
    )

    assert response is not None
    assert response.choices[0].message.content == "hello"


