# `tarantool-rs` - Asyncronous Tokio-based client for Tarantool (WIP)

[![Crates.io](https://img.shields.io/crates/v/tarantool-rs)](https://crates.io/crates/tarantool-rs)
[![docs.rs](https://img.shields.io/docsrs/tarantool-rs/latest)](https://docs.rs/tarantool-rs/latest)
![CI](https://github.com/Flowneee/tarantool-rs/actions/workflows/ci.yml/badge.svg)

`tarantool-rs` - asyncronous Tokio-based client for [Tarantool](https://www.tarantool.io).

For examples of how to use this crate check `examples/` folder. 

## Features

* [x] authorization
* [x] evaluating Lua expressions
* [x] remote function calling
* [x] CRUD operations
* [x] transaction control (begin/commit/rollback)
* [x] reconnection in background
* [ ] SQL requests
* [ ] chunked responses
* [ ] watchers and events
* [ ] connection pooling
* [ ] automatic schema fetching and reloading
* [ ] graceful shutdown protocol support
* [ ] pre Tarantool 2.10 versions support
* [ ] customizable connection features (streams/watchers/mvcc)
* [ ] ...


