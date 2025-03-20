from orpheus import Orpheus
from orpheus.models import Messages, Message

def test_native_chat_completion(orpheus: Orpheus):
    response = orpheus.chat.completions.create(
        model="gpt5",
        messages=Messages(messages=[Message(role="user", content="hello")]),
    )

    assert response is not None
    assert response.choices[0].message.content == "hello"


