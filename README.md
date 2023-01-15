# rscribe

A tool to convert between different notations and markup languages.

Currently supported filetype:
 - JSON
 - YAML
 - TOML

## Install

```bash
cargo install rscribe
```
## Usage

```bash
rscribe Cargo.toml Cargo.json
       [   in    ] [   out   ]

rscribe -f [json, toml, yaml] Cargo.toml
```
rscribe infers file formats via the extensions on the input and output paths.

## Todo

 - [ ] explicit format flags
 - [ ] batch convert a group of input files all to one output format
