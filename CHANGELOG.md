# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## [0.4.0](https://github.com/justinrubek/calendar-scheduler/compare/0.3.0..0.4.0) - 2023-01-07
#### Documentation
- comment clap commands - ([0860ae3](https://github.com/justinrubek/calendar-scheduler/commit/0860ae3e47b678dbc65e751b56d454531067899b)) - [@justinrubek](https://github.com/justinrubek)
#### Features
- implemented UTC timezone for calendars - ([a2b3a43](https://github.com/justinrubek/calendar-scheduler/commit/a2b3a43c488a999b69aa27bad6a022a97b4a7234)) - [@justinrubek](https://github.com/justinrubek)
- start api command - ([3e85263](https://github.com/justinrubek/calendar-scheduler/commit/3e85263cd98608cd6018fec8859ebb244ccb3ba2)) - [@justinrubek](https://github.com/justinrubek)
- list events command - ([9f8377f](https://github.com/justinrubek/calendar-scheduler/commit/9f8377f80a55926bb77cf19e574d8cce1741fc18)) - [@justinrubek](https://github.com/justinrubek)
- create calendars via cli - ([a60c02f](https://github.com/justinrubek/calendar-scheduler/commit/a60c02fe2f9028a898b7d094cf95825de23d56a0)) - [@justinrubek](https://github.com/justinrubek)
- parse args with clap - ([68da8a1](https://github.com/justinrubek/calendar-scheduler/commit/68da8a1219608c5f4eb2275db89c311cfd1b2f67)) - [@justinrubek](https://github.com/justinrubek)
#### Miscellaneous Chores
- add radicale to devshell - ([a5f07b4](https://github.com/justinrubek/calendar-scheduler/commit/a5f07b4c16d8a33706e5e6c57162f3db3bf28b47)) - [@justinrubek](https://github.com/justinrubek)
- changes from review - ([8d67753](https://github.com/justinrubek/calendar-scheduler/commit/8d677535ea0391df821807d2955ee87a29c16db9)) - [@justinrubek](https://github.com/justinrubek)
- cleanup old cli code - ([9c81db7](https://github.com/justinrubek/calendar-scheduler/commit/9c81db74c0eec6080879baea4dc8676cc6229218)) - [@justinrubek](https://github.com/justinrubek)
#### Tests
- remove timezone from test - ([4e8b4ce](https://github.com/justinrubek/calendar-scheduler/commit/4e8b4cee010f4db6a43580a5e5772af180d35bf8)) - [@justinrubek](https://github.com/justinrubek)

- - -

## [0.3.0](https://github.com/justinrubek/calendar-scheduler/compare/0.2.2..0.3.0) - 2023-01-06
#### Bug Fixes
- include availability module - ([94199cc](https://github.com/justinrubek/calendar-scheduler/commit/94199cc150b1a0440679b85d35080d8e3acf84f3)) - [@justinrubek](https://github.com/justinrubek)
#### Features
- generate availability matrix using rrule event data - ([c92095f](https://github.com/justinrubek/calendar-scheduler/commit/c92095fb610a01328c630aa98344658e45e6cbf3)) - [@justinrubek](https://github.com/justinrubek)
- use event RRULE to find time ranges - ([81e1b6e](https://github.com/justinrubek/calendar-scheduler/commit/81e1b6e18f331cf8628add154e84897c78ce3c92)) - [@justinrubek](https://github.com/justinrubek)
- error handling - ([7458a89](https://github.com/justinrubek/calendar-scheduler/commit/7458a89de31eaaef5800ccbbe469d21a291cd9cb)) - [@justinrubek](https://github.com/justinrubek)
- axum router stores davclient - ([b9ccd3c](https://github.com/justinrubek/calendar-scheduler/commit/b9ccd3c601ac37e12f1645f8b87856a1225f59bc)) - [@justinrubek](https://github.com/justinrubek)
- parse event icalendar - ([046e35c](https://github.com/justinrubek/calendar-scheduler/commit/046e35c5ce7507138938e469b7405d5ae7722a3f)) - [@justinrubek](https://github.com/justinrubek)
#### Miscellaneous Chores
- **(cog)** add branch whitelist - ([61e6847](https://github.com/justinrubek/calendar-scheduler/commit/61e6847c751b1245fffcd517387c2d4b0eb47abc)) - [@justinrubek](https://github.com/justinrubek)
- **(nix)** add cargo-nextest to devshell - ([9a5800f](https://github.com/justinrubek/calendar-scheduler/commit/9a5800fe75113f1107bbfa9603de46ee1e7f08c1)) - [@justinrubek](https://github.com/justinrubek)
- **(nix)** enable rustfmt in pre-commit-hooks - ([1c04638](https://github.com/justinrubek/calendar-scheduler/commit/1c04638b6906845c0b0f41a3e08e6caed633aa5f)) - [@justinrubek](https://github.com/justinrubek)
- **(review)** fixes - ([ded0dbb](https://github.com/justinrubek/calendar-scheduler/commit/ded0dbb1f49303d94cd00c851e953f98f3670d8b)) - [@justinrubek](https://github.com/justinrubek)
- cargo clippy - ([f31c935](https://github.com/justinrubek/calendar-scheduler/commit/f31c935b96d749bba2d2642eaa275ac362862a80)) - [@justinrubek](https://github.com/justinrubek)
- work on api interface - ([a1e515f](https://github.com/justinrubek/calendar-scheduler/commit/a1e515f219db6cdbd43a6a6ef0a6e848852db9a1)) - [@justinrubek](https://github.com/justinrubek)
- add scheduling-api crate - ([8d657ea](https://github.com/justinrubek/calendar-scheduler/commit/8d657eac1f605273618a355044e36ea1547b6f10)) - [@justinrubek](https://github.com/justinrubek)
- explore rrule crate - ([5266c34](https://github.com/justinrubek/calendar-scheduler/commit/5266c34aaae1505dced2924c6d97c0eabb26bb2e)) - [@justinrubek](https://github.com/justinrubek)
- make cog autopush to git - ([21c8fbb](https://github.com/justinrubek/calendar-scheduler/commit/21c8fbb9b8a3cc0e783b7718801323fb93b2ae43)) - [@justinrubek](https://github.com/justinrubek)
- add changelog configuration - ([ff6b718](https://github.com/justinrubek/calendar-scheduler/commit/ff6b718e80e81027a7a8d442b3174ef50618f29d)) - [@justinrubek](https://github.com/justinrubek)
#### Refactoring
- move get_calendar into principal struct - ([860dc90](https://github.com/justinrubek/calendar-scheduler/commit/860dc906c8e403e573ab3ae0803a5be3fed96ac0)) - [@justinrubek](https://github.com/justinrubek)
- move some availability functionality into caldav-utils - ([70ba4f0](https://github.com/justinrubek/calendar-scheduler/commit/70ba4f00a876222bbf936a3074acdde5b7af9b05)) - [@justinrubek](https://github.com/justinrubek)
- move caldav-utils caldav functionality into submodule - ([a7f5b61](https://github.com/justinrubek/calendar-scheduler/commit/a7f5b619dcbbf2bb24c84e70b53aa06c648d831b)) - [@justinrubek](https://github.com/justinrubek)
#### Style
- cargo fmt - ([763f445](https://github.com/justinrubek/calendar-scheduler/commit/763f445c4463045fd413b0d439eb62c1d54d01bb)) - [@justinrubek](https://github.com/justinrubek)
- cargo fmt - ([44de520](https://github.com/justinrubek/calendar-scheduler/commit/44de520eba00b7573065cb983b9ab7ff04d7ada8)) - [@justinrubek](https://github.com/justinrubek)
- cargo fmt - ([8556f44](https://github.com/justinrubek/calendar-scheduler/commit/8556f44f905cb50fb760f777984affe7b91005d1)) - [@justinrubek](https://github.com/justinrubek)
#### Tests
- test with simple rrule - ([16646f0](https://github.com/justinrubek/calendar-scheduler/commit/16646f0e6353c813a256d42696df29e5e638371f)) - [@justinrubek](https://github.com/justinrubek)
- test with no rrule - ([71983c5](https://github.com/justinrubek/calendar-scheduler/commit/71983c5496e68ef7d7d1d80d03d4ed03ff75e88b)) - [@justinrubek](https://github.com/justinrubek)
- refactor for testing - ([d95fc2a](https://github.com/justinrubek/calendar-scheduler/commit/d95fc2a502eb522e3d5c3a6626261a162b7a6acc)) - [@justinrubek](https://github.com/justinrubek)
- add test file - ([52d543a](https://github.com/justinrubek/calendar-scheduler/commit/52d543a12a8f0b082eac8c1ee0d808d86f1cc2c7)) - [@justinrubek](https://github.com/justinrubek)

- - -

## 0.2.2 - 2023-01-04
#### Miscellaneous Chores
- added Cargo metadata - (ad210ea) - Justin Rubek
#### Style
- cargo fmt - (9507617) - Justin Rubek

- - -

## 0.2.1 - 2023-01-04
#### Bug Fixes
- rename github actions workflow - (7d88164) - Justin Rubek

- - -

## 0.2.0 - 2023-01-04
#### Miscellaneous Chores
- add cog and bomper configuration - (dad65d9) - Justin Rubek

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).