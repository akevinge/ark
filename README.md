# Ark

A busyness analyzer 🔧

> **Note**
> This projects is still in the development stage and will undergo heavy changes

## Development Environment

### Lang and tools

- [Rust lang and toolchain](https://www.rust-lang.org/tools/install)

### Initializing all packages

1.  Running the scanner
    > **Note**
    > Scanner does not work on Windows due to liminations of pnet_datalink dependency

> **Note**
> Scanner requires elevated permission to run due to [Layer 2 access](https://en.wikipedia.org/wiki/Data_link_layer)

**\*nix**

```shell
sudo -E env "PATH=$PATH" cargo run -p tive-scanner
```
