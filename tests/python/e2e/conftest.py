import pytest

import subprocess
import time


@pytest.fixture(scope="module")
def mockai():
    server_ps = subprocess.Popen(["uvx", "ai-mock", "server", "--port", "8100"])
    time.sleep(3)

    yield server_ps

    server_ps.terminate()

    server_ps.wait()
