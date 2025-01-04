import orpheus


def test_chat_completion(mockai):
    client = orpheus.Orpheus(api_key="test", base_url="http://localhost:8100/openai")

    response = client.chat.completions.create(
        model="gpt5", messages=[{"role": "user", "content": "hello"}]
    )

    assert response is not None
    assert response.choices[0].message.content == "hello"


def test_chat_completion_with_default_header(mockai):
    client = orpheus.Orpheus(
        api_key="test",
        base_url="http://localhost:8100/openai",
        default_headers={"mock-response": "wazza!"},
    )

    response = client.chat.completions.create(
        model="gpt5",
        messages=[{"role": "user", "content": "hello"}],
    )

    assert response is not None
    assert response.choices[0].message.content == "wazza!"


def test_chat_completion_with_extra_header(mockai):
    client = orpheus.Orpheus(api_key="test", base_url="http://localhost:8100/openai")

    response = client.chat.completions.create(
        model="gpt5",
        messages=[{"role": "user", "content": "hello"}],
        extra_headers={"mock-response": "hi!"},
    )

    assert response is not None
    assert response.choices[0].message.content == "hi!"
