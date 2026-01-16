# Standardy kodowania

## Rust Standards

### Formatting
- `rustfmt` for consistent formatting
- Standard Rust style guide

### Linting
- `clippy` for code quality
- All warnings must be addressed

### Error Handling
- `thiserror` for custom error types
- `anyhow` for internal errors
- Comprehensive error messages

### Safety
- Zero unsafe code (except FFI bindings)
- Memory safety guaranteed by compiler

### Testing
- Unit tests for all public APIs
- Integration tests for workflows
- Test coverage >90%

### Documentation
- `rustdoc` for API documentation
- Examples in doc comments
- All public items documented

## TypeScript Standards

### Configuration
- Strict TypeScript configuration
- No `any` types (except FFI boundaries)
- Explicit return types

### Linting
- ESLint with TypeScript rules
- Prettier for formatting
- Airbnb style guide base

### Error Handling
- Custom Error classes
- Async/await with proper error propagation
- User-friendly error messages

### Testing
- Jest for unit and integration tests
- Test coverage >90%
- Mock adapters for testing

### Documentation
- TSDoc comments for public APIs
- Type definitions exported
- Usage examples

## Python Standards (Future)

### Type Hints
- Full type annotations
- `mypy` for type checking
- Generic types where appropriate

### Code Quality
- `black` for formatting
- `flake8` for linting
- `isort` for import sorting

### Error Handling
- Custom exception classes
- Proper exception chaining
- Resource cleanup with context managers

### Testing
- `pytest` framework
- Test coverage >90%
- Fixtures for common test data

### Documentation
- Google-style docstrings
- Sphinx for documentation generation
- Type information in docs

## Cross-Language Standards

### API Consistency
- Same method names across languages
- Consistent parameter order
- Similar error handling patterns

### Documentation
- Consistent terminology
- Cross-references between languages
- Language-specific examples

### Versioning
- Semantic versioning
- Breaking changes coordinated
- Deprecation warnings

### Performance
- Benchmarks for all languages
- Memory usage monitoring
- Performance regression detection

## Code Review Standards

### Pull Request Requirements
- Tests pass
- Linting passes
- Documentation updated
- Performance benchmarks pass
- Cross-platform testing completed

### Review Checklist
- Code correctness
- Performance implications
- Security considerations
- API consistency
- Documentation quality
- Test coverage