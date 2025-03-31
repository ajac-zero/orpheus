from typing import Literal, TypedDict, Mapping, Any

type ImageUrl = Mapping[Literal["url", "detail"], str | None]

type Text = Mapping[Literal["text"], str]

type Image = Mapping[Literal["image"], ImageUrl]

type Part = Text | Image

type MappedMessage = Mapping[Literal["role", "content", "tool_calls", "tool_id"], str | list[Part] | Any | None]

type MappedMessages = list[MappedMessage]
