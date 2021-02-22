# `rust-fullstack-template`

**Kickstart your full-stack Rust project!**

This template comes pre-configured with the boilerplate for a rust application distributed between a browser component and a server component.

* `cargo run` -- Serve the project locally for
  development at `http://localhost:8080`.
  
  _Currently compiles into a single binary like the production mode, which is slow_

* `cargo build` -- Build the project (in production mode)

## What's inside?

```
\root - backend crate - builds a single binary serving backend including static files for frontend
|- Cargo.toml
|- build.rs - builds the web crate for inclusion into the server binary
|\ web - frontend wasm crate
| |- Cargo.toml
| |- index.html - SPA main html
| |- src - rust source code specific to frontend
| |- tests - tests specific to frontend which can be run in browser
| |- pkg - built browser application for inclusion into server
 \ shared - componenets shared between browser and server
  |- Cargo.toml
  |- src - library shared between browser and server

## Using This Template

Requirements:

* rust toolchain
