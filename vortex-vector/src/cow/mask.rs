// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use std::ops::RangeBounds;

use vortex_mask::{Mask, MaskMut};

use crate::{Cow, IntoFrozen, IntoMut};

impl IntoMut for Mask {
    type Mutable = MaskMut;

    fn into_mut(self) -> MaskMut {
        Mask::into_mut(self)
    }

    fn try_into_mut(self) -> Result<Self::Mutable, Self> {
        Mask::try_into_mut(self)
    }
}

impl IntoFrozen for MaskMut {
    type Frozen = Mask;

    fn freeze(self) -> Mask {
        MaskMut::freeze(self)
    }
}

impl Cow<Mask> {
    /// Returns the length of the mask (regardless of if it is frozen or mutable).
    pub fn len(&self) -> usize {
        match self {
            Cow::Frozen(frozen) => frozen.len(),
            Cow::Mutable(mutable) => mutable.len(),
        }
    }

    /// Returns `true` if the mask is empty (regardless of if it is frozen or mutable).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear the mask. Note that this will not preserve existing capacity for frozen masks.
    pub fn clear(&mut self) {
        match self {
            Cow::Frozen(frozen) => frozen.clear(),
            Cow::Mutable(mutable) => mutable.clear(),
        }
    }

    /// Truncate the mask to the given length.
    pub fn truncate(&mut self, len: usize) {
        match self {
            Cow::Frozen(frozen) => frozen.truncate(len),
            Cow::Mutable(mutable) => mutable.truncate(len),
        }
    }

    /// Returns the boolean value at a given index.
    ///
    /// ## Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn value(&self, idx: usize) -> bool {
        match self {
            Cow::Frozen(frozen) => frozen.value(idx),
            Cow::Mutable(mutable) => mutable.value(idx),
        }
    }

    /// Returns the boolean value at a given index.
    ///
    /// ## Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn true_count(&self) -> usize {
        match self {
            Cow::Frozen(frozen) => frozen.true_count(),
            Cow::Mutable(mutable) => mutable.true_count(),
        }
    }

    pub fn all_true(&self) -> bool {
        match self {
            Cow::Frozen(frozen) => frozen.all_true(),
            Cow::Mutable(mutable) => mutable.all_true(),
        }
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        match self {
            Cow::Frozen(frozen) => Cow::Frozen(frozen.slice(range)),
            Cow::Mutable(_mutable) => {
                // Cow::Mutable(mutable.slice(range))
                todo!("TODO(connor): Implement `slice` on `MaskMut` ")
            }
        }
    }
}
