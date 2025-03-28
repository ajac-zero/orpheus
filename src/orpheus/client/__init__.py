from orpheus_core import OrpheusCore


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
        ...
