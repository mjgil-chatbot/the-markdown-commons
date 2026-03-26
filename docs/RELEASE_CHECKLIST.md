# Release checklist

This checklist ensures a complete and safe release of `the-markdown-commons`.

## Pre-release verification

### Code quality

```bash
# Format check
cargo fmt --check

# Lint check
cargo clippy --release --all-features -- \
  -A unused_variables \
  -A clippy::needless_range_loop \
  -A clippy::redundant_closure \
  -A clippy::manual_clamp \
  -D warnings

# Build verification
cargo build --release --all-features
```

### Tests

```bash
# Unit tests (default)
cargo test --release

# All feature-gated tests
cargo test --release --all-features --lib --tests --bins -- --test-threads=1

# Public API tests
cargo test --release --test public_api

# Doc tests
cargo test --release --doc
```

### Documentation

```bash
# Generate rustdoc
cargo doc --release --no-deps --all-features
```

### Package verification

```bash
# List package contents (verify no missing files)
cargo package --allow-dirty --list

# Dry run publish
cargo publish --dry-run --allow-dirty
```

### Benchmark compile

```bash
cargo bench --release --bench parse --no-run
```

## Release steps

1. **Update version** in `Cargo.toml` following semver:
   - `patch` for bug fixes
   - `minor` for new backwards-compatible features
   - `major` for breaking changes

2. **Update CHANGELOG.md** with all changes since last release. Categorize as:
   - Added
   - Changed
   - Fixed
   - Security

3. **Commit the release** with a message like `Release v0.1.x`

4. **Tag the release**:
   ```bash
   git tag v0.1.x
   git push --tags
   ```

5. **Publish** (if publishing to crates.io):
   ```bash
   cargo publish
   ```

## Post-release

- Verify CI passes on the tag commit
- Monitor for any reports of regression or issues
- Update the changelog entry date after publishing
- If you run `auto-ui-tests`, confirm whether they executed or skipped because the X11 tools were unavailable

## Changelog categories

See `docs/CHANGELOG_POLICY.md` for changelog formatting rules.
