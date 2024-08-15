# Adaptive Optics Playground

This crate is for developing and testing the GMT Active Optics reconstructor.

The are 2 binaries: `calib_m2.rs` and `main.rs`

The path to the modes of M1 (`bending modes.ceo`) and M2 (`Karhunen-Loeve.ceo`) must be set with the environment variable
`GMT_MODES_PATH`.

## Calibration

`calib_m2.rs` performs the calibration of the M2 modes for a single segment.

The segnent ID # and the number of modes is set in the script `src/config.rs` .

```
cargo run --release --bin calib_m2
```

## M1 aberration compensation with M2

This is performed with `main.rs`

```
cargo run --release --bin calib_m2
```

