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
    role: str | None
    content: str | None
    tool_calls: list[ToolCall] | None
    tool_id: str | None

    def __init__(
        self,
        role: str | None = None,
        content: str | None = None,
        tool_calls: list[ToolCall] | None = None,
        tool_id: str | None = None,
    ) -> None: ...

    class System:
        role: str
        content: str

        def __init__(self, content: str) -> None: ...

    class User:
        role: str
        content: str | list[Part]

        def __init__(self, content: str | list[Part]) -> None: ...

    class Assistant:
        role: str
        content: str | list[Part]
        tool_calls: list[ToolCall] | None

        def __init__(
            self, content: str | list[Part], tool_calls: list[ToolCall] | None = None
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

@dataclass
class Function:
    name: str
    arguments: str

@dataclass
class ImageUrl:
    url: str
    detail: str | None

@dataclass
class TextPart:
    text: str

@dataclass
class ImagePart:
    image_url: ImageUrl

type Part = TextPart | ImagePart

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
