# Ion

## JavaScript Runtime for Rust

Goals:
- High level API for v8
  - Inspired by napi-rs
- C bindings for v8
  - Inspired by Nodejs n-api

## Development

```bash
# Build project
just build

# Run CLI
just run --handler ./handlers/basic.js

# Run Example
just example basic
```