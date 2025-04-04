from __future__ import annotations

from typing import Any, Literal, overload

import orjson
from orpheus_core import (
    ChatCompletion,
    Embeddings,
    OrpheusCore,
    Message,
    AsyncOrpheusCore,
)
from pydantic import BaseModel

from orpheus.types import MappedMessages
from orpheus._utils import tools_into_bytes


type Messages = list[Message]


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
    def message(
        self,
        model: str,
        messages: MappedMessages | Messages,
        stream: Literal[False] | None = None,
        tools: list[dict] | list[BaseModel] | None = None,
        extra_headers: dict[str, str] | None = None,
        extra_query: dict[str, str] | None = None,
        **kwargs,
    ) -> ChatCompletion:
        """This is the chat completion method"""
        ...

    @overload
    def message(
        self,
        model: str,
        messages: MappedMessages | Messages,
        stream: Literal[True],
        tools: list[dict] | list[BaseModel] | None = None,
        extra_headers: dict[str, str] | None = None,
        extra_query: dict[str, str] | None = None,
        **kwargs,
    ) -> Any:
        """This is the chat completion method"""
        ...

    def message(
        self,
        model: str,
        messages: MappedMessages | Messages,
        stream: bool | None = None,
        tools: list[dict] | list[BaseModel] | None = None,
        extra_headers: dict[str, str] | None = None,
        extra_query: dict[str, str] | None = None,
        **kwargs,
    ) -> ChatCompletion | Any:
        """This is the chat completion method"""
        return self.native_chat_completions_create(
            model=model,
            messages=messages,
            stream=stream,
            tools=tools_into_bytes(tools) if tools else None,
            extra_headers=extra_headers,
            extra_query=extra_query,
            extra=orjson.dumps(kwargs),
        )

    def embed(
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
        return self.native_embeddings_create(
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
                self.create = orpheus.message

        def __init__(self, orpheus: Orpheus):
            self.completions = self.Completions(orpheus=orpheus)
            self.create = orpheus.message


class AsyncOrpheus(AsyncOrpheusCore):
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
    async def message(
        self,
        model: str,
        messages: MappedMessages | Messages,
        stream: Literal[False] | None = None,
        extra_headers: dict[str, str] | None = None,
        extra_query: dict[str, str] | None = None,
        **kwargs,
    ) -> ChatCompletion:
        """This is the chat completion method"""
        ...

    @overload
    async def message(
        self,
        model: str,
        messages: MappedMessages | Messages,
        stream: Literal[True],
        extra_headers: dict[str, str] | None = None,
        extra_query: dict[str, str] | None = None,
        **kwargs,
    ) -> Any:
        """This is the chat completion method"""
        ...

    async def message(
        self,
        model: str,
        messages: MappedMessages | Messages,
        stream: bool | None = None,
        extra_headers: dict[str, str] | None = None,
        extra_query: dict[str, str] | None = None,
        **kwargs,
    ) -> ChatCompletion | Any:
        """This is the chat completion method"""
        return await self.native_chat_completions_create(
            model=model,
            messages=messages,
            stream=stream,
            extra_headers=extra_headers,
            extra_query=extra_query,
            extra=orjson.dumps(kwargs),
        )

    async def embed(
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
        return await self.native_embeddings_create(
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
            def __init__(self, orpheus: AsyncOrpheus):
                self.create = orpheus.message

        def __init__(self, orpheus: AsyncOrpheus):
            self.completions = self.Completions(orpheus=orpheus)
            self.create = orpheus.message
