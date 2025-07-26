# Contributing to Orpheus 🎸

Thank you for your interest in contributing to Orpheus! We're excited to have you join our community of developers working to make AI applications more ergonomic and accessible.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Pull Request Process](#pull-request-process)
- [Development Guidelines](#development-guidelines)
- [Testing](#testing)
- [Documentation](#documentation)
- [Release Process](#release-process)

## Code of Conduct

We are committed to providing a friendly, safe, and welcoming environment for all contributors. Please be respectful and considerate in your interactions with other community members.

### Our Standards

- Be welcoming and inclusive
- Be respectful of differing viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

### Prerequisites

- **Rust**: Latest stable version (install via [rustup](https://rustup.rs/))
- **Python**: 3.9+ (for Python bindings)
- **Git**: For version control
- **OpenRouter API Key**: Get one at [OpenRouter](https://openrouter.ai/)

### Optional Tools

- **mise**: For managing tool versions (see `mise.toml`)
- **cargo-watch**: For auto-recompiling during development
- **cargo-expand**: For debugging macros
- **maturin**: For building Python bindings

## Development Setup

1. **Fork and Clone the Repository**

   ```bash
   git clone https://github.com/YOUR_USERNAME/orpheus.git
   cd orpheus
   ```

2. **Set up Environment Variables**

   ```bash
   export ORPHEUS_API_KEY="your-openrouter-api-key"
   ```

3. **Install Rust Dependencies**

   ```bash
   cargo build
   ```

4. **Install Python Development Dependencies** (optional, for Python bindings)

   ```bash
   pip install maturin
   cd py
   pip install -e .
   ```

5. **Run Tests**

   ```bash
   cargo test
   cargo test --all-features
   ```

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. When creating a bug report, include:

- A clear and descriptive title
- Steps to reproduce the issue
- Expected behavior vs actual behavior
- Your environment (OS, Rust version, Python version if applicable)
- Relevant code snippets or error messages
- OpenRouter model being used (if applicable)

### Suggesting Enhancements

Enhancement suggestions are welcome! Please provide:

- A clear and descriptive title
- Detailed description of the proposed feature
- Use cases and examples
- Why this enhancement would be useful to most users
- Possible implementation approach (if you have ideas)

### Contributing Code

1. **Find an Issue**: Look for issues labeled `good first issue` or `help wanted`
2. **Comment on the Issue**: Let us know you're working on it
3. **Create a Branch**: Use a descriptive branch name (e.g., `feature/add-retry-logic`, `fix/streaming-timeout`)
4. **Write Code**: Follow our coding standards (see below)
5. **Write Tests**: Ensure your code is tested
6. **Update Documentation**: Include doc comments and update README if needed
7. **Submit a Pull Request**: Follow the PR template

## Pull Request Process

1. **Ensure CI Passes**: All tests and checks must pass
2. **Update Documentation**: Include relevant documentation updates
3. **Write Clear Commit Messages**: Follow conventional commits format
4. **Request Review**: Tag maintainers for review
5. **Address Feedback**: Respond to review comments promptly

### PR Title Format

Use conventional commit format:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `test:` Test additions or modifications
- `refactor:` Code refactoring
- `perf:` Performance improvements
- `chore:` Maintenance tasks

Example: `feat: add support for function calling with streaming`

## Development Guidelines

### Rust Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Use clippy for linting (`cargo clippy`)
- Write idiomatic Rust code
- Prefer explicit error handling over panics
- Use the builder pattern consistently (see existing code)

### Key Design Principles

1. **Ergonomics First**: APIs should be intuitive and easy to use
2. **Type Safety**: Leverage Rust's type system for compile-time guarantees
3. **Performance**: Optimize for speed without sacrificing usability
4. **Flexibility**: Support both sync and async patterns
5. **Compatibility**: Maintain backward compatibility when possible

### Error Handling

- Use `thiserror` for custom error types
- Provide helpful error messages
- Support optional `anyhow` integration via feature flag
- Never panic in library code (except for programmer errors)

### Feature Flags

When adding new optional dependencies, use feature flags:

```toml
[features]
your-feature = ["dep:your-dependency"]
```

Update the README to document the new feature flag.

## Testing

### Unit Tests

- Write unit tests for all new functionality
- Place tests in a `tests` module within the same file
- Use descriptive test names

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_behavior() {
        // Your test here
    }
}
```

### Integration Tests

- Add integration tests for features that interact with OpenRouter
- Use the free tier models for testing (e.g., `deepseek/deepseek-r1-0528-qwen3-8b:free`)
- Mock external dependencies when possible

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with all features
cargo test --all-features

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

## Documentation

### Code Documentation

- Add doc comments to all public APIs
- Include examples in doc comments
- Use `///` for item documentation
- Use `//!` for module documentation

```rust
/// Sends a chat completion request to OpenRouter.
///
/// # Example
/// ```
/// use orpheus::prelude::*;
///
/// let client = Orpheus::new("api-key");
/// let response = client
///     .chat("Hello!")
///     .model("openai/gpt-4o")
///     .send()?;
/// ```
pub fn chat(&self, prompt: impl Into<ChatInput>) -> ChatRequest {
    // Implementation
}
```

### Examples

- Add runnable examples to the `examples/` directory
- Keep examples simple and focused
- Include comments explaining what the example demonstrates

### README Updates

Update the README when:
- Adding new features
- Changing public APIs
- Adding new dependencies or feature flags

## Release Process

Releases are managed by maintainers. The process includes:

1. Update version in `Cargo.toml` and `py/pyproject.toml`
2. Update CHANGELOG (if exists)
3. Create a git tag: `git tag v0.x.x`
4. Push tag: `git push origin v0.x.x`
5. GitHub Actions will handle publishing to crates.io and PyPI

## Getting Help

- **Discord**: Join our community (link TBD)
- **GitHub Issues**: For bug reports and feature requests
- **GitHub Discussions**: For questions and general discussion

## Recognition

Contributors will be recognized in:
- The project's contributor list
- Release notes for significant contributions
- Special mentions for exceptional contributions

## License

By contributing to Orpheus, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Orpheus! Your efforts help make AI development more accessible and enjoyable for everyone. 🎸