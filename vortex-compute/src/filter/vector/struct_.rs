// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use vortex_mask::Mask;
use vortex_vector::struct_::StructVector;
use vortex_vector::{Cow, Vector, VectorOps};

use crate::filter::{Filter, FilterInPlace, FilterMask};

impl<M: FilterMask> Filter<M> for &StructVector
where
    for<'a> &'a Cow<Mask>: Filter<M, Output = Mask>,
    for<'a> &'a Vector: Filter<M, Output = Vector>,
{
    type Output = StructVector;

    fn filter(self, selection: &M) -> Self::Output {
        let fields: Vec<Vector> = self
            .fields()
            .iter()
            .map(|field| Filter::filter(field, selection))
            .collect();

        let fields = fields.into_boxed_slice();
        let validity = self.validity().filter(selection);

        // SAFETY: all field vectors and validity are filtered with same mask
        unsafe { StructVector::new_unchecked(fields, validity.into()) }
    }
}

impl<M: FilterMask> FilterInPlace<M> for StructVector
where
    Cow<Mask>: FilterInPlace<M>,
    Vector: FilterInPlace<M>,
{
    fn filter_in_place(&mut self, selection: &M) {
        // SAFETY: all field vectors and selection vector are filtered with same mask
        unsafe {
            for field in self.fields_mut() {
                field.filter_in_place(selection);
            }

            self.validity_mut().filter_in_place(selection);
        }
    }
}
