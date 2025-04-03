import os
import re

import pytest

from orpheus import Orpheus

BASE_URL_EXCEPTION = re.escape(
    "No base URL provided and none found in environment variables:"
)

API_KEY_EXCEPTION = re.escape(
    "No API key provided and none found in environment variables:"
)


class EnvContextManager:
    def __init__(self, **env_vars):
        """Initialize with environment variables to set."""
        self.env_vars = env_vars
        self.original_env_vars = {}

    def __enter__(self):
        """Set the environment variables and store their original values."""
        for key, value in self.env_vars.items():
            self.original_env_vars[key] = os.environ.get(key)  # Store original value
            os.environ[key] = value  # Set new value

    def __exit__(self, exc_type, exc_value, traceback):
        """Restore original environment variables."""
        for key in self.env_vars:
            if self.original_env_vars[key] is None:
                del os.environ[key]  # Remove if it wasn't originally set
            else:
                os.environ[key] = self.original_env_vars[key]  # Restore original value


def test_create_client_no_env():
    with pytest.raises(RuntimeError, match=BASE_URL_EXCEPTION):
        Orpheus()


def test_create_client_orpheus_url_env():
    with EnvContextManager(ORPHEUS_BASE_URL="https://example.com"):
        with pytest.raises(RuntimeError, match=API_KEY_EXCEPTION):
            Orpheus()


def test_create_client_orpheus_key_env():
    with EnvContextManager(ORPHEUS_API_KEY="empty"):
        with pytest.raises(RuntimeError, match=BASE_URL_EXCEPTION):
            Orpheus()


def test_create_client_openai_url_env():
    with EnvContextManager(OPENAI_BASE_URL="https://example.com"):
        with pytest.raises(RuntimeError, match=API_KEY_EXCEPTION):
            Orpheus()


def test_create_client_openai_key_env():
    with EnvContextManager(OPENAI_API_KEY="empty"):
        with pytest.raises(RuntimeError, match=BASE_URL_EXCEPTION):
            Orpheus()


def test_create_client_orpheus_envs():
    with EnvContextManager(
        ORPHEUS_BASE_URL="https://example.com",
        ORPHEUS_API_KEY="empty",
    ):
        orpheus = Orpheus()

        assert isinstance(orpheus, Orpheus)


def test_create_client_openai_envs():
    with EnvContextManager(
        OPENAI_BASE_URL="https://example.com",
        OPENAI_API_KEY="empty",
    ):
        orpheus = Orpheus()

        assert isinstance(orpheus, Orpheus)


def test_create_client_vars():
    orpheus = Orpheus(
        base_url="https://example.com",
        api_key="empty",
    )

    assert isinstance(orpheus, Orpheus)
