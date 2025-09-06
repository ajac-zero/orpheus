---
description: Learn how to define your tools with Orpheus
---

# Defining Tools

Most providers expect tools to be defined using JSON Schema. While this allows for flexibility, it is prone to errors and requires you to remember the specific schema that is expected for a tool definition.

To improve this, Orpheus has full, type-safe support for tool definitions via the `Tool` and `Param` objects. These objects are then transformed into a valid JSON Schema under the hood.

Check out this example of a basic tool definition in JSON and the equivalent with Orpheus:

{% tabs %}
{% tab title="JSON" %}
```json
{
    "type": "function",
    "name": "get_weather",
    "description": "Get current temperature for provided coordinates.",
    "parameters": {
        "type": "object",
        "properties": {
            "latitude": {
              "type": "number"
            },
            "longitude": {
              "type": "number"
            }
        },
        "required": ["latitude", "longitude"]
    }
}
```
{% endtab %}

{% tab title="Rust" %}
```rust
Tool::function("get_weather")
    .description("Get current temperature for provided coordinates.")
    .with_parameters(|params| {
        params
            .property("latitude", Param::number())
            .property("longitude", Param::number())
            .required(["latitude", "longitude"])
    })
    .build()
```
{% endtab %}
{% endtabs %}

Each Param also implements the builder pattern, so you can specify any nested fields.

Let's look at more examples, like this one that uses the enum key to limit the response to a set of values.

{% tabs %}
{% tab title="JSON" %}
```json
{
  "type": "function",
  "function": {
    "name": "get_weather",
    "description": "Retrieves current weather for the given location.",
    "parameters": {
      "type": "object",
      "properties": {
        "location": {
          "type": "string",
          "description": "City and country e.g. Bogotá, Colombia"
        },
        "units": {
          "type": "string",
          "description": "Units the temperature will be returned in.",
          "enum": ["celsius", "fahrenheit"]
        }
      },
      "required": ["location", "units"]
    }
  }
}
```
{% endtab %}

{% tab title="Rust" %}
```rust
Tool::function("get_weather")
    .description("Retrieve current weather for the given location.")
    .with_parameters(|params| {
        params
            .property(
                "location",
                Param::string().description("The city to get weather for.")
            )
            .property(
                "units",
                Param::string()
                    .description("Units the temperature will be returned in.")
                    .enums(["celsius", "fahrenheit"])
            )
            .required(["location", "units"])
    })
    .build()
```
{% endtab %}
{% endtabs %}

This is a more complex example that uses array and object parameters.

{% tabs %}
{% tab title="JSON" %}
```json
{
  "type": "function",
  "function": {
    "name": "get_weather",
    "description": "Retrieve current weather for a set of given locations.",
    "parameters": {
      "type": "object",
      "properties": {
        "locations": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "city": {
                "type": "string",
                "description": "The city to get weather for."
              },
              "country": {
                "type": "string",
                "description": "The country the city is in."
              }
            },
            "required": ["city", "country"]
          }
        },
        "units": {
          "type": "string",
          "description": "Units the temperature will be returned in.",
          "enum": ["celsius", "fahrenheit"]
        }
      },
      required: ["locations", "units"]
    }
  }
}
```
{% endtab %}

{% tab title="Rust" %}
```rust
Tool::function("get_weather")
    .description("Retrieve current weather for a set of given locations.")
    .with_parameters(|params| {
        params
            .property(
                "locations",
                Param::array().items(
                    Param::object()
                        .property(
                            "city",
                            Param::string().description("The city to get weather for.")
                        )
                        .property(
                            "country",
                            Param::string().description("The country the city is in.")
                        )
                        .required(["city", "country"])
                )
            )
            .property(
                "units",
                Param::string()
                    .description("Units the temperature will be returned in.")
                    .enums(["celsius", "fahrenheit"])
            )
    })
    .build()
```
{% endtab %}
{% endtabs %}

If you have a Tool definition that is not supported by the current Tool builder, please submit an issue! The Tool builder aims to have complete support for all possible tool definitions (Though a word of warning, the more complex a tool is, the worse models will be at using it).

Now that we know how to define our tools, let's see how to use them.
