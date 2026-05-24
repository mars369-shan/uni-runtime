# Contributing to Uni-Runtime

Welcome to Uni-Runtime! We appreciate your contributions. Here's a guide to help you get started.

## Development Environment

### Prerequisites

- Rust 1.70+
- Docker (optional, for creating real distribution environments)
- proot (optional, for lightweight environments)

### Building the Project

```bash
# Clone the repository
git clone https://github.com/mars369-shan/uni-runtime.git
cd uni-runtime

# Build
cargo build --release

# Run tests
cargo test

# Run the program
./target/release/uni-runtime --help
```

## Code Guidelines

### Rust Code Style

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow Rust official style guidelines

### Commit Convention

Please use the following format for commits:

```
<type>: <description>

<detailed explanation>
```

Types include:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation update
- `refactor`: Code refactoring
- `test`: Test update
- `chore`: Other changes

## Branch Management

- `main`: Main branch, stable releases
- `develop`: Development branch
- `feature/*`: Feature branches
- `fix/*`: Bug fix branches

## Submitting a PR

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to your fork
5. Create a Pull Request

## License

All contributed code will be licensed under the MIT License.

## Contact

If you have questions, please create an Issue or send an email.
