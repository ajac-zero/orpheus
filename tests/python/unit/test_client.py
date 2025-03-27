import os

import pytest

from orpheus import Orpheus


def test_create_client_no_env():
    with pytest.raises(
        KeyError,
        match="environment variable not found.",
    ):
        Orpheus()


def test_create_client_orpheus_url_env():
    os.environ["ORPHEUS_BASE_URL"] = "https://example.com"

    with pytest.raises(
        KeyError,
        match="environment variable not found.",
    ):
        Orpheus()
