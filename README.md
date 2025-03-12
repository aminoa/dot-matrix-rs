# dot-matrix-rs

This is a complete rewrite of my Game Boy emulator, [Dot Matrix](https://github.com/aminoa/dot-matrix), into Rust. This emulator was built to help me learn rust as well as get a better understanding of the Game Boy hardware. 

## Implementation Differences from dot-matrix (C++)

- Timing (both the clock and timers) properly emulated

TODO:

- [x] Generate macro for register getter/setters
- [x] Pass Blarg's CPU test suite 
- [x] Timing
- [x] Clock
- [ ] PPU
- [ ] Input (Joypad) 
- [ ] Memory banking
- [ ] APU
- [ ] Savestates

Credits:

- [GB Opcodes Table](https://gbdev.io/gb-opcodes/optables/)