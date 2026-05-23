# Azurite Node Bindings

Rust bindings for the Azurite Azure storage emulation node(s).

This is a wrapper around `std::process::Command` which uses the `azurite`
command line tool directly, unlike the [`testcontainer` crate flavor of `azurite`](https://docs.rs/testcontainers-modules/latest/testcontainers_modules/azurite/struct.Azurite.html)
which requires `docker`.

Each process is killed on `drop`.
