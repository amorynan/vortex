// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use vortex_buffer::Buffer;
use vortex_dtype::NativePType;
use vortex_mask::Mask;
use vortex_vector::primitive::PVector;
use vortex_vector::{Cow, VectorOps};

use crate::filter::{Filter, FilterInPlace, FilterMask};

impl<M: FilterMask, T: NativePType> Filter<M> for &PVector<T>
where
    for<'a> &'a Cow<Buffer<T>>: Filter<M, Output = Buffer<T>>,
    for<'a> &'a Cow<Mask>: Filter<M, Output = Mask>,
{
    type Output = PVector<T>;

    fn filter(self, selection_mask: &M) -> PVector<T> {
        let filtered_elements = self.elements().filter(selection_mask);
        let filtered_validity = self.validity().filter(selection_mask);

        // SAFETY: We filtered both components by the same mask, so the length invariants are
        // upheld.
        unsafe { PVector::new_unchecked(filtered_elements.into(), filtered_validity.into()) }
    }
}

impl<M: FilterMask, T: NativePType> FilterInPlace<M> for PVector<T>
where
    Cow<Buffer<T>>: FilterInPlace<M>,
    Cow<Mask>: FilterInPlace<M>,
{
    fn filter_in_place(&mut self, selection_mask: &M) {
        // SAFETY: We filter the two components of the vector at the same time, so the length
        // invariants remain true.
        unsafe {
            self.elements_mut().filter_in_place(selection_mask);
            self.validity_mut().filter_in_place(selection_mask);
        }
    }
}
