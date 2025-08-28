# Documentation Guidelines for Code Agents

This document provides guidelines for AI agents writing documentation for the Orpheus project. These guidelines are based on observed patterns and user preferences from the codebase.

## Core Principles

### 1. Be Factual, Not Prescriptive
- State what options are available and how to use them
- Avoid recommendations, best practices, or "should/shouldn't" language
- Let users make their own decisions with the information provided
- Focus on "what" and "how" rather than "why" or "when"

### 2. Keep It Short and Sweet
- Prioritize conciseness over comprehensiveness
- Remove verbose explanations and redundant examples
- Eliminate lengthy use case scenarios unless essential
- Aim for practical, focused content

### 3. Show, Don't Tell
- Provide working code examples for every feature
- Use realistic but simple examples that users can adapt
- Include imports and complete function signatures
- Demonstrate the builder patterns and API usage

## Structure Guidelines

### Required Sections
1. **Brief Introduction** - One sentence explaining what the feature does
2. **Basic Usage** - Simple example showing the feature in action
3. **Configuration Options** - Available parameters with types and descriptions
4. **Integration Examples** - How it works with other features (if applicable)

### Optional Sections
- **Multiple Usage Patterns** - If there are significantly different ways to use the feature
- **Error Handling** - If there are specific validation or error patterns
- **Type Information** - Tables showing parameters, types, and defaults

### Avoid These Sections
- "Best Practices" or "Recommendations"
- "Troubleshooting" or "Common Issues"
- "Performance Considerations"
- "Future Enhancements" or roadmap items
- Extensive "Use Cases" or scenario descriptions

## Code Example Standards

### Format
```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    // Brief comment explaining what this does
    let response = client
        .chat("Example query")
        .model("openai/gpt-4o")
        .feature_method()  // The feature being demonstrated
        .send()?;

    println!("{}", response.content()?);
    Ok(())
}
```

### Guidelines
- Always include necessary imports (`use orpheus::prelude::*;`)
- Use `anyhow::Result<()>` for error handling in examples
- Show both builder pattern usage and direct construction where applicable
- Keep examples focused on the feature being documented
- Use realistic model names from the OpenRouter ecosystem
- Include error handling (`?` operator) in examples

## API Documentation Patterns

### Builder APIs
When documenting builder patterns:
```rust
// Basic usage with defaults
Feature::builder().build()

// With configuration
Feature::builder()
    .parameter(value)
    .another_parameter("string value")
    .build()
```

### Configuration Tables
Use markdown tables for parameters:

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `param_name` | `Option<String>` | Brief description | Default value or "None" |

### Type Annotations
- Always show the exact Rust types (`Option<i32>`, `Vec<String>`, etc.)
- Use code formatting for types, parameters, and values
- Show both optional and required parameters clearly

## Language and Tone

### Do Use
- Present tense ("enables", "processes", "returns")
- Active voice ("The method returns..." not "A result is returned...")
- Specific technical terms from the codebase
- Neutral, factual language

### Don't Use
- Subjective language ("better", "easier", "recommended")
- Modal verbs suggesting obligation ("should", "must", "ought to")
- Marketing language ("powerful", "seamless", "comprehensive")
- Comparative language unless stating factual differences

## Integration Documentation

### Cross-Feature References
When a feature integrates with others:
- Show minimal working examples of the integration
- Reference other features by their exact documentation section names
- Use relative links: `[Tool Calling](tool-calling/)`
