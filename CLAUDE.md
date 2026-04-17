# homebrew-tap

Homebrew tap repository for distributing pre-built CLI binaries.

## Structure

```
homebrew-tap/
├── Formula/         ← Homebrew formulae (one .rb per tool)
└── README.md
```

## Adding a new formula

Each formula downloads a pre-built binary from the source project's GitHub Releases.
No source code lives in this repo — only Formula definitions.

## Updating a formula

Update `version` and `sha256` values in the `.rb` file when a new release is published.
