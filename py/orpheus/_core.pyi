from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Generator, Literal

class OrpheusCore:
    """
    Thic class wraps all API clients (Chat Completion, Embedding) and allows inheritance
    by the synchronous Orpheus client.

    It is not meant to be used directly.

    >>> class Orpheus(OrpheusCore): ...
    """

    ...

    def native_embeddings_create(
        self,
        input: str | list[str] | list[int] | list[list[int]],
        model: str,
        dimensions: int | None = None,
        encoding_format: str | None = None,
        user: str | None = None,
        extra_headers: dict[str, str] | None = None,
        extra_query: dict[str, str] | None = None,
    ) -> Embeddings:
        """
        filler
        """
        ...

    def native_chat_completions_create(
        self,
        model: str,
        messages: Any | Messages,
        stream: bool | None = None,
        extra_headers: dict[str, str] | None = None,
        extra_query: dict[str, str] | None = None,
        **kwargs,
    ) -> ChatCompletion: ...

@dataclass
class Embeddings:
    index: int
    embedding: list[float]
    object: Literal["embedding"]

@dataclass
class ChatCompletion:
    id: str
    choices: list[Choice]
    created: int
    model: str
    service_tier: str | None
    system_fingerprint: str | None
    object: Literal["chat.completion"]
    usage: PromptUsage

@dataclass
class Choice:
    index: int
    message: Message
    finish_reason: str
    logprobs: LogProbs | None

@dataclass
class TopLogProbs:
    token: str
    logprob: int
    bytes: bytes | None

@dataclass
class Content:
    token: str
    logprob: int
    bytes: bytes | None
    top_logprobs: TopLogProbs

@dataclass
class Refusal:
    token: str
    logprob: int
    bytes: bytes | None
    top_logprobs: TopLogProbs

@dataclass
class LogProbs:
    content: list[Content] | None
    refusal: list[Refusal] | None

@dataclass
class PromptUsage:
    completion_tokens: int
    prompt_tokens: int
    total_tokens: int
    completion_tokens_details: CompletionTokensDetails | None
    prompt_tokens_details: PromptTokensDetails | None

@dataclass
class CompletionTokensDetails:
    accepted_prediction_tokens: int | None
    audio_tokens: int | None
    reasoning_tokens: int | None
    rejected_prediction_tokens: int | None

@dataclass
class PromptTokensDetails:
    audio_tokens: int | None
    cached_tokens: int | None

class Message:
    """
    The Message class represents a single message in a conversation with an LLM.

    It does not have an initializer. Instead, it exposes the subclasses for the
    different message types:

    >>> Message.System
    >>> Message.User
    >>> Message.Assistant
    >>> Message.Tool
    """

    role: str | None
    content: str | None
    tool_calls: list[ToolCall] | None
    tool_id: str | None

    class System:
        """
        A System message accepts a 'content' string parameter.

        This string serves as the guidelines for how the LLM should behave,
        during the conversation.

        Exposed attributes:
        - role: str
        - content: str

        >>> system_message = Message.System(content="You are a helpful assistant.")

        >>> assert system_message.role == "system"
        >>> assert system_message.content == "You are a helpful assistant."
        """

        role: Literal["system"]
        content: str

        def __init__(self, content: str) -> None: ...

    class User:
        """
        A User message accepts a 'content' parameter of type 'str | list[orpheus.models.Part]'.

        For most cases you'll can pass just a string; This is a 'Simple' variant.
        However, if you want to include an image within your prompt, you'll need to pass
        a list of orpheus.models.Part objects; This is a 'Complex' variant.

        Simple variant:
        >>> message = Message.User(content="Hello, how are you?")

        >>> assert message.role == "user"
        >>> assert message.content == "Hello, how are you?"

        Complex variant:
        >>> message = Message.User(
        >>>     content=[
        >>>         Part.Text(text="Describe this image."),
        >>>         Part.Image(url="https://example.com/image.png")
        >>>     ]
        >>> )

        >>> assert message.role == "user"
        >>> assert len(message.content) == 2
        >>> assert message.content[0].type == "text"
        >>> assert message.content[0].text == "Describe this image."
        >>> assert message.content[1].type == "image_url"
        >>> assert message.content[1].image_url.url == "https://example.com/image.png"
        """

        role: Literal["user"]
        content: str | list[PartType]

        def __init__(self, content: str | list[PartType]) -> None: ...

    class Assistant:
        role: str
        content: str | list[PartType]
        tool_calls: list[ToolCall] | None

        def __init__(
            self, content: str | list[PartType], tool_calls: list[ToolCall] | None = None
        ) -> None: ...

    class Tool:
        role: str
        content: str
        tool_id: str

        def __init__(self, content: str, tool_id: str) -> None: ...

class Messages(list[Message]):
    def __init__(self, *args: Message): ...

@dataclass
class ToolCall:
    id: str
    type: str
    function: Function

    def __init__(self, id: str, name: str, arguments: dict[str, Any]) -> None: ...

@dataclass
class Function:
    name: str
    arguments: str

@dataclass
class ImageUrl:
    url: str
    detail: str | None

class Part:
    class Text:
        text: str

        def __init__(self, text: str) -> None: ...

    class Image:
        url: str
        detail: str | None
        image_url: ImageUrl

        def __init__(self, url: str, detail: str | None = None) -> None: ...

type PartType = Part.Text | Part.Image

@dataclass
class StreamUsage:
    completion_tokens: int
    prompt_tokens: int
    total_tokens: int

@dataclass
class ChoiceChunk:
    index: int
    delta: Delta
    finish_reason: str | None
    logprobs: LogProbs | None

@dataclass
class ChatCompletionChunk:
    id: str
    choices: list[ChoiceChunk]
    created: int
    model: str
    service_tier: str | None
    system_fingerprint: str | None
    object: str
    usage: StreamUsage | None

@dataclass
class Delta:
    role: str | None
    content: str | list[Part] | None
    refusal: str | None
    tool_calls: list[ToolCall] | None

type Stream = Generator[ChatCompletionChunk]
