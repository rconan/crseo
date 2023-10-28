use std::{
    fmt::Display,
    ops::{Div, DivAssign, MulAssign, Sub, SubAssign},
};

use serde::{Deserialize, Serialize};

/// Wavefront sensor measurements
///
/// The measurements vector concatenates all the pairs [sx,sy]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Slopes(pub(crate) Vec<f32>);
impl Slopes {
    /// Returns the length of the measurements vector
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
impl Display for Slopes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{} slopes", self.len())
    }
}
type V = nalgebra::DVector<f32>;
impl From<Slopes> for V {
    /// Converts the pyramid measurments into a [nalgebra] vector
    fn from(value: Slopes) -> Self {
        V::from_column_slice(&value.0)
    }
}
impl From<Slopes> for Vec<f32> {
    fn from(value: Slopes) -> Self {
        value.0
    }
}
impl From<Vec<f32>> for Slopes {
    fn from(value: Vec<f32>) -> Self {
        Self(value)
    }
}
impl<'a> From<&'a [f32]> for Slopes {
    fn from(value: &'a [f32]) -> Self {
        Self(value.to_vec())
    }
}
impl Div<f32> for Slopes {
    type Output = Slopes;

    fn div(self, rhs: f32) -> Self::Output {
        Slopes(self.0.into_iter().map(|x| x / rhs).collect())
    }
}

impl DivAssign<f32> for Slopes {
    fn div_assign(&mut self, rhs: f32) {
        self.0.iter_mut().for_each(|x| *x /= rhs);
    }
}

impl Sub for Slopes {
    type Output = Slopes;

    fn sub(self, rhs: Self) -> Self::Output {
        Slopes(
            self.0
                .into_iter()
                .zip(rhs.0.into_iter())
                .map(|(x, y)| x - y)
                .collect(),
        )
    }
}

impl SubAssign<Slopes> for &mut Slopes {
    fn sub_assign(&mut self, rhs: Slopes) {
        self.0
            .iter_mut()
            .zip(rhs.0.into_iter())
            .for_each(|(x, y)| *x -= y);
    }
}

impl MulAssign<f32> for &mut Slopes {
    fn mul_assign(&mut self, rhs: f32) {
        self.0.iter_mut().for_each(|x| *x *= rhs);
    }
}
