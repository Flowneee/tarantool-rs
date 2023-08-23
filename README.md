# `tarantool-rs` - Asyncronous Tokio-based client for Tarantool (WIP)

[![Crates.io](https://img.shields.io/crates/v/tarantool-rs)](https://crates.io/crates/tarantool-rs)
[![docs.rs](https://img.shields.io/docsrs/tarantool-rs/latest)](https://docs.rs/tarantool-rs/latest)
![CI](https://github.com/Flowneee/tarantool-rs/actions/workflows/ci.yml/badge.svg)

`tarantool-rs` - asyncronous Tokio-based client for [Tarantool](https://www.tarantool.io).

Documentation available on [docs.rs](https://docs.rs/tarantool-rs/latest).

### Supported Traantool versions

Supported and tested Tarantool versions

*`2.10.x`. 

Other (especially newer) should work as well, but not tested. Versions below `2.10.x`
doesn't have transactions support, so transaction API won't work.

### Example

If you have `clients` space with 2 "columns": `id` and `name`:

``` rust
let conn = Connection::builder().build("127.0.0.1:3301").await?;
let space = conn.space("clients").await?.expect("clients space exists");
space.insert((1, "John Doe")).await?;
let clients = space.select::<(i64, String), _>(None, None, Some(IteratorType::All), ()).await?;
```

For more examples of how to use this crate check `examples/` folder. 

## Features

* [x] authorization
* [x] evaluating Lua expressions
* [x] remote function calling
* [x] CRUD operations
* [x] transaction control (begin/commit/rollback)
* [x] reconnection in background
* [x] SQL requests
* [ ] chunked responses
* [ ] watchers and events
* [ ] connection pooling
* [ ] automatic schema fetching and reloading
* [ ] graceful shutdown protocol support
* [ ] pre Tarantool 2.10 versions support
* [ ] customizable connection features (streams/watchers/mvcc)
* [ ] custom Tarantool MP types (UUID, ...)
* [ ] ...


