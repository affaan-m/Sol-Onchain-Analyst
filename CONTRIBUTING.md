# Contributing to Cainam Core

Thank you for considering contributing to Cainam Core! Here are some guidelines to help you get started.

## Issues

Before reporting an issue, please check existing or similar issues that are currently tracked.

## Pull Requests

Contributions are always encouraged and welcome. Before creating a pull request, create a new issue that tracks that pull request describing the problem in more detail. Pull request descriptions should include information about it's implementation, especially if it makes changes to existing abstractions.

PRs should be small and focused and should avoid interacting with multiple facets of the library. This may result in a larger PR being split into two or more smaller PRs. Commit messages should follow the [Conventional Commit](conventionalcommits.org/en/v1.0.0) format (prefixing with `feat`, `fix`, etc.) as this integrates into our auto-releases via a [release-plz](https://github.com/MarcoIeni/release-plz) Github action.

**Working on your first Pull Request?** You can learn how from this *free* series [How to Contribute to an Open Source Project on GitHub](https://kcd.im/pull-request)

## Project Structure

TBD

## Developing

### Setup

```bash
git clone https://github.com/cainamventures/cainam-core
cd cainam-core
cargo test
```

### Clippy and Fmt

We enforce both `clippy` and `fmt` for all pull requests.

```bash
cargo clippy -- -D warnings
```

```bash
cargo fmt
```

### Tests

Make sure to test against the test suite before making a pull request.

```bash
cargo test
```
