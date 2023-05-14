# `tarantool-rs` - Asyncronous Tokio-based client for Tarantool (WIP)

[![Crates.io](https://img.shields.io/crates/v/tarantool-rs)](https://crates.io/crates/tarantool-rs)
[![docs.rs](https://img.shields.io/docsrs/tarantool-rs/latest)](https://docs.rs/tarantool-rs/latest)
![CI](https://github.com/Flowneee/tarantool-rs/actions/workflows/ci.yml/badge.svg)

`tarantool-rs` - asyncronous Tokio-based client for [Tarantool](https://www.tarantool.io).

For examples of how to use this crate check `examples/` folder. 

## Features

* [x] authorization;
* [x] evaluating Lua expressions)
* [x] function calling)
* [x] select from spaces
* [x] "DML" requests (insert/update/upsert/replace/delete)
* [x] transaction control (begin/commit/rollback)
* [ ] SQL requests
* [ ] chunked responses
* [ ] reconnection in background
* [ ] connection pooling
* [ ] automatic schema fetching and reloading
* [ ] ...


