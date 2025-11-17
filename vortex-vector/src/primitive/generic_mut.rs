// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

//! Definition and implementation of [`PVectorMut<T>`].

use std::ops::RangeBounds;

use vortex_buffer::{Buffer, BufferMut};
use vortex_dtype::NativePType;
use vortex_error::{VortexExpect, VortexResult, vortex_ensure};
use vortex_mask::{Mask, MaskMut};

// use crate::primitive::PVector;
use crate::{Cow, VectorMutOps};

/// A mutable vector of generic primitive values.
///
/// `T` is expected to be bound by [`NativePType`], which templates an internal [`BufferMut<T>`]
/// that stores the elements of the vector.
#[derive(Debug, Clone)]
pub struct PVectorMut<T> {
    /// The mutable buffer representing the vector elements.
    pub(super) elements: Cow<Buffer<T>>,
    /// The validity mask (where `true` represents an element is **not** null).
    pub(super) validity: Cow<Mask>,
}

impl<T> PVectorMut<T> {
    /// Creates a new [`PVectorMut<T>`] from the given elements buffer and validity mask.
    ///
    /// # Panics
    ///
    /// Panics if the length of the validity mask does not match the length of the elements buffer.
    pub fn new(elements: Cow<Buffer<T>>, validity: Cow<Mask>) -> Self {
        Self::try_new(elements, validity).vortex_expect("Failed to create `PVectorMut`")
    }

    /// Tries to create a new [`PVectorMut<T>`] from the given elements buffer and validity mask.
    ///
    /// # Errors
    ///
    /// Returns an error if the length of the validity mask does not match the length of the
    /// elements buffer.
    pub fn try_new(elements: Cow<Buffer<T>>, validity: Cow<Mask>) -> VortexResult<Self> {
        vortex_ensure!(
            validity.len() == elements.len(),
            "`PVectorMut` validity mask must have the same length as elements"
        );

        Ok(Self { elements, validity })
    }

    /// Creates a new [`PVectorMut<T>`] from the given elements buffer and validity mask without
    /// validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the validity mask has the same length as the elements buffer.
    ///
    /// Ideally, they are taken from `into_parts`, mutated in a way that doesn't re-allocate, and
    /// then passed back to this function.
    pub unsafe fn new_unchecked(elements: Cow<Buffer<T>>, validity: Cow<Mask>) -> Self {
        if cfg!(debug_assertions) {
            Self::new(elements, validity)
        } else {
            Self { elements, validity }
        }
    }

    /// Create a new mutable primitive vector with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elements: Cow::Mutable(BufferMut::with_capacity(capacity)),
            validity: Cow::Mutable(MaskMut::with_capacity(capacity)),
        }
    }

    /// Decomposes the primitive vector into its constituent parts (buffer and validity).
    pub fn into_parts(self) -> (Cow<Buffer<T>>, Cow<Mask>) {
        (self.elements, self.validity)
    }

    /// Returns the internal [`Cow<Buffer<T>>`] of the [`PVectorMut`].
    ///
    /// Note that the internal buffer may hold garbage data in place of nulls. That information is
    /// tracked by the [`validity()`](Self::validity).
    #[inline]
    pub fn elements(&self) -> &Cow<Buffer<T>> {
        &self.elements
    }

    /// Returns a mutable reference to the `elements` buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that any mutations to the elements do not violate the invariants of
    /// the vector (e.g., the length must remain consistent with the elements buffer).
    #[inline]
    pub unsafe fn elements_mut(&mut self) -> &mut Cow<Buffer<T>> {
        &mut self.elements
    }
}

impl<T: NativePType> VectorMutOps for PVectorMut<T> {
    fn len(&self) -> usize {
        self.elements.len()
    }

    fn validity(&self) -> &Cow<Mask> {
        &self.validity
    }

    unsafe fn validity_mut(&mut self) -> &mut Cow<Mask> {
        &mut self.validity
    }

    fn slice(&self, range: impl RangeBounds<usize> + Clone) -> Self {
        let elements = self.elements.slice(range.clone());
        let validity = self.validity.slice(range);
        Self::new(elements, validity)
    }

    fn capacity(&self) -> usize {
        // self.elements.to_mut().capacity();
        todo!("TODO(connor): It seems weird that we would need to call `into_mut` here");
    }

    fn reserve(&mut self, additional: usize) {
        self.elements.to_mut().reserve(additional);
        self.validity.to_mut().reserve(additional);
    }

    fn clear(&mut self) {
        self.elements.to_mut().clear();
        self.validity.to_mut().clear();
    }

    fn truncate(&mut self, len: usize) {
        self.elements.to_mut().truncate(len);
        self.validity.to_mut().truncate(len);
    }

    /// Extends the vector by appending elements from another vector.
    fn extend_from_vector(&mut self, _other: &Self) {
        // self.elements
        //     .to_mut()
        //     .extend_from_slice(other.elements.as_slice());
        // self.validity
        //     .to_mut()
        //     .append_mask(other.validity().to_frozen()); // <- How do you do this???
        todo!()
    }

    fn append_nulls(&mut self, n: usize) {
        self.elements.to_mut().push_n(T::zero(), n); // Note that the value we push doesn't actually matter.
        self.validity.to_mut().append_n(false, n);
    }

    /// Freeze the vector into an immutable one.
    fn freeze(self) -> Self {
        Self {
            elements: Cow::Frozen(self.elements.freeze()),
            validity: Cow::Frozen(self.validity.freeze()),
        }
    }

    fn split_off(&mut self, at: usize) -> Self {
        Self {
            elements: Cow::Mutable(self.elements.to_mut().split_off(at)),
            validity: Cow::Mutable(self.validity.to_mut().split_off(at)),
        }
    }

    fn unsplit(&mut self, other: Self) {
        if self.is_empty() {
            *self = other;
            return;
        }

        self.elements.to_mut().unsplit(other.elements.into_mut());
        self.validity.to_mut().unsplit(other.validity.into_mut());
    }
}
