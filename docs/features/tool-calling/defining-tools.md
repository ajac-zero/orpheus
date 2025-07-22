# Defining Tools

Most providers expect tools to be defined using JSON Schema. While this allows for flexibility, it is prone to errors and requires you to remember the specific schema that is expected for a tool definition.

To combat this, Orpheus has full, type-safe support for tool calling in chat requests via the `Tool` and `Param` objects. These objects are then transformed into valid JSON Schema under the hood.

Say you previously had a Tool definition like this, in JSON Schema.

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

Lots of room to mistype a key or forget a coma, so let's use the Tool builder instead. The equivalent with Orpheus would be this.

```rust
Tool::function("get_weather")
    .description("Get current temperature for provided coordinates.")
    .with_parameters(|params| {
        params
            .property("latitude", Param::number().end())
            .property("longitude", Param::number().end())
            .required(["latitude", "longitude"])
    })
    .build()
```

As you can see, each Param also implements the builder pattern, so you can specify any nested fields.

Let's look at more examples, like this one that uses the enum key to limit the response to a set of values.

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

```rust
Tool::function("get_weather")
    .description("Retrieve current weather for the given location.")
    .with_parameters(|params| {
        params
            .property("location", Param::string()
                .description("The city to get weather for.")
                .end()
            )
            .property("units", Param::string()
                .description("Units the temperature will be returned in.")
                .r#enum(["celsius", "fahrenheit"])
                .end()
            )
            .required(["location", "units"])
    })
    .build()
```

This is a more complex example that uses array and object parameters.

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

```rust
Tool::function("get_weather")
    .description("Retrieve current weather for a set of given locations.")
    .with_parameters(|params| {
        params
            .property("locations", Param::array().items(
                Param::object()
                    .property("city", Param::string().description("The city to get weather for."))
                    .property("country", Param::string().description("The country the city is in."))
                    .required(["city", "country"])
                    .end(),
                )
                .end()
            )
            .property("units", Param::string()
                .description("Units the temperature will be returned in.")
                .r#enum(["celsius", "fahrenheit"])
                .end()
            )
    })
    .build()
```

If you have a Tool definition that is not supported by the current Tool builder, please submit an issue! The Tool builder aims to have complete support for all possible tool definitions (Though a word of warning, the more complex a tool is, the worse models will be at using it).

Now that we know how to define our tools, let's see how to use them.
