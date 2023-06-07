# Changelog

## [0.10.2] 2023-06-05 - Unreleased

- Refactoring to make main.rs short and easy to understand
- Settings revised. See --help
- adopted clap
- add --force-update-data command line argument to force data refresh on startup
- fixed unnecessary errors if folder for data file doesn't exist
- drop support for `--config` which allowed settings.json to be loaded from json
  - TODO: still have a lot of `serde` annotations to make this work which could
          be removed now maybe.

## [0.10.1] 2023-06-01

- create changelog
- enable gzip compression
- implement `x-request-id` echo header
  - note: still does not work on reject :/
