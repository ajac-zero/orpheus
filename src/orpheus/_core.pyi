from __future__ import annotations

class Orpheus:
    def __init__(
        self,
        *,
        api_key: str | None = None,
        base_url: str | None = None,
    ) -> None:
        """Construct a new synchronous openai client instance.

        This automatically infers the following arguments from their corresponding environment variables if they are not provided:
        - `api_key` from `ORPHEUS_API_KEY`
        - `base_url` from `ORPHEUS_PROJECT_ID`
        """

    @property
    def chat(self) -> Orpheus:
        return self

    @property
    def completions(self) -> Orpheus:
        return self

    def create(self): ...
