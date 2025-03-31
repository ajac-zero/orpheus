from typing import Literal, TypedDict

class ImageUrl(TypedDict):
    url: str
    detail: str | None

class TextPart(TypedDict):
    text: str

class ImagePart(TypedDict):
    image_url: ImageUrl

type Part = TextPart | ImagePart

class MappedMessage(TypedDict):
    role: Literal["system", "user", "assistant", "tool"]
    content: str | list[Part]
    tool_calls: list[dict[str, str | dict[str, str]]] | None
    tool_id: str | None

type MappedMessages = list[MappedMessage]
