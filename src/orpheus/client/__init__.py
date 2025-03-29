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
        chat_completions = SimpleNamespace(create=self.create_chat_completion)
        self.chat = SimpleNamespace(completions=chat_completions)
