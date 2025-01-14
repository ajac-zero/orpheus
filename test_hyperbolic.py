from openai import OpenAI
from orpheus import Orpheus
from orpheus._core import UnauthorizedError

client = Orpheus(
    api_key="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhamNhcmRvemEyMDAwQGdtYWlsLmNvbSJ9.wp8RSwupOpV0aDAAwhp4_lV0lB-BV9WPYMwXH10p10o",
    base_url="https://api.hyperbolic.xyz/v1",
)

try:
    r = client.chat.completions.create(
        model="NousResearch/Hermes-3-Llama-3.1-70B",
        messages=[
            {"role": "user", "content": "Hello, how are you?"},
        ],
    )

    print(r.choices[0].message.content)
except UnauthorizedError as e:
    print(e)
