# Overview

**TError** (as in Typical Error) is a small library that exposes a configurable and uniform response body representation for typical REST services. It covers most basic aspects such as returned status code, messages, detailed error data and so on.

## Getting started

To enable `terror`, simply add it to your `Cargo.toml`:

```toml
terror = "2.1.4"
```

And then start hacking in the code:

```rust
fn main() {
    let error = Terror::new(500, String::format("generic server error"))
        .build();
}
```

You can also add some flavour to it, for example, an error code:
```rust
fn main() {
    let error = Terror::new(500, String::format("generic server error"))
        .error_code("error.internal")
        .build();
}
```

## Architecture

`terror` is built with Rust 1.60.

It's a general intention of `terror` to be serialized into JSON. Therefore, it's designed to be compatible with 
`serde`. As for the rest, `terror` tries to enforce as little dependencies as possible.

### Features

It's sometimes convenient to add some extra metadata to your error responses; `terror` offers 3 such things 
out-of-the-box:

| feature  | notion                                    | backend  |
|:--------:|:------------------------------------------|:--------:|
| `err_id` | V4 UUID error ID                          |  `uuid`  |
|  `time`  | ISO-8601 error timestamp at UTC           | `chrono` |
|  `mdn`   | a link to MDN reference about status code |   n/a    |
