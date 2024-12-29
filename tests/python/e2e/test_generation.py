import orpheus


def test_chat_completion(mockai):
    client = orpheus.Orpheus(api_key="test", base_url="http://localhost:8100/openai")

    response = client.chat.completions.create(
        model="gpt5", messages=[{"role": "user", "content": "hello"}]
    )

    assert response is not None
