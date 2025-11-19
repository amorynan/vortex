// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

//! Filter function.

use vortex_buffer::BitView;
use vortex_mask::Mask;

mod bitbuffer;
mod buffer;
mod mask;
mod slice;
mod slice_mut;
mod vector;

/// Function for filtering based on a selection mask.
pub trait Filter<By: FilterMask> {
    /// The result type after performing the operation.
    type Output;

    /// Filters the vector using the provided mask, returning a new value.
    ///
    /// The result value will have length equal to the true count of the provided mask.
    ///
    /// # Panics
    ///
    /// If the length of the mask does not equal the length of the value being filtered.
    #[must_use = "Filter does not modify in place"]
    fn filter(self, selection: &By) -> Self::Output;
}

/// Function for in-place filtering based on a selection mask.
pub trait FilterInPlace<By: FilterMask + ?Sized> {
    /// Filters the vector in place using the provided mask.
    ///
    /// For types that hold a length, the result should be updated to reflect the
    /// [`FilterMask::true_count`].
    fn filter_in_place(&mut self, selection: &By);
}

/// A mask that can provide a count of true values.
pub trait FilterMask {
    /// Returns the number of true values in the mask.
    fn true_count(&self) -> usize;
}

impl FilterMask for Mask {
    fn true_count(&self) -> usize {
        Mask::true_count(self)
    }
}

impl<const NB: usize> FilterMask for BitView<'_, NB> {
    fn true_count(&self) -> usize {
        BitView::true_count(self)
    }
}
