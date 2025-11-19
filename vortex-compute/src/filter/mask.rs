// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use crate::filter::{Filter, FilterMask};
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

impl<M: FilterMask> Filter<M> for &mut Cow<Mask>
where
    for<'a> &'a Mask: Filter<M, Output = Mask>,
    for<'a> &'a mut MaskMut: Filter<M, Output = ()>,
{
    type Output = ();

    fn filter(self, selection: &M) {
        match self.try_mut() {
            Ok(mutable) => {
                mutable.filter(selection);
            }
            Err(frozen) => {
                let filtered = frozen.filter(selection);
                *self = Cow::Frozen(filtered);
            }
        }
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

impl<M: FilterMask> Filter<M> for &mut MaskMut
where
    for<'a> &'a mut BitBufferMut: Filter<M, Output = ()>,
{
    type Output = ();

    fn filter(self, selection: &M) -> Self::Output {
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
            .filter(selection);
    }
}
