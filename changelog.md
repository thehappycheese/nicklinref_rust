# Changelog

## [Unreleased] 2023-06-12

- drop support for case insensitive query

## [0.10.2] 2023-06-11

- Refactoring to make main.rs short and easy to understand and decouple
  `Settings` from the rest of the codebase
- Dropped support for `--config config.json`
- Fixed #3 unnecessary errors if folder for data file doesn't exist
- Added CLI settings (Fixed #4 )
  - See new `--help` option for details
  - added `--force-update-data` flag to force data refresh on startup
- `/.vscode` is now included in the repo to store project specific settings

## [0.10.1] 2023-06-01

- create changelog
- enable gzip compression
- implement `x-request-id` echo header
  - note: still does not work on reject :/
