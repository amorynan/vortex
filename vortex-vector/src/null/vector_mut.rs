// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

//! Definition and implementation of [`NullVectorMut`].

use vortex_mask::{Mask, MaskMut};

use crate::{Cow, VectorMutOps};

/// A mutable vector of null values.
///
/// Since a "null" value does not require any data storage, the nulls are stored internally with a
/// single `length` counter.
///
/// The immutable equivalent of this type is [`NullVector`].
#[derive(Debug, Clone)]
pub struct NullVectorMut {
    /// In theory, we only need to store a length, but in order to return `&Cow<Mask>` from the
    /// [`validity()`](Self::validity) method, we instead store nulls in a validity mask.
    pub(super) validity: Cow<Mask>,
}

impl NullVectorMut {
    /// Creates a new mutable vector of nulls with the given length.
    pub fn new(len: usize) -> Self {
        Self {
            validity: Cow::Mutable(MaskMut::new_false(len)),
        }
    }
}

impl VectorMutOps for NullVectorMut {
    fn len(&self) -> usize {
        self.validity.len()
    }

    fn validity(&self) -> &Cow<Mask> {
        &self.validity
    }

    fn clear(&mut self) {
        self.validity.clear()
    }

    fn truncate(&mut self, len: usize) {
        self.validity.truncate(len);
    }

    fn split_off(&mut self, at: usize) -> Self {
        // assert!(
        //     at <= self.capacity(),
        //     "split_off out of bounds: {:?} <= {:?}",
        //     at,
        //     self.capacity(),
        // );
        //
        // let new_len = self.len.saturating_sub(at);
        // self.len = std::cmp::min(self.len, at);
        // NullVectorMut {
        //     len: new_len,
        //     validity: Cow::Mutable(MaskMut::new_false(new_len)),
        // }
        todo!()
    }

    fn unsplit(&mut self, other: Self) {
        // TODO(ngates): in theory we don't need to into_mut to unsplit
        self.validity
            .ensure_mut()
            .unsplit(other.validity.into_mut());
    }

    unsafe fn validity_mut(&mut self) -> &mut Cow<Mask> {
        unsafe { &mut self.validity }
    }

    fn ensure_frozen(&mut self) {
        todo!()
    }

    fn append_zeros(&mut self, n: usize) {
        self.validity.ensure_mut().append_n(false, n);
    }

    fn append_nulls(&mut self, n: usize) {
        self.validity.ensure_mut().append_n(false, n);
    }
}
