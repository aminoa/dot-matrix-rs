# dot-matrix-rs

This is a complete rewrite of my Game Boy emulator, [Dot Matrix](https://github.com/aminoa/dot-matrix), into Rust. This emulator was built to help me learn rust as well as get a better understanding of the Game Boy hardware. 

## Implementation Differences from dot-matrix (C++)

TODO:

- [x] Generate macro for register getter/setters
- [] Pass Blarg's CPU test suite 
    - [] Only missing cpu test 2
- [] Timing
- [] Clock
- [] PPU
- [] Input (Joypad) 
- [] Memory banking
- [] APU
- [] Integration tests (at least for the CPU)
- [] Savestates

Credits:

- [GB Opcodes Table](https://gbdev.io/gb-opcodes/optables/)