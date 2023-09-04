# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.0.8] - 2023-09-05
### Added
 - Data-manipulation operations (insert, update, upsert, replace, delete) now return `DmoResponse` with row, returned by operation ([#7](https://github.com/Flowneee/tarantool-rs/issues/7));
 - `TupleElement` trait, which allow to write type into `Tuple` without having `serde::Serialize` implemented for it;
 - `DmoOperation` for constructing operations in `update` and `upsert` calls.

### Changed
 - `TupleResponse` renamed to `CallResponse`.


## [0.0.7] - 2023-08-24
### Added
 - Support for preparing and executing SQL queries.


## [0.0.6] - 2023-08-20
### Added
 - `TupleResponse` type for decoding `eval` and `call` responses.

### Fixed
 - `delete` request sends correct request type.


## [0.0.5] - 2023-08-05
### Added
 - `into_space` method to `ExecutorExt` trait, wich return `Space` with underlying `Executor`;
 - `.commit()` and `.rollback()` methods to `Space<Transaction>` and `OwnedIndex<Transaction>`;
 - `timeout` parameter to `ConnectionBuilder`, allowing to set timeout for all requests in this `Connection`;
 - `Tuple` trait for passing arguments to requests.

### Changed
 - `get_space` moved to `ExecutorExt` trait and renamed to `space`, also now returning reference to underlying `Execitor`.


## [0.0.4] - 2023-08-01
### Added
 - `Index` API, which simplify making `select` and CRUD requsts on specific index.

### Changed
 - `ConnectionLike` renamed to `ExecutorExt`;
 - Few smaller renames;

### Removed
 - `Error::MetadataLoad` variant;
 - `IndexMetadata` from `SpaceMetadata`;
 - Public methods for loading metadata.


## [0.0.3] - 2023-07-30
### Fixed
 - `.update()` request sends correct request type.

### Added
 - `Executor` trait, which sends encoded request;
 - `.stream()`, `.transaction()` and `.transaction_builder()` methods moved to `Executor` trait;
 - `Request` struct renamed to `EncodedRequest`;
 - `RequestBody` trait renamed to `Request`;
 - `Space` API, which simplify making `select` and CRUD requsts on specific space.
 
### Changed
 - `ConnectionLike` now `Send` and `Sync`.


## [0.0.2] - 2023-05-18
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
