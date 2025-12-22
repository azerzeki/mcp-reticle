# Contributing to Reticle

Thank you for your interest in contributing to Reticle! This document provides guidelines and instructions for contributing.

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/reticle.git
   cd reticle
   ```
3. Install dependencies:
   ```bash
   just setup
   ```
4. Start the development server:
   ```bash
   just dev
   ```

## Development Workflow

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation changes
- `refactor/description` - Code refactoring

### Code Style

**Rust:**
- Run `just fmt` before committing
- Run `just lint` to check for warnings
- Follow standard Rust conventions

**TypeScript/React:**
- Use functional components with hooks
- Follow existing patterns in the codebase

### Testing

```bash
just test-direct    # Test MCP server directly
just test-proxy     # Test with proxy
just check          # Check Rust compilation
```

## Submitting Changes

### Pull Request Process

1. Create a feature branch from `main`
2. Make your changes
3. Ensure all tests pass
4. Update documentation if needed
5. Submit a pull request

### PR Guidelines

- Keep PRs focused on a single change
- Write clear commit messages
- Include screenshots for UI changes
- Reference related issues

### Commit Messages

Use clear, descriptive commit messages:

```
Add session export to CSV format

- Implement CSV serialization for recorded sessions
- Add export button to session panel
- Update documentation
```

## Areas for Contribution

We welcome contributions in these areas:

- **Security firewall** - Method blocking/allowing policies
- **Traffic replay** - Request modification and replay
- **Export formats** - CSV and HAR export support
- **Token analytics** - Context usage profiling
- **Documentation** - Guides, examples, tutorials
- **Testing** - Additional test coverage

## Reporting Bugs

Use the GitHub issue tracker with:

- Clear description of the issue
- Steps to reproduce
- Expected vs actual behavior
- System information (OS, Rust version, Node version)
- Relevant logs or screenshots

## Feature Requests

Open an issue with:

- Clear description of the feature
- Use case and motivation
- Proposed implementation (if any)

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Questions?

Open a discussion on GitHub or file an issue.

---

Thank you for contributing to Reticle!
