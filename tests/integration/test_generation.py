from orpheus import Orpheus
from pydantic import BaseModel


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


def test_chat_completion_with_tools_pydantic_model(orpheus: Orpheus):
    class CallMom(BaseModel):
        number: int
        name: str

    call_mom_tool = CallMom(number=1, name="mom")

    response = orpheus.chat.completions.create(
        model="gpt5",
        messages=[{"role": "user", "content": "hello"}],
        tools=[call_mom_tool],
        extra_headers={
            "mock-response": 'f:{"name": "call_mom", "arguments": {"number": "1"}}'
        },
    )

    assert response is not None
    assert response.choices[0].message.content == ""

    assert (tool_calls := response.choices[0].message.tool_calls) is not None

    assert tool_calls[0].function.name == "call_mom"
    assert tool_calls[0].function.arguments == {"number": "1"}


def test_chat_completion_with_tools_schema(orpheus: Orpheus):
    response = orpheus.chat.completions.create(
        model="gpt5",
        messages=[{"role": "user", "content": "hello"}],
        extra_headers={
            "mock-response": 'f:{"name": "call_mom", "arguments": {"number": "1"}}'
        },
        tools=[
            {
                "type": "function",
                "function": {
                    "name": "call_mom",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "number": {"type": "integer"},
                            "name": {"type": "string"},
                        },
                    },
                },
            }
        ],
    )

    assert response is not None
    assert response.choices[0].message.content == ""

    assert (tool_calls := response.choices[0].message.tool_calls) is not None

    assert tool_calls[0].function.name == "call_mom"
    assert tool_calls[0].function.arguments == {"number": "1"}
