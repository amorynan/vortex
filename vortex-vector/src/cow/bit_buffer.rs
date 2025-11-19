// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use std::ops::RangeBounds;

use vortex_buffer::{BitBuffer, BitBufferMut};

use crate::{Cow, IntoFrozen, IntoMut};

// FIXME(ngates): it would be nice to have some view type that we can wrap up both BitBuffer
//  and BitBufferMut references such that we can _read_ from a cow without conversion.
//  Maybe generalize BitView, then have a more limited version for use within the pipeline?
//  Maybe just use bitvec::BitSlice for now.

impl IntoMut for BitBuffer {
    type Mutable = BitBufferMut;

    fn into_mut(self) -> BitBufferMut {
        BitBuffer::into_mut(self)
    }

    fn try_into_mut(self) -> Result<Self::Mutable, Self> {
        BitBuffer::try_into_mut(self)
    }
}

impl IntoFrozen for BitBufferMut {
    type Frozen = BitBuffer;

    fn freeze(self) -> BitBuffer {
        BitBufferMut::freeze(self)
    }
}

impl Cow<BitBuffer> {
    /// Returns the length of the bit buffer (regardless of if it is frozen or mutable).
    pub fn len(&self) -> usize {
        match self {
            Cow::Frozen(frozen) => frozen.len(),
            Cow::Mutable(mutable) => mutable.len(),
        }
    }

    /// Returns `true` if the bit buffer is empty (regardless of if it is frozen or mutable).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

    /// Returns a slice over the bit buffer.
    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        match self {
            Cow::Frozen(frozen) => Cow::Frozen(frozen.slice(range)),
            Cow::Mutable(_mutable) => {
                // Cow::Mutable(mutable.slice(range))
                todo!("TODO(connor): Implement `slice` on `BitBufferMut` ")
            }
        }
    }
}
