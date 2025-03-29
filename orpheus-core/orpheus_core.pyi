from dataclasses import dataclass
from typing import Literal

class OrpheusCore:
    """
    Thic class wraps all API clients (Chat Completion, Embedding) and allows inheritance
    by the synchronous Orpheus client.

    It is not meant to be used directly.

    >>> class Orpheus(OrpheusCore): ...
    """

    ...

    def create_embeddings(
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

    def create_chat_completion(
        self,
        model: str,
        messages: list[dict[str, str | list[dict[str, str]]]],
        stream: bool | None = None,
        extra_headers: dict[str, str] | None = None,
        extra_query: dict[str, str] | None = None,
        **kwargs,
    ) -> dict[str, str]: ...

@dataclass
class Embeddings:
    index: int
    embedding: list[float]
    object: Literal["embedding"]
