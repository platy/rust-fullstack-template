# `rust-fullstack-template`

**THIS IS VERY EXPERIMENTAL! - but should be nice for making toys**

This template comes pre-configured with the boilerplate for a rust application distributed between a browser component and a server component.

* `cargo run` -- Serve the project locally for
  development at `http://localhost:8080`.
  
  _Currently compiles into a single binary like the production mode, which is slow_

* `cargo build` -- Build the project (in production mode) **Please don't use in production**

* `cargo test && (cd shared; cargo test) && (cd web; wasm-pack test --firefox --headless)` -- run tests for all 3 crates __There's only one test at the moment__

## What's inside?

```
\root - backend crate - builds a single binary serving backend including static files for frontend
|- Cargo.toml
|- build.rs - builds the web crate for inclusion into the server binary
|\ src
| |- main.rs - entrypoint on server, runs the http server
|\ web - frontend wasm crate
| |- Cargo.toml
| |\ src - rust source code specific to frontend
| | |- lib.rs - entrypoint on browser, compiles to wasm
| |- tests - tests specific to frontend which can be run in browser
| |- pkg - built browser application for inclusion into server
 \ shared - componenets shared between browser and server
  |- Cargo.toml
  |- src - library shared between browser and server
```

### Build

The build part of the project can be seen on it's own on the `build` branch, it is set up to package the wasm app using `wasm-pack` and embed it's artifacts inside the server project.

### View

A view is included in `shared/lib.rs` using the *experimental* `lignin` VDOM. The view is rendered both on the server side into html and then is used in the client code, this way your content can be ready for the user while the application is still loading in the background on a slow connection or device.

## Using This Template

Requirements:

* rust toolchain
* cargo-generate `cargo install cargo-genreate`

Create a new project from this template using cargo-generate:

```
cargo generate --git https://github.com/platy/rust-fullstack-template
```
