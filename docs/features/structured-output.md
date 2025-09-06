---
icon: box-open
---

# Structured Output

Structured output allows you to request JSON responses that follow a specific schema from language models. This ensures that the model's response conforms to your expected data structure, making it easier to parse and use in your applications.

Orpheus provides a powerful and ergonomic API for structured output through the `Format` type. You can define JSON schemas that specify exactly what structure you want the model to return, including property types, descriptions, and requirements.

## Basic Usage

To use structured output, create a `Format` using the builder pattern and attach it to your chat request:

{% code title="set_response_format.rs" %}
```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    // Define your response format
    let person_format = Format::json("person")
        .with_schema(|schema| {
            schema
                .property("name", Param::string())
                .property("age", Param::number())
                .required(["name", "age"])
        })
        .build();

    let response = client
        .chat("Jessica is a 20 year old college student.")
        .model("openai/gpt-4o")
        .response_format(person_format)
        .send()?;

    println!("{}", response.content()?);
    // Output: {"name": "Jessica", "age": 20}

    Ok(())
}
```
{% endcode %}

## Schema Definition

### Format Builder

The `Format::json()` method starts a JSON schema format builder:

```rust
let format = Format::json("schema_name")
    .strict(true)  // Optional: enforce strict mode (default: true)
    .with_schema(|schema| {
        // Define your schema here
        schema
            .property("field1", Param::string())
            .property("field2", Param::number())
            .required(["field1"])
    })
    .build();
```

### Parameter Types

Orpheus supports all standard JSON schema types through the `Param` enum:

#### String Parameters

```rust
// Basic string
Param::string()

// String with description
Param::string().description("User's full name")

// String with enumerated values
Param::string()
    .description("Temperature unit")
    .enums(["celsius", "fahrenheit"])
```

#### Number Parameters

```rust
// Basic number (floating point)
Param::number()

// Number with description
Param::number().description("Temperature in degrees")

// Integer (whole numbers only)
Param::integer().description("Age in years")
```

#### Boolean Parameters

```rust
Param::boolean().description("Whether the user is active")
```

#### Object Parameters

```rust
Param::object()
    .description("User location")
    .property("city", Param::string())
    .property("country", Param::string())
    .property("coordinates", 
        Param::object()
            .property("lat", Param::number())
            .property("lng", Param::number())
            .required(["lat", "lng"])
    )
    .required(["city", "country"])
    .additional_properties(false)  // Prevent extra properties
```

#### Array Parameters

```rust
// Array of strings
Param::array()
    .description("List of hobbies")
    .items(Param::string())

// Array of objects
Param::array()
    .description("List of users")
    .items(
        Param::object()
            .property("name", Param::string())
            .property("email", Param::string())
            .required(["name", "email"])
    )
```

#### Null Parameters

```rust
Param::null()  // Represents a null value
```

## Advanced Features

### Union Types (anyOf)

You can specify that a field can be one of several types using the `anyof!` macro:

```rust
use orpheus::anyof;

let schema = Param::object()
    .property("value", anyof![
        Param::string(),
        Param::number(),
        Param::null()
    ])
    .required(["value"])
    .end();
```
