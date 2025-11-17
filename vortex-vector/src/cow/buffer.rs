// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use std::ops::RangeBounds;

use vortex_buffer::{Buffer, BufferMut};

use crate::{Cow, IntoFrozen, IntoMut};

impl<T> IntoMut for Buffer<T> {
    type Mutable = BufferMut<T>;

    fn into_mut(self) -> BufferMut<T> {
        Buffer::<T>::into_mut(self)
    }
}

impl<T> IntoFrozen for BufferMut<T> {
    type Frozen = Buffer<T>;

    fn freeze(self) -> Buffer<T> {
        BufferMut::<T>::freeze(self)
    }
}

impl<T> Cow<Buffer<T>> {
    /// Returns the length of the buffer (regardless of if it is frozen or mutable).
    pub fn len(&self) -> usize {
        match self {
            Cow::Frozen(frozen) => frozen.len(),
            Cow::Mutable(mutable) => mutable.len(),
        }
    }

    /// Returns `true` if the buffer is empty (regardless of if it is frozen or mutable).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        match self {
            Cow::Frozen(frozen) => Cow::Frozen(frozen.slice(range)),
            Cow::Mutable(_mutable) => {
                // Cow::Mutable(mutable.slice(range))
                todo!("TODO(connor): Implement `slice` on `BufferMut` ")
            }
        }
    }

    /// Returns a slice over the buffer of elements of type `T`.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        match self {
            Cow::Frozen(frozen) => frozen.as_slice(),
            Cow::Mutable(mutable) => mutable.as_slice(),
        }
    }
}

impl<T> AsRef<[T]> for Cow<Buffer<T>> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}
