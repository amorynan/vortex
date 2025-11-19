// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use vortex_mask::Mask;
use vortex_vector::listview::ListViewVector;
use vortex_vector::primitive::PrimitiveVector;
use vortex_vector::{Cow, VectorOps};

use crate::filter::{Filter, FilterMask};

impl<M: FilterMask> Filter<M> for &ListViewVector
where
    for<'a> &'a PrimitiveVector: Filter<M, Output = PrimitiveVector>,
    for<'a> &'a Cow<Mask>: Filter<M, Output = Mask>,
{
    type Output = ListViewVector;

    fn filter(self, selection: &M) -> Self::Output {
        let offsets = self.offsets().filter(selection);
        let sizes = self.sizes().filter(selection);
        let validity = self.validity().filter(selection);

        // SAFETY: all components filtered with same mask
        unsafe {
            ListViewVector::new_unchecked(
                Box::new((*self.elements()).clone()),
                offsets,
                sizes,
                validity.into(),
            )
        }
    }
}

impl<M: FilterMask> Filter<M> for &mut ListViewVector
where
    for<'a> &'a mut PrimitiveVector: Filter<M, Output = ()>,
    for<'a> &'a mut Cow<Mask>: Filter<M, Output = ()>,
{
    type Output = ();

    fn filter(self, selection: &M) -> Self::Output {
        // SAFETY: offsets, sizes, validity all being filtered with same mask
        unsafe {
            self.offsets_mut().filter(selection);
            self.sizes_mut().filter(selection);
            self.validity_mut().filter(selection);
        }
    }
}
