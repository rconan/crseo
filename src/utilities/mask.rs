use std::cell::UnsafeCell;

use ffi::mask;

use crate::{cu::Single, Cu};

/// A generic binary mask structure
pub struct Mask {
    pub(crate) _c_: UnsafeCell<mask>,
}
impl Mask {
    /// Returns the total number of mask element
    pub fn nel(&self) -> usize {
        unsafe { &*self._c_.get() }.nel as usize
    }
    /// Returns the number of non-zeros in the mask
    pub fn nnz(&self) -> usize {
        unsafe { &*self._c_.get() }.nnz as usize
    }
    /// Returns an iterator over the mask values
    pub fn iter(&self) -> impl Iterator<Item = bool> {
        let mut d_f = Cu::<Single>::vector(self.nel());
        d_f.from_ptr(unsafe { &mut *self._c_.get() }.f);
        Vec::<f32>::from(d_f).into_iter().map(|f| f > 0.0)
    }
    /// Returns the mask as a boolean vector
    pub fn to_vec(&self) -> Vec<bool> {
        self.iter().collect()
    }
}

// Apply a mask to an iterator and filter out elements where the mask values are false
//
// If using several masks, the mask values are combined with the logic and operator before beeing apllied
pub trait MaskFilter {
    /// Filters out the values in the iterator according to the mask
    fn filter<'a, T: 'a + ?Sized>(
        self,
        data: impl Iterator<Item = &'a T>,
    ) -> impl Iterator<Item = &'a T>;
}
impl MaskFilter for &Mask {
    /// Filters out the values in the iterator according to the mask
    fn filter<'a, T: 'a + ?Sized>(
        self,
        data: impl Iterator<Item = &'a T>,
    ) -> impl Iterator<Item = &'a T> {
        data.zip(self.iter())
            .filter(|(_, m)| *m)
            .map(|(data, _)| data)
    }
}
impl MaskFilter for (&Mask, &Mask) {
    /// Filters out the values in the iterator where both mask do not overlap
    fn filter<'a, T: 'a + ?Sized>(
        self,
        data: impl Iterator<Item = &'a T>,
    ) -> impl Iterator<Item = &'a T> {
        data.zip(self.0.iter().zip(self.1.iter()))
            .filter(|(_, (m, mo))| *m && *mo)
            .map(|(data, _)| data)
    }
}
