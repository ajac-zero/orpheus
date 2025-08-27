---
icon: box-open
---

# Structured Output

Structured output allows you to request JSON responses that follow a specific schema from language models. This ensures that the model's response conforms to your expected data structure, making it easier to parse and use in your applications.

## Overview

Orpheus provides a powerful and ergonomic API for structured output through the `Format` type. You can define JSON schemas that specify exactly what structure you want the model to return, including property types, descriptions, and requirements.

## Basic Usage

To use structured output, create a `Format` using the builder pattern and attach it to your chat request:

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

## Schema Definition

### Format Builder

The `Format::json()` method starts building a JSON schema format:

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

### Complex Example

Here's a comprehensive example showing various schema features:

```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let weather_format = Format::json("weather_report")
        .with_schema(|schema| {
            schema
                .property("location", 
                    Param::object()
                        .description("Location information")
                        .property("city", Param::string().description("City name"))
                        .property("country", Param::string().description("Country name"))
                        .property("coordinates",
                            Param::object()
                                .property("latitude", Param::number())
                                .property("longitude", Param::number())
                                .required(["latitude", "longitude"])
                        )
                        .required(["city", "country"])
                )
                .property("current", 
                    Param::object()
                        .description("Current weather conditions")
                        .property("temperature", Param::number().description("Temperature in Celsius"))
                        .property("humidity", Param::integer().description("Humidity percentage"))
                        .property("conditions", Param::string().description("Weather conditions"))
                        .property("wind_speed", Param::number().description("Wind speed in km/h"))
                        .required(["temperature", "conditions"])
                )
                .property("forecast",
                    Param::array()
                        .description("3-day forecast")
                        .items(
                            Param::object()
                                .property("date", Param::string())
                                .property("high", Param::number())
                                .property("low", Param::number())
                                .property("conditions", Param::string())
                                .required(["date", "high", "low", "conditions"])
                        )
                )
                .required(["location", "current", "forecast"])
        })
        .build();

    let response = client
        .chat("What's the weather like in Tokyo, Japan? Include a 3-day forecast.")
        .model("openai/gpt-4o")
        .response_format(weather_format)
        .send()?;

    println!("{}", response.content()?);

    Ok(())
}
```

## Data Extraction Use Case

Structured output is particularly useful for data extraction tasks:

```rust
use serde::Deserialize;
use orpheus::prelude::*;

#[derive(Deserialize, Debug)]
struct ExtractedData {
    name: String,
    age: u32,
    occupation: Option<String>,
    skills: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let extraction_format = Format::json("extracted_person")
        .with_schema(|schema| {
            schema
                .property("name", Param::string().description("Person's full name"))
                .property("age", Param::integer().description("Person's age"))
                .property("occupation", Param::string().description("Person's job title"))
                .property("skills", 
                    Param::array()
                        .description("List of skills")
                        .items(Param::string())
                )
                .required(["name", "age", "skills"])
        })
        .build();

    let text = "John Smith is a 35-year-old software engineer who specializes in Rust, Python, and machine learning.";
    
    let response = client
        .chat(&[
            Message::system("Extract structured information from the given text."),
            Message::user(text)
        ])
        .model("openai/gpt-4o")
        .response_format(extraction_format)
        .send()?;

    // Parse the JSON response directly into your struct
    let extracted: ExtractedData = serde_json::from_str(&response.content()?)?;
    println!("{:#?}", extracted);

    Ok(())
}
```

## Error Handling

When working with structured output, be prepared to handle potential JSON parsing errors:

```rust
use serde::Deserialize;
use orpheus::prelude::*;

#[derive(Deserialize)]
struct ParsedResponse {
    result: String,
    confidence: f64,
}

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;
    
    let format = Format::json("analysis")
        .with_schema(|schema| {
            schema
                .property("result", Param::string())
                .property("confidence", Param::number())
                .required(["result", "confidence"])
        })
        .build();

    let response = client
        .chat("Analyze this text: 'The weather is nice today'")
        .model("openai/gpt-4o")
        .response_format(format)
        .send()?;

    match serde_json::from_str::<ParsedResponse>(&response.content()?) {
        Ok(parsed) => {
            println!("Result: {}", parsed.result);
            println!("Confidence: {}", parsed.confidence);
        }
        Err(e) => {
            eprintln!("Failed to parse response: {}", e);
            eprintln!("Raw response: {}", response.content()?);
        }
    }

    Ok(())
}
```

## Best Practices

### 1. Use Descriptive Names and Descriptions

Always provide clear descriptions for your schema and properties:

```rust
let format = Format::json("user_profile")
    .with_schema(|schema| {
        schema
            .property("username", 
                Param::string().description("Unique username, lowercase, no spaces")
            )
            .property("email", 
                Param::string().description("Valid email address")
            )
            .property("age", 
                Param::integer().description("Age in years, must be positive")
            )
            .required(["username", "email"])
    })
    .build();
```

### 2. Use Appropriate Data Types

Choose the most specific data type for each field:

```rust
// Good: Use integer for whole numbers
.property("count", Param::integer())

// Good: Use number for decimals
.property("price", Param::number())

// Good: Use enums for limited choices
.property("status", Param::string().enums(["active", "inactive", "pending"]))
```

### 3. Handle Optional Fields Properly

Use the `required()` method to specify which fields are mandatory:

```rust
let schema = Param::object()
    .property("id", Param::string())           // Required
    .property("name", Param::string())         // Required  
    .property("nickname", Param::string())     // Optional
    .property("age", Param::integer())         // Optional
    .required(["id", "name"])                  // Only id and name are required
    .end();
```

### 4. Validate Against Your Schema

Always test your schemas with sample data to ensure they work as expected:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_serialization() {
        let format = Format::json("test_schema")
            .with_schema(|schema| {
                schema
                    .property("name", Param::string())
                    .property("count", Param::integer())
                    .required(["name"])
            })
            .build();
            
        // Ensure the schema serializes correctly
        let json = serde_json::to_value(format).unwrap();
        assert!(json["json_schema"]["schema"]["properties"]["name"].is_object());
    }
}
```

## Model Compatibility

Structured output works best with models that have strong JSON generation capabilities:

- **Recommended**: OpenAI GPT-4o, Claude 3.5 Sonnet, Gemini Pro
- **Good**: Most recent instruction-tuned models
- **Limited**: Older or smaller models may struggle with complex schemas

Always test with your target model to ensure consistent results.

## Troubleshooting

### Schema Not Being Followed

If the model isn't following your schema:

1. **Simplify the schema** - Complex nested structures may be harder to follow
2. **Add more descriptive prompts** - Include instructions about the expected format
3. **Use stricter models** - Some models handle structured output better than others
4. **Enable strict mode** - Use `.strict(true)` in your format builder

### JSON Parsing Errors

If you're getting JSON parsing errors:

1. **Check the raw response** - Log `response.content()?` to see what the model actually returned
2. **Handle malformed JSON** - Some models may include extra text around the JSON
3. **Use more robust parsing** - Consider using `serde_json::from_str()` with error handling

### Performance Considerations

For better performance with structured output:

1. **Keep schemas simple** - Avoid deeply nested or overly complex structures
2. **Use caching** - Cache format objects when making repeated requests
3. **Batch requests** - Process multiple items in a single structured response when possible

## See Also

- [Tool Calling](tool-calling/) - For function calling with structured parameters
- [Multimodality](multimodality.md) - For structured output with image inputs
- [Async Support](async-support.md) - For async structured output requests