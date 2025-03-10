from orpheus import Orpheus, AsyncOrpheus


def test_chat_completion(orpheus: Orpheus):
    response = orpheus.chat.completions.create(
        model="gpt5", messages=[{"role": "user", "content": "hello"}]
    )

    assert response is not None
    assert response.choices[0].message.content == "hello"


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


async def test_async_chat_completion(async_orpheus: AsyncOrpheus):
    response = await async_orpheus.chat.completions.create(
        model="gpt5", messages=[{"role": "user", "content": "hello"}]
    )

    assert response is not None
    assert response.choices[0].message.content == "hello"


async def test_async_chat_completion_with_default_header(mockai):
    client = AsyncOrpheus(
        api_key="test",
        base_url=f"http://localhost:{mockai.get_exposed_port(8100)}/openai",
        default_headers={"mock-response": "wazza!"},
    )

    response = await client.chat.completions.create(
        model="gpt5",
        messages=[{"role": "user", "content": "hello"}],
    )

    assert response is not None
    assert response.choices[0].message.content == "wazza!"


async def test_async_chat_completion_with_extra_header(async_orpheus: AsyncOrpheus):
    response = await async_orpheus.chat.completions.create(
        model="gpt5",
        messages=[{"role": "user", "content": "hello"}],
        extra_headers={"mock-response": "hi!"},
    )

    assert response is not None
    assert response.choices[0].message.content == "hi!"
