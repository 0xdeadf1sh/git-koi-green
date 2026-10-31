# Changelog

Notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0] - 2026-10-31

### Changed

- The koi is replaced by a side-view green fish drawn from a fixed sprite. It
  grows with contributions, laps the frame seen from the side, and faces the
  way it swims.

### Removed

- Bodies, language colors, and streak marks: the `coat` and `markings` modules,
  the `koi_body` argument of `render_koi` and `render_swim_frame`, the
  `--body` option, and the Action's `body` input.

## [1.0.1] - 2026-10-28

### Changed

- The Action is listed as `git-profile-koi`, since a GitHub user already holds
  the name `git-koi`, and its description fits the Marketplace limit.

## [1.0.0] - 2026-10-28

### Added

- `git-koi collect`, which gathers an account's public snapshot through the
  GitHub CLI, and `--help` and `--version` on the command.
- `git-koi render`, which draws the day's koi as a looping GIF, a still PNG, and
  a preview page, in six bodies.
- A GitHub Action that downloads the release binary and runs both inside a
  profile repository's workflow.
- The `git-koi` library: account history, koi development, behavior, motion,
  markings, rendering, and encoding.

[Unreleased]: https://github.com/0xdeadf1sh/git-koi-green/compare/v2.0.0...HEAD
[2.0.0]: https://github.com/0xdeadf1sh/git-koi-green/releases/tag/v2.0.0
[1.0.1]: https://github.com/yalongwastaken/git-koi/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/yalongwastaken/git-koi/releases/tag/v1.0.0
