# Changelog policy

This document defines how changes are recorded in the `CHANGELOG.md` file.

## Format

Entries follow this structure:

```markdown
## [version] — YYYY-MM-DD

### Added
- New feature description

### Changed
- Existing behavior change

### Fixed
- Bug fix description

### Security
- Security-related change or note
```

## Categories

Use the most appropriate category. Each change gets exactly one category.

### Added
New public API, new parser feature, new renderer output, new documentation.

### Changed
Behavior changes to existing features, including performance improvements.

### Fixed
Bug fixes for incorrect behavior.

### Security
Changes that affect security posture: new mitigations, breaking defaults, documentation updates for secure usage.

## Entry guidelines

1. **Be concise** — one line per change, max 72 characters
2. **Be specific** — mention the affected component if not obvious
3. **Use imperative mood** — "Add feature" not "Added feature"
4. **Reference issues** — if there's a relevant issue or PR, link to it

## Example entries

```markdown
## [0.1.5] — 2024-04-15

### Added
- `HtmlOptions::allow_raw_html` documentation and example

### Fixed
- Balanced parentheses in bare autolinks not stripped (#42)
```

## When to update

- On every release (merge to main with version bump)
- When making security-relevant changes (even if no version bump)

## What to include

- All user-facing changes: API, parser behavior, renderer output
- All security-relevant changes
- All significant bug fixes

## What to omit

- Internal refactoring with no behavior change
- Documentation-only fixes (those don't need changelog entries)
- Test additions without behavior change