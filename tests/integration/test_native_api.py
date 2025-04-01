from orpheus import Orpheus
from orpheus.models import Message


def test_native_chat_completion(orpheus: Orpheus):
    response = orpheus.message(
        model="gpt5",
        messages=[Message.User(content="hello")],
    )

    assert response is not None
    assert response.choices[0].message.content == "hello"
