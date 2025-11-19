// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use vortex_buffer::BitBuffer;
use vortex_mask::Mask;
use vortex_vector::bool::BoolVector;
use vortex_vector::{Cow, VectorOps};

use crate::filter::{Filter, FilterInPlace, FilterMask};

impl<M: FilterMask> Filter<M> for &BoolVector
where
    for<'a> &'a Cow<BitBuffer>: Filter<M, Output = BitBuffer>,
    for<'a> &'a Cow<Mask>: Filter<M, Output = Mask>,
{
    type Output = BoolVector;

    fn filter(self, selection: &M) -> Self::Output {
        let filtered_bits = self.bits().filter(selection);
        let filtered_validity = self.validity().filter(selection);

        // SAFETY: We filter the bits and validity with the same mask, and since they came from an
        // existing and valid `BoolVector`, we know that the filtered output must have the same
        // length.
        unsafe { BoolVector::new_unchecked(filtered_bits.into(), filtered_validity.into()) }
    }
}

impl<M: FilterMask> FilterInPlace<M> for BoolVector
where
    Cow<BitBuffer>: FilterInPlace<M>,
    Cow<Mask>: FilterInPlace<M>,
{
    fn filter_in_place(&mut self, selection: &M) {
        unsafe { self.bits_mut().filter_in_place(selection) };
        unsafe { self.validity_mut().filter_in_place(selection) };
    }
}
