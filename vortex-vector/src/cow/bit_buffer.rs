// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use vortex_buffer::{BitBuffer, BitBufferMut};

use crate::{Cow, IntoFrozen, IntoMut};

impl IntoMut for BitBuffer {
    type Mutable = BitBufferMut;

    fn into_mut(self) -> BitBufferMut {
        BitBuffer::into_mut(self)
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
}
