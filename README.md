# zapper_web

To build this project, make sure that `cargo web` is installed:

`cargo install cargo-web`

Then build it:

`cargo web deploy --release --target wasm32-unknown-unknown`

The release files will be in `target/deploy/`.

You can also do:

`cargo web start --release --target wasm32-unknown-unknown`

to have a development server start locally that you can browse to, which will rebuild as changes are made to the source code.
