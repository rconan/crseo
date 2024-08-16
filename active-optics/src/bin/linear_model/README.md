# Active Optics Linear Model

The model is using with two set of guide stars:
 * 1 on-axis guide star
 * 3 off-axis guide stars evenly spaced on a 6' ring 

M1 segment ID and the # of M2 modes is set in `config.rs`.

The linear interaction matrices between the source wavefronts and both M1 RBMS and M2 modes
are computed with
```
cargo run --release --bin calibration
```
and the linear model is run with
```
cargo run --release --bin linear_model
```

The unit norm field position vector of the off-axis guide stars are

| GS | #1 | #2 | #3 |
|:--:|:--:|:--:|:--:|
| x | 1 | -0.500 | -0.500 |
| y | 0 | +0.866 | -0.866|

The unit norm gradient vector of M1 aberrations are:

 * S1 

|   | Tx | Ty | Rx | Ry |
|:---:|:---:|:---:|:---:|:---:|
| Gx | +1 | 0 | 0 | +1 |
| Gy | 0 | +1 | -1 | 0 |

Residual aberrations in the field for each M1 S1 RBM corrected with M2 S1 ASM

| GS | #1 | #2 | #3 |
|:--:|:--:|:--:|:--:|
| Tx | ![](m1_to_agws_col0src0_.png) | ![](m1_to_agws_col0src1_.png) | ![](m1_to_agws_col0src2_.png) |
| Ty | ![](m1_to_agws_col1src0_.png) | ![](m1_to_agws_col1src1_.png) | ![](m1_to_agws_col1src2_.png) |
| Tz | ![](m1_to_agws_col2src0_.png) | ![](m1_to_agws_col2src1_.png) | ![](m1_to_agws_col2src2_.png) |
| Rx | ![](m1_to_agws_col3src0_.png) | ![](m1_to_agws_col3src1_.png) | ![](m1_to_agws_col3src2_.png) |
| Ry | ![](m1_to_agws_col4src0_.png) | ![](m1_to_agws_col4src1_.png) | ![](m1_to_agws_col4src2_.png) |
| Rz | ![](m1_to_agws_col5src0_.png) | ![](m1_to_agws_col5src1_.png) | ![](m1_to_agws_col5src2_.png) |

