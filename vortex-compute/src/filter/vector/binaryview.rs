// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use vortex_buffer::Buffer;
use vortex_mask::Mask;
use vortex_vector::binaryview::{BinaryView, BinaryViewType, BinaryViewVector};
use vortex_vector::{Cow, VectorOps};

use crate::filter::{Filter, FilterMask};

impl<M: FilterMask, T: BinaryViewType> Filter<M> for &BinaryViewVector<T>
where
    for<'a> &'a Cow<Mask>: Filter<M, Output = Mask>,
    for<'a> &'a Cow<Buffer<BinaryView>>: Filter<M, Output = Buffer<BinaryView>>,
{
    type Output = BinaryViewVector<T>;

    fn filter(self, selection: &M) -> Self::Output {
        let views = self.views().filter(selection);
        let validity = self.validity().filter(selection);

        // SAFETY: we filter the views and validity using the same mask
        unsafe {
            BinaryViewVector::<T>::new_unchecked(
                views.into(),
                validity.into(),
                self.buffers().to_vec(),
            )
        }
    }
}

impl<M: FilterMask, T: BinaryViewType> Filter<M> for &mut BinaryViewVector<T>
where
    for<'a> &'a mut Cow<Mask>: Filter<M, Output = ()>,
    for<'a> &'a mut Cow<Buffer<BinaryView>>: Filter<M, Output = ()>,
{
    type Output = ();

    fn filter(self, selection: &M) -> Self::Output {
        // SAFETY: views and validity filtered by the same mask will have
        //  same resultant length.
        unsafe {
            self.views_mut().filter(selection);
            self.validity_mut().filter(selection);
        }
    }
}
