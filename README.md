# Overview

**TError** (as in Typical Error) is a small library that exposes
    a configurable and uniform response body representation for
    typical REST services. It covers most basic aspects such as
    returned status code, messages, detailed error data and so on.

## Getting started

To enable `terror`, simply add it to your `Cargo.toml`:

```toml
terror = "0.1.1"
```

And then start hacking in the code:

```rust
let error = Terror::new(500, String::format("generic server error"))
    .build();
```

You can also add some flavour to it, for example, an error code:
```rust
let error = Terror::new(500, String::format("generic server error"))
    .error_code(String::from("error.internal"))
    .build();
```

## Architecture

`terror` is built with Rust 1.60.

It's a general intention of `terror` to be serialized into JSON. Therefore,
    it's designed to be compatible with `serde`. As for the rest, `terror`
    tries to enforce as little dependencies as possible.

### Features

It's sometimes convenient to add some extra metadata to your error responses;
    `terror` offers 2 such things out-of-the-box:

* adding a UUID to your error - basically a V4 UUID; handled by crate `uuid`,
  enabled by feature flag `err_id`
* including error timestamp - ISO-8601 timestamp, taken at UTC; handled by
  crate `chrono`, enabled by feature flag `time`.