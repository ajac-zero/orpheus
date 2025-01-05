import os
import openai

client = openai.OpenAI(
    api_key=os.getenv("ORPHEUS_API_KEY", "mock"),
    base_url=os.getenv("ORPHEUS_BASE_URL", "http://localhost:8100/openai"),
)

messages = [
    {"role": "system", "content": "You are a friendly bot"},
    {
        "role": "user",
        "content": [
            {"type": "text", "text": "hello, whats in the image"},
            {
                "type": "image_url",
                "image_url": {
                    "url": "https://www.pawlovetreats.com/cdn/shop/articles/pembroke-welsh-corgi-puppy_2000x.jpg?v=1628638716"
                },
            },
        ],
    },
]
model = "gpt-4o"

client.chat.completions.create(messages=messages, model=model)
