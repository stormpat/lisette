# Changelog

## [0.1.2](https://github.com/ivov/lisette/compare/lisette-v0.1.1...lisette-v0.1.2) - 2026-03-31

### Added

- add quickstart link to CLI help and redirect page
- show nil diagnostic for null, Nil, and undefined

### Fixed

- improve doc help text colors, examples, and description
- fold Range sub-expressions in AstFolder
- prevent OOM by lowering max parser errors to 50
- prevent subtraction overflow in span calculation
- lower parser max depth to 64 to prevent stack overflow
- lower parser max depth to prevent stack overflow under asan
- remove unnecessary borrow in nil diagnostic format

### Other

- improve CLI help consistency and hide internal commands

## [0.1.1](https://github.com/ivov/lisette/compare/lisette-v0.1.0...lisette-v0.1.1) - 2026-03-21

### Fixed

- ensure complete go.sum before running go build
- resolve prelude path for crates.io packaging

## [0.1.0](https://github.com/ivov/lisette/releases/tag/lisette-v0.1.0) - 2026-03-21

### Added

- initial release v0.1.0
