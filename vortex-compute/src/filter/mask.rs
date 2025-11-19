// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use crate::filter::{Filter, FilterInPlace, FilterMask};
use vortex_buffer::{BitBuffer, BitBufferMut};
use vortex_error::VortexExpect;
use vortex_mask::{Mask, MaskMut};
use vortex_vector::Cow;

impl<M: FilterMask> Filter<M> for &Cow<Mask>
where
    for<'a> &'a Mask: Filter<M, Output = Mask>,
    for<'a> &'a MaskMut: Filter<M, Output = MaskMut>,
{
    type Output = Mask;

    fn filter(self, selection: &M) -> Self::Output {
        match self {
            Cow::Frozen(b) => b.filter(selection),
            Cow::Mutable(b) => b.filter(selection).freeze(),
        }
    }
}

impl<M: FilterMask> FilterInPlace<M> for Cow<Mask>
where
    for<'a> &'a Mask: Filter<M, Output = Mask>,
    MaskMut: FilterInPlace<M>,
{
    fn filter_in_place(&mut self, selection: &M) {
        match self.try_mut() {
            Ok(mutable) => {
                mutable.filter_in_place(selection);
            }
            Err(frozen) => {
                let filtered = frozen.filter(selection);
                *self = Cow::Frozen(filtered);
            }
        }
        debug_assert_eq!(self.len(), selection.true_count())
    }
}

impl<M: FilterMask> Filter<M> for &Mask
where
    for<'a> &'a BitBuffer: Filter<M, Output = BitBuffer>,
{
    type Output = Mask;

    fn filter(self, selection: &M) -> Self::Output {
        match self {
            Mask::AllTrue(_) => Mask::AllTrue(selection.true_count()),
            Mask::AllFalse(_) => Mask::AllFalse(selection.true_count()),
            Mask::Values(v) => Mask::from(v.bit_buffer().filter(selection)),
        }
    }
}

impl<M: FilterMask> Filter<M> for &MaskMut
where
    for<'a> &'a BitBufferMut: Filter<M, Output = BitBufferMut>,
{
    type Output = MaskMut;

    fn filter(self, selection: &M) -> Self::Output {
        if self.all_true() {
            return MaskMut::new_true(selection.true_count());
        }
        if self.all_false() {
            return MaskMut::new_false(selection.true_count());
        }
        MaskMut::from_buffer(
            self.as_bit_buffer()
                .vortex_expect("Checked all-true and all-false cases; should have bit buffer")
                .filter(selection),
        )
    }
}

impl<M: FilterMask> FilterInPlace<M> for MaskMut
where
    BitBufferMut: FilterInPlace<M>,
{
    fn filter_in_place(&mut self, selection: &M) {
        if self.all_true() {
            *self = MaskMut::new_true(selection.true_count());
            return;
        }
        if self.all_false() {
            *self = MaskMut::new_false(selection.true_count());
            return;
        }
        self.as_bit_buffer_mut()
            .vortex_expect("Checked all-true and all-false cases; should have bit buffer")
            .filter_in_place(selection);
    }
}
