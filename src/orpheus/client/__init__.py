import orjson

from orpheus_core import OrpheusCore
from types import SimpleNamespace


class Orpheus(OrpheusCore):
    """
    Hola
    """

    def __init__(
        self,
        *,
        api_key: str | None = None,
        base_url: str | None = None,
        default_headers: dict[str, str] | None = None,
        default_query: dict[str, str] | None = None,
    ):
        """This is the constructor"""
        chat_completions = SimpleNamespace(create=self.chat_completion)
        self.chat = SimpleNamespace(
            completions=chat_completions, create=self.chat_completion
        )

    def chat_completion(
        self,
        model: str,
        messages: list[dict[str, str | list[dict[str, str]]]],
        stream: bool | None = None,
        extra_headers: dict[str, str] | None = None,
        extra_query: dict[str, str] | None = None,
        **kwargs,
    ) -> dict[str, str]:
        """This is the chat completion method"""
        return self.create_chat_completion(
            model=model,
            messages=messages,
            stream=stream,
            extra_headers=extra_headers,
            extra_query=extra_query,
            extra=orjson.dumps(kwargs),
        )

    def embeddings(
        self,
        input: str | list[str] | list[int] | list[list[int]],
        model: str,
        dimensions: int | None = None,
        encoding_format: str | None = None,
        user: str | None = None,
        extra_headers: dict[str, str] | None = None,
        extra_query: dict[str, str] | None = None,
    ) -> dict[str, str]:
        """This is the embedding completion method"""
        return self.create_embeddings(
            input=input,
            model=model,
            dimensions=dimensions,
            encoding_format=encoding_format,
            user=user,
            extra_headers=extra_headers,
            extra_query=extra_query,
        )
