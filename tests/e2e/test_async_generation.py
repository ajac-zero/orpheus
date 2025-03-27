from orpheus import AsyncOrpheus


async def test_async_chat_completion(async_orpheus: AsyncOrpheus):
    response = await async_orpheus.chat.completions.create(
        model="gpt5", messages=[{"role": "user", "content": "hello"}]
    )

    assert response is not None
    assert response.choices[0].message.content == "hello"


async def test_async_chat_stream_completion(async_orpheus: AsyncOrpheus):
    response = await async_orpheus.chat.completions.create(
        model="gpt5", messages=[{"role": "user", "content": "hello"}], stream=True
    )

    buffer = ""

    async for chunk in response:
        assert chunk.choices[0].delta.content is not None

        buffer += chunk.choices[0].delta.content

    assert buffer == "hello"


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


async def test_async_chat_stream_completion_with_extra_header(
    async_orpheus: AsyncOrpheus,
):
    response = await async_orpheus.chat.completions.create(
        model="gpt5",
        messages=[{"role": "user", "content": "hello"}],
        stream=True,
        extra_headers={"mock-response": "hi!"},
    )

    buffer = ""

    async for chunk in response:
        assert chunk.choices[0].delta.content is not None

        buffer += chunk.choices[0].delta.content

    assert buffer == "hi!"
