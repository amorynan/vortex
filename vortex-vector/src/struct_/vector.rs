// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

//! Definition and implementation of [`StructVector`].

// use std::sync::Arc;

use vortex_dtype::StructFields;
use vortex_error::{vortex_ensure, VortexExpect, VortexResult};
use vortex_mask::{Mask, MaskMut};

use crate::{match_vector_pair, Cow, Vector, VectorOps};

/// A mutable vector of struct values (values with named fields).
///
/// Struct values are stored column-wise in the vector, so values in the same field are stored next
/// to each other (rather than values in the same struct stored next to each other).
#[derive(Debug)]
pub struct StructVector {
    /// The (owned) fields of the `StructVectorMut`, each stored column-wise as a [`Vector`].
    pub(super) fields: Box<[Vector]>,

    /// The validity mask (where `true` represents an element is **not** null).
    pub(super) validity: Cow<Mask>,

    /// The length of the vector (which is the same as all field vectors).
    ///
    /// This is stored here as a convenience, and also helps in the case that the `StructVector` has
    /// no fields.
    pub(super) len: usize,
}

impl StructVector {
    /// Creates a new [`StructVector`] with the given fields and validity mask.
    ///
    /// # Panics
    ///
    /// Panics if:
    ///
    /// - Any field vector has a length that does not match the length of other fields.
    /// - The validity mask length does not match the field length.
    pub fn new(fields: Box<[Vector]>, validity: Cow<Mask>) -> Self {
        Self::try_new(fields, validity).vortex_expect("Failed to create `StructVectorMut`")
    }

    /// Tries to create a new [`StructVector`] with the given fields and validity mask.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Any field vector has a length that does not match the length of other fields.
    /// - The validity mask length does not match the field length.
    pub fn try_new(fields: Box<[Vector]>, validity: Cow<Mask>) -> VortexResult<Self> {
        let len = validity.len();

        // Validate that all fields have the correct length.
        for (i, field) in fields.iter().enumerate() {
            vortex_ensure!(
                field.len() == len,
                "Field {} has length {} but expected length {}",
                i,
                field.len(),
                len
            );
        }

        Ok(Self {
            fields,
            validity,
            len,
        })
    }

    /// Creates a new [`StructVector`] with the given fields and validity mask without
    /// validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// - All field vectors have the same length.
    /// - The validity mask has a length equal to the field length.
    pub unsafe fn new_unchecked(fields: Box<[Vector]>, validity: Cow<Mask>) -> Self {
        let len = validity.len();

        if cfg!(debug_assertions) {
            Self::new(fields, validity)
        } else {
            Self {
                fields,
                validity,
                len,
            }
        }
    }

    /// Creates a new [`StructVector`] with the given fields and capacity.
    pub fn with_capacity(struct_fields: &StructFields, capacity: usize) -> Self {
        let fields: Vec<Vector> = struct_fields
            .fields()
            .map(|dtype| Vector::with_capacity(&dtype, capacity))
            .collect();

        let validity = Cow::Mutable(MaskMut::with_capacity(capacity));

        Self {
            fields: fields.into_boxed_slice(),
            validity,
            len: 0,
        }
    }

    /// Decomposes the struct vector into its constituent parts (fields, validity, and length).
    pub fn into_parts(self) -> (Box<[Vector]>, Cow<Mask>, usize) {
        (self.fields, self.validity, self.len)
    }

    /// Returns the fields of the `StructVectorMut`, each stored column-wise as a [`Vector`].
    pub fn fields(&self) -> &[Vector] {
        self.fields.as_ref()
    }

    /// Returns a mutable handle to the field vectors.
    ///
    /// # Safety
    ///
    /// Callers must ensure that any modifications to the field vectors do not violate
    /// the invariants of this type, namely that all field vectors are of the same length
    /// and equal to the length of the validity.
    pub unsafe fn fields_mut(&mut self) -> &mut [Vector] {
        self.fields.as_mut()
    }

    /// Returns a mutable handle to the validity mask of the vector.
    ///
    /// # Safety
    ///
    /// Callers must ensure that if the length of the mask is modified, the lengths
    /// of all of the field vectors should be updated accordingly to continue meeting
    /// the invariants of the type.
    pub unsafe fn validity_mut(&mut self) -> &mut Cow<Mask> {
        &mut self.validity
    }

    /// Finds the minimum capacity of all field vectors.
    ///
    /// This is equal to the maximum amount of scalars we can add before we need to reallocate at
    /// least one of the child field vectors.
    ///
    /// If there are no fields, this returns the length of the vector.
    ///
    /// Note that this takes time in `O(f)`, where `f` is the number of fields.
    pub fn minimum_capacity(&self) -> usize {
        // self.fields
        //     .iter()
        //     .map(|field| field.capacity())
        //     .min()
        //     .unwrap_or(self.len)
        todo!()
    }
}

impl VectorOps for StructVector {
    fn len(&self) -> usize {
        self.len
    }

    fn validity(&self) -> &Cow<Mask> {
        &self.validity
    }

    unsafe fn validity_mut(&mut self) -> &mut Cow<Mask> {
        unsafe { &mut self.validity }
    }

    fn clear(&mut self) {
        for field in &mut self.fields {
            field.clear();
        }
        self.validity.clear();
        self.len = 0;
    }

    fn truncate(&mut self, len: usize) {
        for field in &mut self.fields {
            field.truncate(len);
        }

        // self.validity.truncate(len);
        self.len = self.validity.len();
    }

    fn append_zeros(&mut self, n: usize) {
        for field in &mut self.fields {
            field.append_zeros(n);
        }
        self.validity.ensure_mut().append_n(true, n);
        self.len += n;
    }

    fn append_nulls(&mut self, n: usize) {
        for field in &mut self.fields {
            field.append_zeros(n);
        }
        self.validity.ensure_mut().append_n(false, n);
        self.len += n;
    }

    fn ensure_frozen(&mut self) {
        for field in &mut self.fields {
            field.ensure_frozen();
        }
        self.validity.ensure_frozen();
    }

    fn split_off(&mut self, at: usize) -> Self {
        let split_fields: Vec<Vector> = self
            .fields
            .iter_mut()
            .map(|field| field.split_off(at))
            .collect();

        let tail_validity = self.validity.ensure_mut().split_off(at);
        let tail_len = self.len.saturating_sub(at);
        self.len = self.len.min(at);
        debug_assert_eq!(self.len, self.validity.len());

        Self {
            fields: split_fields.into_boxed_slice(),
            len: tail_len,
            validity: Cow::Mutable(tail_validity),
        }
    }

    fn unsplit(&mut self, other: Self) {
        assert_eq!(
            self.fields.len(),
            other.fields.len(),
            "Cannot unsplit StructVectorMut: field count mismatch ({} vs {})",
            self.fields.len(),
            other.fields.len()
        );

        if self.is_empty() {
            *self = other;
            return;
        }

        // Unsplit each field vector.
        let pairs = self.fields.iter_mut().zip(other.fields);
        for (self_mut_vector, other_mut_vec) in pairs {
            match_vector_pair!(self_mut_vector, other_mut_vec, |a, b| a.unsplit(b))
        }

        self.validity
            .ensure_mut()
            .unsplit(other.validity.into_mut());
        self.len += other.len;
        debug_assert_eq!(self.len, self.validity.len());
    }
}

#[cfg(test)]
mod tests {
    use vortex_mask::Mask;

    use super::*;
    use crate::bool::BoolVector;
    use crate::null::NullVector;
    use crate::primitive::PVector;
    use crate::Vector;

    #[test]
    fn test_empty_fields() {
        let mut struct_vec =
            StructVector::try_new(Box::new([]), Mask::new_true(10).into()).unwrap();
        let second_half = struct_vec.split_off(6);
        assert_eq!(struct_vec.len(), 6);
        assert_eq!(second_half.len(), 4);
    }

    #[test]
    fn test_nested_struct() {
        let inner1 = StructVector::try_new(
            Box::new([
                NullVector::new(4).into(),
                BoolVector::from_iter([true, false, true, false]).into(),
            ]),
            Mask::new_true(4).into(),
        )
        .unwrap()
        .into();

        let inner2 = StructVector::try_new(
            Box::new([PVector::<u32>::from_iter([100, 200, 300, 400]).into()]),
            Mask::new_true(4).into(),
        )
        .unwrap()
        .into();

        let mut outer =
            StructVector::try_new(Box::new([inner1, inner2]), Mask::new_true(4).into()).unwrap();

        let second = outer.split_off(2);
        assert_eq!(outer.len(), 2);
        assert_eq!(second.len(), 2);

        outer.unsplit(second);
        assert_eq!(outer.len(), 4);
        assert!(matches!(outer.fields[0], Vector::Struct(_)));
    }
}
