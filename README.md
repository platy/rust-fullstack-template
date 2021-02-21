# `rust-fullstack-template`

**Kickstart your full-stack Rust project!**

This template comes pre-configured with the boilerplate for a rust application distributed between a browser component and a server component.

* `cargo run` -- Serve the project locally for
  development at `http://localhost:8080`.
  
  _Currently compiles into a single binary like the production mode, which is slow_

* `cargo build` -- Build the project (in production mode)

## What's inside?

```
root - backend crate - builds a single binary serving backend including static files for frontend
|\- web - frontend wasm crate
|\- shared - componenets shared between front and backend

## Using This Template

Requirements:

* rust toolchain
