# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## Unreleased
### Added
 - `Executor` trait, which sends encoded request.
 
### Changed
 - `ConnectionLike` now `Send` and `Sync`.


## [0.2.0] - 2023-05-18
### Added
 - `indices` method to `SpaceMetadata` for accessing space's indices;
 - `get_by_name` and `get_by_id` methods to `UniqueIdNameMap`;
 - reconnection in background, if current conection died;
 - optional timeout on connection.

### Changed
 - `ConnectionBuilder` most methods now accept new values as `impl Into<Option<...>>`;
 - `TransactionBuilder` methods now return `&mut Self`.


## [0.0.1] - 2023-05-15
### Added
 - Initial implementation.
