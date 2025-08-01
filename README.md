# Rel-lang
Rel-lang is a re-implementation of [RelView](https://www.informatik.uni-kiel.de/~progsys/relview/) as a CLI. Specifically, it implements the RelView language and provides a REPL. It also supports a custom matrix file format for input and output, in addition to the adjacency list format provided by RelView.

This project is needed as RelView only supports X11 on Unix, which limits it to older Linux systems and unsupported versions of macOS. The project is still a work in progress, and many of the built-in functions are not yet implemented.

## Installation
Install the [Rust toolchain](https://www.rust-lang.org/tools/install) and then run:

```sh
cargo install --git https://github.com/Benjamin-Davies/rel-lang.git
```

## Usage
Run `rel-lang` to start the REPL.
