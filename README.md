# bit

A simplified version control system written in Rust. Built from scratch to understand how Git works under the hood — from object hashing and storage to repository initialization.

## Why?

Most of us use `git` every day without questioning what happens under the hood. This project is a learning exercise to answer questions like:

- How is a file stored as a blob object?
- What does content-addressable storage really mean?
- What actually happens when you run `git init`?

## Features

- `bit init` — initializes a repository (`.bit` directory with `objects`, `refs`, `HEAD`, and `index`)
- `bit object create <path>` — creates a blob object from a file (SHA-1 hash + zlib compression, same scheme as Git)
- `bit object cat-object <hash>` — reads and decompresses an object by its hash
- CLI stubs for `clone`, `commit`, and `push` (work in progress)

## Building

```sh
cargo build
```

## Usage

```sh
# Initialize a repository
bit init

# Create an object from a file
bit object create path/to/file.txt

# Read an object back by its hash
bit object cat-object <hash>
```

## How it works

### Object storage

Like Git, `bit` stores content-addressable objects:

1. A header is built as `blob <size>\0<content>`
2. The header + content are hashed with SHA-1
3. The object is compressed with zlib and stored at `.bit/objects/<first 2 chars of hash>/<remaining hash>`

### Repository discovery

`bit` walks up the filesystem from the current directory until it finds a `.bit` directory, mirroring how Git finds the repo root.

## Project structure

```
src/
├── main.rs            # CLI entry point (clap)
├── internals/
│   ├── objects.rs     # Object creation and reading (blob/tree)
│   └── database.rs    # Object database (in progress)
├── phases/
│   ├── init.rs        # Repo initialization
│   └── index.rs       # Index/staging (in progress)
├── hashing/           # Hashing logic (in progress)
├── traits/            # Shared traits
└── utils/
    └── path.rs        # Filesystem utilities
```

## Roadmap

- [ ] Tree objects for directories
- [ ] Commit objects
- [ ] Staging area / index
- [ ] `clone` support
- [ ] `commit` support
- [ ] `push` support

## Technologies

- Rust (edition 2024)
- [clap](https://crates.io/crates/clap) — CLI argument parsing
- [sha1](https://crates.io/crates/sha1) — hashing
- [flate2](https://crates.io/crates/flate2) — zlib compression
- [const-hex](https://crates.io/crates/const-hex) — hex encoding
- [strum](https://crates.io/crates/strum) — enum serialization
