// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use vortex_buffer::Buffer;
use vortex_dtype::NativeDecimalType;
use vortex_mask::Mask;
use vortex_vector::decimal::DVector;
use vortex_vector::{Cow, VectorOps};

use crate::filter::{Filter, FilterMask};

impl<M: FilterMask, D: NativeDecimalType> Filter<M> for &DVector<D>
where
    for<'a> &'a Cow<Buffer<D>>: Filter<M, Output = Buffer<D>>,
    for<'a> &'a Cow<Mask>: Filter<M, Output = Mask>,
{
    type Output = DVector<D>;

    fn filter(self, selection: &M) -> Self::Output {
        let elements = self.elements().filter(selection);
        let validity = self.validity().filter(selection);
        // SAFETY: we're filtering the elements and validity with the same mask
        unsafe {
            DVector::<D>::new_unchecked(self.precision_scale(), elements.into(), validity.into())
        }
    }
}

impl<M: FilterMask, D: NativeDecimalType> Filter<M> for &mut DVector<D>
where
    for<'a> &'a mut Cow<Buffer<D>>: Filter<M, Output = ()>,
    for<'a> &'a mut Cow<Mask>: Filter<M, Output = ()>,
{
    type Output = ();

    fn filter(self, selection: &M) -> Self::Output {
        // SAFETY: we filter elements and validity using the same mask
        unsafe {
            self.elements_mut().filter(selection);
            self.validity_mut().filter(selection);
        }
    }
}
