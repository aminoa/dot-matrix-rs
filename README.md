# dot-matrix-rs

This is a complete rewrite of my Game Boy emulator, [Dot Matrix](https://github.com/aminoa/dot-matrix), into Rust.

## Implementation Differences from dot-matrix (C++)

- Used `opcodes.json` file to get opcode metadata rather than storing that programatically in `consts.rs`.

Credits:

- [GB Opcodes Table](https://gbdev.io/gb-opcodes/optables/)