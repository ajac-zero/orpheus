from __future__ import annotations

from typing import Literal, overload

import orjson
from orpheus_core import ChatCompletion, Embeddings, OrpheusCore, Stream, Messages


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
        self.chat = self.Chat(orpheus=self)

    @overload
    def chat_completion(
        self,
        model: str,
        messages: list[dict[str, str | list[dict[str, str]]]] | Messages,
        stream: Literal[False] | None = None,
        extra_headers: dict[str, str] | None = None,
        extra_query: dict[str, str] | None = None,
        **kwargs,
    ) -> ChatCompletion:
        """This is the chat completion method"""
        ...

    @overload
    def chat_completion(
        self,
        model: str,
        messages: list[dict[str, str | list[dict[str, str]]]] | Messages,
        stream: Literal[True],
        extra_headers: dict[str, str] | None = None,
        extra_query: dict[str, str] | None = None,
        **kwargs,
    ) -> Stream:
        """This is the chat completion method"""
        ...

    def chat_completion(
        self,
        model: str,
        messages: list[dict[str, str | list[dict[str, str]]]] | Messages,
        stream: bool | None = None,
        extra_headers: dict[str, str] | None = None,
        extra_query: dict[str, str] | None = None,
        **kwargs,
    ) -> ChatCompletion | Stream:
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
    ) -> Embeddings:
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

    class Chat:
        class Completions:
            def __init__(self, orpheus: Orpheus):
                self.create = orpheus.chat_completion

        def __init__(self, orpheus: Orpheus):
            self.completions = self.Completions(orpheus=orpheus)
            self.create = Orpheus.chat_completion
