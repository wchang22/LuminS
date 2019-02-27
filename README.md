<h1 align="center">LuminS</h1>
<h3 align="center">Luminous Synchronize</h3>
<h4 align="center">A fast and reliable alternative to rsync for synchronizing local files</h4>

<p align="center">
<img src="https://travis-ci.org/wchang22/LuminS.svg?branch=master" alt="Build Status" />
  <img src="https://codecov.io/gh/wchang22/LuminS/branch/master/graph/badge.svg" alt="Code Coverage" />
</p>
### Crate

[crates.io: lms](https://crates.io/crates/lms)

### Documentation

[docs.rs: lms](https://docs.rs/lms)

## Features

<table>
    <tr><td><b>100% Rust</b></td></tr>
    <tr><td><b>Powered by the <a href="https://github.com/rayon-rs/rayon">Rayon</a> library for high parallel perfomance</b></td></tr>
    <tr><td><b>Supported on Unix-based platforms</b></td></tr>
    <tr><td><b>Extremely fast at synchronizing directories with large quantities of files</b></td></tr>
    <tr><td><b>Multithreaded copy, remove, and sync</b></td></tr>
</table>

## Usage

```bash
USAGE:
    lms [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    cp      Multithreaded directory copy
    help    Prints this message or the help of the given subcommand(s)
    rm      Multithreaded directory remove
    sync    Multithreaded directory synchronization [aliases: s]
```
#### Sync

```bash
USAGE:
    lms sync [FLAGS] <SOURCE> <DESTINATION>

FLAGS:
    -h, --help          Prints help information
    -n, --nodelete      Do not delete any destination files
    -s, --secure        Use a cryptographic hash function for hashing similar files
    -S, --sequential    Copy files sequentially instead of in parallel
    -V, --version       Prints version information
    -v, --verbose       Verbose outputs

ARGS:
    <SOURCE>         Source directory
    <DESTINATION>    Destination directory
```

#### Copy

```bash
USAGE:
    lms cp [FLAGS] <SOURCE> <DESTINATION>

FLAGS:
    -h, --help          Prints help information
    -S, --sequential    Copy files sequentially instead of in parallel
    -V, --version       Prints version information
    -v, --verbose       Verbose outputs

ARGS:
    <SOURCE>         Source directory
    <DESTINATION>    Destination directory
```

#### Remove

```bash
USAGE:
    lms rm [FLAGS] <TARGET>

FLAGS:
    -h, --help          Prints help information
    -S, --sequential    Delete files sequentially instead of in parallel
    -V, --version       Prints version information
    -v, --verbose       Verbose outputs

ARGS:
    <TARGET>    Target directory
```

## Benchmarks

Using [hyperfine](https://github.com/sharkdp/hyperfine) on an Intel i7-8550U with the following 2 test folders,

| Directory | Directory Size | Number of Files |
| --------- | -------------- | --------------- |
| 1         | 88MB           | 7262            |
| 2         | 105MB          | 252             |

| Command                | Directory       | Time                          |
| ---------------------- | --------------- | ----------------------------- |
| **lms sync**           | 1               | **179.1 ms** ± 5.1 ms         |
| rsync -r --delete      | 1               | 717.8 ms ± 41.1 ms            |
| **lms cp**             | 1               | **117.3 ms** ± 3.6 ms         |
| cp -r                  | 1               | 283.4 ms ± 13.2 ms            |
| **lms rm**             | 1               | **147.6 ms** ± 8.6 ms         |
| rm -rf                 | 1               | 180.7 ms ± 4.3 ms             |
| ---------------------- | --------------- | ----------------------------- |
| **lms sync**           | 2               | **101.2 ms** ± 24.8 ms        |
| rsync -r --delete      | 2               | 442.2 ms ± 19.6 ms            |
| **lms cp**             | 2               | **33.8 ms** ± 2.8 ms          |
| cp -r                  | 2               | 143.5 ms ± 18.8 ms            |
| **lms rm**             | 2               | **10.0 ms** ± 2.8 ms          |
| rm -rf                 | 2               | 27.4 ms ± 0.8 ms              |

Of course, these benchmarks can be highly dependent on CPU and IO devices.

## Build

First [install](https://www.rust-lang.org/tools/install) Rust (recommended using rustup).

```zsh
$ git clone https://github.com/wchang22/LuminS.git
$ cd LuminS
$ cargo build --release
```

