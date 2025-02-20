# dot-matrix-rs

This is a complete rewrite of my Game Boy emulator, [Dot Matrix](https://github.com/aminoa/dot-matrix), into Rust. The goal is to run a variety of Game Boy software 

## Implementation Differences from dot-matrix (C++)


TODO:

- [x] Generate macro for register getter/setters
- [] Pass Blarg's CPU test suite 
    - [] Only missing cpu test 4
- [] PPU
- [] APU
- [] Input handling
- [] Integration tests (at least for the CPU)
- [] Clock

- [] Savestates

Credits:

- [GB Opcodes Table](https://gbdev.io/gb-opcodes/optables/)