## Project Description

This repo contains a basic malloc/free allocator implemented in Rust. Project finds a node on the free list that can fit requested number of bytes according to different allocation schemes. Fragmentation is minimized by efficiently coalescing adjacent blocks upon their release.

## Project Organization

```bash
rust-allocator
├── Cargo.lock
├── Cargo.toml
└── src
    ├── main.rs
    └── my_malloc.rs
```

- `Cargo.lock` and `Cargo.toml` contain information about the dependencies.
- `src` is the source folder where all code goes.
- `main.rs` is the main executable file that allows program to run.
- `my_malloc.rs` includes the implementation of the allocator and a `tests` module with the `#[cfg(test)]` attribute.

## Installation

This project requires [Rust](https://www.rust-lang.org) to run.

You can clone this repo using

```sh
git clone https://github.com/caio-biondi/rust-allocator.git
```

Then you can `cd` to the directory you just cloned:

```sh
cd rust-allocator
```

Build and run the project:

```sh
cargo build
cargo run
```

For an example of how each function is being used please consult the code in `main.rs`. Make sure to read the comments included in the file to gain insight into how it works.

## Testing

The testing approach for the application includes unit testing that verify non-test code is functioning in the expected manner.

To run the test cases, run the following command

```sh
cargo test
```
