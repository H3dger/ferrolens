# Contributing to ferrolens

Thanks for your interest in improving `ferrolens`.

## Before you start

- Search existing issues before opening a new one
- Keep changes focused and easy to review
- If the change affects user-facing behavior, update the README or release notes as needed

## Development workflow

1. Fork the repository
2. Create a focused branch for your change
3. Make the smallest coherent change that solves one problem
4. Run the local verification steps before opening a pull request

## Local verification

```bash
cargo test -q
cargo fmt --check
cargo run -- --help
```

If your change affects interactive behavior, also run the tool manually on a real TSV/CSV/VCF file.

## Pull requests

Please include:

- a short explanation of the problem
- the approach you took
- any user-facing behavior changes
- screenshots or terminal recordings when UI behavior changes materially

## Scope expectations

`ferrolens` is intentionally focused. Please avoid bundling unrelated refactors or feature expansions into one pull request.
