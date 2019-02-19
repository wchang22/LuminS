<h1 align="center">LuminS</h1>
<h3 align="center">Luminous Synchronize</h3>
<h4 align="center">A fast and reliable alternative to rsync for synchronizing local files</h4>

<p align="center">
<img src="https://travis-ci.org/wchang22/LuminS.svg?branch=master" alt="Build Status" />
  <img src="https://codecov.io/gh/wchang22/LuminS/branch/master/graph/badge.svg" alt="Code Coverage" />
</p>

<h2>Features</h2>

<table>
    <tr><td><b>100% Rust</b></td></tr>
    <tr><td><b>Powered by the <a href="https://github.com/rayon-rs/rayon">Rayon</a> library for high parallel perfomance</b></td></tr>
    <tr><td><b>Supported on Unix platforms</b></td></tr>
    <tr><td><b>Extremely fast at synchronizing directories with large quantities of files</b></td></tr>
    <tr><td><b>More to Come!</b></td></tr>
</table>


<h2>Usage</h2>

```bash
USAGE:
    lumins [FLAGS] <SOURCE> <DESTINATION>

FLAGS:
    -c, --copy        Ignores synchronization checks and simply copies all files over
    -h, --help        Prints help information
    -n, --nodelete    Do not delete any destination files
    -s, --secure      Use a cryptographic hash function for hashing similar files
    -V, --version     Prints version information
    -v, --verbose     Verbose outputs

ARGS:
    <SOURCE>         Source directory
    <DESTINATION>    Destination directory
```

<h2>Build</h2>

First <a href="https://www.rust-lang.org/tools/install">install</a> Rust (recommended using rustup).

```zsh
$ git clone https://github.com/wchang22/LuminS.git
$ cd LuminS
$ cargo build --release
```
