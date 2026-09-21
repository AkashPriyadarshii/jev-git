# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]
### Fixed
- Fail-closed gates: missing key, API error, and decode error refuse the commit (exit 2) instead of skipping.
- Threshold bands: block at 0.80+, warn at 0.55-0.80, pass below. Pinned model jev-1.13.0.
- Char-boundary diff truncation with tail warning. 8s request timeout.
- Hook installer chains existing hooks, sets executable bit, adds pre-push.

## [0.0.1] - 2026-09-18
### Added
- Initial implementation of `git-jev`.
- `git jev install`: One-command hook installer for `.git/hooks/pre-commit`.
- `git jev check`: Sub-second semantic diff evaluator powered by TypeSafe AI Jev.
