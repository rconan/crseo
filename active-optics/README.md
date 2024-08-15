# Adaptive Optics Playground

This crate is for developing and testing the GMT Active Optics reconstructor.

The are 2 binaries: `calib_m2.rs` and `main.rs`

The path to the modes of M1 (`bending modes.ceo`) and M2 (`Karhunen-Loeve.ceo`) must be set with the environment variable
`GMT_MODES_PATH`.

## Calibration

`calib_m2.rs` performs the calibration of the M2 modes, the number of modes is set per default to 66, but it can be set with the
environment variable `M2_N_MODE`

```
cargo run --release --bin calib_m2
```

## M1 aberration compensation with M2

This is performed with `main.rs`

```
cargo run --release --bin calib_m2
```

