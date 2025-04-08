from pydantic import BaseModel
import orjson


def model_into_tool(
    model: type[BaseModel],
    *,
    name: str | None = None,  # inferred from class name by default
    description: str | None = None,  # inferred from class docstring by default
):
    if description is None:
        # note: we intentionally don't use `.getdoc()` to avoid
        # including pydantic's docstrings
        description = model.__doc__

    function = {
        "name": name or model.__name__,
        "strict": True,
        "parameters": model.model_json_schema(),
    }

    if description is not None:
        function["description"] = description

    return {
        "type": "function",
        "function": function,
    }


def tools_into_bytes(tools: list[dict] | list[BaseModel]) -> bytes:
    return orjson.dumps(
        [
            model_into_tool(type(tool)) if isinstance(tool, BaseModel) else tool
            for tool in tools
        ]
    )
