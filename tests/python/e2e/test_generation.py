from orpheus import Orpheus


def test_chat_completion(orpheus: Orpheus):
    response = orpheus.chat.completions.create(
        model="gpt5", messages=[{"role": "user", "content": "hello"}]
    )

    assert response is not None
    assert response.choices[0].message.content == "hello"


def test_complex_chat_completion(orpheus: Orpheus):
    response = orpheus.chat.completions.create(
        model="gpt5",
        messages=[
            {"role": "system", "content": "You are a friendly bot"},
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "hello, whats in the image"},
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": "https://www.pawlovetreats.com/cdn/shop/articles/pembroke-welsh-corgi-puppy_2000x.jpg?v=1628638716"
                        },
                    },
                ],
            },
        ],
    )

    assert response is not None
    assert response.choices[0].message.content == "hello, whats in the image"


def test_chat_stream_completion(orpheus: Orpheus):
    response = orpheus.chat.completions.create(
        model="gpt5", messages=[{"role": "user", "content": "hello"}], stream=True
    )

    buffer = ""

    for chunk in response:
        print(chunk)
        assert chunk.choices[0].delta.content is not None

        buffer += chunk.choices[0].delta.content

    assert buffer == "hello"


def test_chat_completion_with_default_header(mockai):
    client = Orpheus(
        api_key="test",
        base_url=f"http://localhost:{mockai.get_exposed_port(8100)}/openai",
        default_headers={"mock-response": "wazza!"},
    )

    response = client.chat.completions.create(
        model="gpt5",
        messages=[{"role": "user", "content": "hello"}],
    )

    assert response is not None
    assert response.choices[0].message.content == "wazza!"


def test_chat_completion_with_extra_header(orpheus: Orpheus):
    response = orpheus.chat.completions.create(
        model="gpt5",
        messages=[{"role": "user", "content": "hello"}],
        extra_headers={"mock-response": "hi!"},
    )

    assert response is not None
    assert response.choices[0].message.content == "hi!"


def test_chat_stream_completion_with_extra_header(orpheus: Orpheus):
    response = orpheus.chat.completions.create(
        model="gpt5",
        messages=[{"role": "user", "content": "hello"}],
        stream=True,
        extra_headers={"mock-response": "hi!"},
    )

    buffer = ""

    for chunk in response:
        assert chunk.choices[0].delta.content is not None

        buffer += chunk.choices[0].delta.content

    assert buffer == "hi!"
