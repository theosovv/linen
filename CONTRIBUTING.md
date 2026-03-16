# Contributing to Linen

Thank you for your interest in contributing to Linen! This document provides guidelines for contributing to the project.

## Code of Conduct

Be respectful and constructive. We welcome contributors of all backgrounds and experience levels.

## Development Setup

1. **Install Rust** (>= 1.75):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Clone the repository**:
   ```bash
   git clone https://github.com/linen-lang/linen.git
   cd linen
   ```

3. **Install pre-commit hooks** (optional but recommended):
   ```bash
   pip install pre-commit
   pre-commit install
   ```

## Project Structure

```
linen/
├── crates/
│   ├── compiler/    # Lexer, parser, type checker
│   ├── vm/          # Virtual machine
│   ├── stdlib/      # Standard library
│   └── cli/         # Command-line interface
├── docs/            # Documentation
└── examples/        # Example programs
```

## Workflow

1. **Create a branch**: `git checkout -b feature/your-feature-name`
2. **Make changes**: Follow the code style guidelines
3. **Test**: Run `cargo test --workspace`
4. **Format**: Run `cargo fmt --all`
5. **Lint**: Run `cargo clippy --all-targets --all-features`
6. **Commit**: Use clear, descriptive commit messages
7. **Push**: `git push origin feature/your-feature-name`
8. **Open a Pull Request**

## Code Style

- Follow standard Rust conventions (enforced by `rustfmt`)
- Write documentation for all public items
- Add tests for new functionality
- Keep functions small and focused
- Use meaningful variable names

## Testing

```bash
# Run all tests
cargo test --workspace

# Run with coverage
cargo tarpaulin --workspace

# Run specific crate tests
cargo test -p linen-compiler
```

## Documentation

- Use `cargo doc --open` to build and view documentation
- Document all public APIs with doc comments
- Include examples in documentation where appropriate

## Commit Messages

Use conventional commits format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style (formatting, no logic change)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Example:
```
feat(parser): add support for pattern matching

Implement pattern matching with exhaustiveness checking.
```

## Questions?

- Open an issue for bugs or feature requests
- Start a discussion for questions or ideas

## License

By contributing, you agree that your contributions will be licensed under the MIT/Apache-2.0 dual license.
