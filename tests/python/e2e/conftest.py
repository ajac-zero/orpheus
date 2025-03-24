import pytest

from testcontainers.generic import ServerContainer
from testcontainers.core.waiting_utils import wait_for_logs

from orpheus import Orpheus, AsyncOrpheus


@pytest.fixture(scope="session")
def mockai():
    with ServerContainer(8100, "ajaczero/mock-ai") as container:
        wait_for_logs(container, "Uvicorn running")
        yield container


@pytest.fixture()
def orpheus(mockai):
    return Orpheus(
        api_key="test",
        base_url=f"http://localhost:{mockai.get_exposed_port(8100)}/openai",
    )


@pytest.fixture()
def async_orpheus(mockai):
    return AsyncOrpheus(
        api_key="test",
        base_url=f"http://localhost:{mockai.get_exposed_port(8100)}/openai",
    )
