# Changelog

The format of this changelog is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

## Unreleased

## [0.3.3] - 2024-12-14
- Added "list" alias for "db show" subcommand
- More Logging
    - Improved Error Messages
- Updated dependencies
    - scraper to 0.22.0
- Added thiserror as dependency to refactor Error types
- "Responsorial" will now be read as Psalm
- Added "db store" command

[0.3.3]: https://github.com/Squidroot2/lectio-diei/compare/v0.3.2...v0.3.3

## [0.3.2] - 2024-12-09
- Code lint
- Updated dependencies
    - scraper to 0.21.0
    - sqlx to 0.8.2
- Fix for Christmas days

[0.3.2]: https://github.com/Squidroot2/lectio-diei/compare/v0.3.1...v0.3.2

## [0.3.1] - 2024-08-10
- Added alias "reading" for "readings" arg
- Updated dependencies
    - scraper to 0.20.0

[0.3.1]: https://github.com/Squidroot2/lectio-diei/compare/v0.3.0...v0.3.1

## [0.3.0] - 2024-07-28
- Added Alleluia

[0.3.0]: https://github.com/Squidroot2/lectio-diei/compare/v0.2.2...v0.3.0

## [0.2.2] - 2024-07-24
- updated dependencies
    - sqlx to 0.8.0

[0.2.2]: https://github.com/Squidroot2/lectio-diei/compare/v0.2.1...v0.2.2

## [0.2.1] - 2024-07-20
- Added 'config show' command
- Fixed readings for holidays
- Handle case where reading has no location

[0.2.1]: https://github.com/Squidroot2/lectio-diei/compare/v0.2.0...v0.2.1

## [0.2.0] - 2024-07-17
- Added formatting options (config and args) to preserve original new lines OR use custom width
- Fixed verse not always being removed
- Added 'config upgrade' command

[0.2.0]: https://github.com/Squidroot2/lectio-diei/compare/v0.1.2...v0.2.0

## [0.1.2] - 2024-07-14
- Updated dependencies (7/13/24)
- Removed once_lock as direct dependency (still a transitive dependency)
- Changed formatting of Psalm: removed verse

[0.1.2]: https://github.com/Squidroot2/lectio-diei/compare/v0.1.1...v0.1.2

## [0.1.1] - 2024-07-13
- XDG environment variables (XDG_CONFIG_HOME, XDG_DATA_HOME, XDG_STATE_HOME) are now respected
- Fixed name of database file

[0.1.1]: https://github.com/Squidroot2/lectio-diei/compare/v0.1.0...v0.1.1

## [0.1.0] - 2024-07-12
- Initial Release

[0.1.0]: https://github.com/Squidroot2/lectio-diei/commits/v0.1.0
