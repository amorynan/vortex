// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

//! Definition and implementation of [`DVector<D>`].

use vortex_buffer::{Buffer, BufferMut};
use vortex_dtype::{NativeDecimalType, PrecisionScale};
use vortex_error::{vortex_bail, VortexExpect, VortexResult};
use vortex_mask::{Mask, MaskMut};

use crate::{Cow, VectorOps};

/// A mutable vector of decimal values with fixed precision and scale.
///
/// `D` is bound by [`NativeDecimalType`], which can be one of the native integer types (`i8`,
/// `i16`, `i32`, `i64`, `i128`) or `i256`. `D` is used to store the decimal values.
///
/// The decimal vector maintains a [`PrecisionScale<D>`] that defines the precision (total number of
/// digits) and scale (digits after the decimal point) for all values in the vector.
///
/// Unlike primitive vectors, decimal vectors require validation during construction and
/// modification to ensure values stay within the bounds defined by their precision and scale.
/// This makes operations like "push" fallible, thus we have a [`try_push()`] method instead.
///
/// [`try_push()`]: Self::try_push
#[derive(Debug, Clone)]
pub struct DVector<D> {
    /// The precision and scale of each decimal in the decimal vector.
    pub(super) ps: PrecisionScale<D>,
    /// The buffer representing the vector decimal elements (can be frozen or mutable).
    pub(super) elements: Cow<Buffer<D>>,
    /// The validity mask (where `true` represents an element is **not** null).
    pub(super) validity: Cow<Mask>,
}

impl<D: NativeDecimalType> DVector<D> {
    /// Creates a new [`DVector<D>`] from the given [`PrecisionScale`], elements buffer, and
    /// validity mask.
    ///
    /// # Panics
    ///
    /// Panics if:
    ///
    /// - The lengths of the `elements` and `validity` do not match.
    /// - Any of the elements are out of bounds for the given [`PrecisionScale`].
    pub fn new(ps: PrecisionScale<D>, elements: Cow<Buffer<D>>, validity: Cow<Mask>) -> Self {
        Self::try_new(ps, elements, validity).vortex_expect("Failed to create `DVector`")
    }

    /// Tries to create a new [`DVector<D>`] from the given [`PrecisionScale`], elements buffer,
    /// and validity mask.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - The lengths of the `elements` and `validity` do not match.
    /// - Any of the elements are out of bounds for the given [`PrecisionScale`].
    pub fn try_new(
        ps: PrecisionScale<D>,
        elements: Cow<Buffer<D>>,
        validity: Cow<Mask>,
    ) -> VortexResult<Self> {
        if elements.len() != validity.len() {
            vortex_bail!(
                "Elements length {} does not match validity length {}",
                elements.len(),
                validity.len()
            );
        }

        // We assert that each element is within bounds for the given precision/scale.
        if !elements.as_ref().iter().all(|e| ps.is_valid(*e)) {
            vortex_bail!(
                "One or more elements are out of bounds for precision {} and scale {}",
                ps.precision(),
                ps.scale()
            );
        }

        Ok(Self {
            ps,
            elements,
            validity,
        })
    }

    /// Creates a new [`DVector<D>`] from the given [`PrecisionScale`], elements buffer, and
    /// validity mask, _without_ validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    ///
    /// - The lengths of the elements and validity are equal.
    /// - All elements are in bounds for the given [`PrecisionScale`].
    pub unsafe fn new_unchecked(
        ps: PrecisionScale<D>,
        elements: Cow<Buffer<D>>,
        validity: Cow<Mask>,
    ) -> Self {
        if cfg!(debug_assertions) {
            Self::try_new(ps, elements, validity).vortex_expect("Failed to create `DVectorMut`")
        } else {
            Self {
                ps,
                elements,
                validity,
            }
        }
    }

    /// Create a new mutable primitive vector with the given capacity.
    pub fn with_capacity(ps: PrecisionScale<D>, capacity: usize) -> Self {
        Self {
            ps,
            elements: Cow::Mutable(BufferMut::with_capacity(capacity)),
            validity: Cow::Mutable(MaskMut::with_capacity(capacity)),
        }
    }

    /// Decomposes the decimal vector into its constituent parts ([`PrecisionScale`], decimal
    /// buffer, and validity).
    pub fn into_parts(self) -> (PrecisionScale<D>, Cow<Buffer<D>>, Cow<Mask>) {
        (self.ps, self.elements, self.validity)
    }

    /// Get the precision/scale of the decimal vector.
    pub fn precision_scale(&self) -> PrecisionScale<D> {
        self.ps
    }

    /// Returns a reference to the underlying elements buffer containing the decimal data.
    pub fn elements(&self) -> &Cow<Buffer<D>> {
        &self.elements
    }

    /// Returns a mutable reference to the underlying elements buffer containing the decimal data.
    ///
    /// # Safety
    ///
    /// Modifying the elements buffer directly may violate the precision/scale constraints.
    /// The caller must ensure that any modifications maintain these invariants.
    pub unsafe fn elements_mut(&mut self) -> &mut Cow<Buffer<D>> {
        &mut self.elements
    }

    /// Gets a nullable element at the given index, panicking on out-of-bounds.
    ///
    /// If the element at the given index is null, returns `None`. Otherwise, returns `Some(x)`,
    /// where `x: D`.
    ///
    /// Note that this `get` method is different from the standard library [`slice::get`], which
    /// returns `None` if the index is out of bounds. This method will panic if the index is out of
    /// bounds, and return `None` if the elements is null.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get(&self, index: usize) -> Option<&D> {
        self.validity
            .value(index)
            .then(|| &self.elements.as_ref()[index])
    }

    /// Appends a new element to the end of the vector.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is out of bounds for the vector's precision/scale.
    pub fn try_push(&mut self, value: D) -> VortexResult<()> {
        self.try_append_n(value, 1)
    }

    /// Appends n elements to the vector, all set to the given value.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is out of bounds for the vector's precision/scale.
    pub fn try_append_n(&mut self, value: D, n: usize) -> VortexResult<()> {
        if !self.ps.is_valid(value) {
            vortex_bail!("Value {:?} is out of bounds for {}", value, self.ps);
        }

        self.elements.ensure_mut().push_n(value, n);
        self.validity.ensure_mut().append_n(true, n);
        Ok(())
    }
}

impl<D: NativeDecimalType> AsRef<[D]> for DVector<D> {
    fn as_ref(&self) -> &[D] {
        self.elements.as_ref()
    }
}

impl<D: NativeDecimalType> VectorOps for DVector<D> {
    fn len(&self) -> usize {
        self.elements.len()
    }

    fn validity(&self) -> &Cow<Mask> {
        &self.validity
    }

    unsafe fn validity_mut(&mut self) -> &mut Cow<Mask> {
        &mut self.validity
    }

    fn clear(&mut self) {
        self.elements.ensure_mut().clear();
        self.validity.ensure_mut().clear();
    }

    fn truncate(&mut self, len: usize) {
        self.elements.ensure_mut().truncate(len);
        self.validity.ensure_mut().truncate(len);
    }

    fn append_zeros(&mut self, n: usize) {
        self.elements.ensure_mut().push_n(D::default(), n);
        self.validity.ensure_mut().append_n(true, n);
    }

    fn append_nulls(&mut self, n: usize) {
        self.elements.ensure_mut().push_n(D::default(), n);
        self.validity.ensure_mut().append_n(false, n);
    }

    fn ensure_frozen(&mut self) {
        self.elements.ensure_frozen();
        self.validity.ensure_frozen();
    }

    fn split_off(&mut self, at: usize) -> Self {
        DVector {
            ps: self.ps,
            elements: Cow::Mutable(self.elements.ensure_mut().split_off(at)),
            validity: Cow::Mutable(self.validity.ensure_mut().split_off(at)),
        }
    }

    fn unsplit(&mut self, other: Self) {
        if self.is_empty() {
            *self = other;
            return;
        }
        self.elements
            .ensure_mut()
            .unsplit(other.elements.into_mut());
        self.validity
            .ensure_mut()
            .unsplit(other.validity.into_mut());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction_and_validation() {
        // Test try_new with valid data.
        let ps = PrecisionScale::<i32>::new(9, 2);
        let elements = Buffer::from_iter([100_i32, 200, 300]);
        let validity = Mask::new_true(3);
        let vec = DVector::try_new(ps, elements.into(), validity.into()).unwrap();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.precision_scale().precision(), 9);
        assert_eq!(vec.precision_scale().scale(), 2);

        // Test try_new error handling - length mismatch.
        let elements_bad = Buffer::from_iter([100_i32, 200]);
        let validity_bad = Mask::new_true(3);
        let result = DVector::try_new(ps, elements_bad.into(), validity_bad.into());
        assert!(result.is_err());

        // Test try_new error handling - out of bounds values.
        let too_large = 10_i32.pow(9); // 10^9 exceeds precision 9.
        let elements_oob = Buffer::from_iter([100_i32, too_large, 300]);
        let validity_oob = Mask::new_true(3);
        let result = DVector::try_new(ps, elements_oob.into(), validity_oob.into());
        assert!(result.is_err());

        // Test new_unchecked.
        let elements_unchecked = Buffer::from_iter([100_i32, 200]);
        let validity_unchecked = Mask::new_true(2);
        let vec_unchecked = unsafe {
            DVector::new_unchecked(ps, elements_unchecked.into(), validity_unchecked.into())
        };
        assert_eq!(vec_unchecked.len(), 2);
    }

    #[test]
    fn test_push_append_and_access() {
        let ps = PrecisionScale::<i32>::new(9, 2);
        let mut vec = DVector::<i32>::with_capacity(ps, 10);

        // Test try_push with valid values.
        vec.try_push(12345).unwrap(); // 123.45.
        vec.try_push(9999).unwrap(); // 99.99.
        vec.try_push(-5000).unwrap(); // -50.00.
        assert_eq!(vec.len(), 3);

        // Test try_push with out-of-bounds values.
        let too_large = 10_i32.pow(9);
        assert!(vec.try_push(too_large).is_err());
        assert_eq!(vec.len(), 3); // Length unchanged after failed push.

        // Test get without nulls.
        assert_eq!(vec.get(0), Some(&12345));
        assert_eq!(vec.get(1), Some(&9999));
        assert_eq!(vec.get(2), Some(&-5000));

        // Test append_nulls.
        vec.append_nulls(2);
        assert_eq!(vec.len(), 5);
        assert_eq!(vec.get(3), None);
        assert_eq!(vec.get(4), None);

        // Test AsRef<[D]> slice access.
        let slice = vec.as_ref();
        assert_eq!(slice.len(), 5);
        assert_eq!(slice[0], 12345);
        assert_eq!(slice[1], 9999);
        assert_eq!(slice[2], -5000);
        // Note: slice[3] and slice[4] are default values (0) but marked as null in validity.
    }

    #[test]
    fn test_precision_scale_combinations() {
        // Test Decimal(9, 2) - common currency format.
        let ps_9_2 = PrecisionScale::<i32>::new(9, 2);
        let mut vec_9_2 = DVector::<i32>::with_capacity(ps_9_2, 5);
        vec_9_2.try_push(999999999).unwrap(); // Max: 9999999.99 stored as 999999999.
        assert!(vec_9_2.try_push(1000000000).is_err()); // 10000000.00 stored as 1000000000 exceeds precision.
        assert!(vec_9_2.try_push(-999999999).is_ok()); // Negative within bounds.
        assert_eq!(vec_9_2.len(), 2);

        // Test Decimal(38, 10) - high precision scientific.
        let ps_38_10 = PrecisionScale::<i128>::new(38, 10);
        let mut vec_38_10 = DVector::<i128>::with_capacity(ps_38_10, 3);
        let large_value = 10_i128.pow(28) - 1; // 10^28 - 1, well within 38 digits.
        vec_38_10.try_push(large_value).unwrap();
        assert_eq!(vec_38_10.len(), 1);

        // Test Decimal(4, 0) - integer-only decimals that fit in i16.
        let ps_4_0 = PrecisionScale::<i16>::new(4, 0);
        let mut vec_4_0 = DVector::<i16>::with_capacity(ps_4_0, 5);
        vec_4_0.try_push(9999).unwrap(); // Max: 9999.
        assert!(vec_4_0.try_push(10000).is_err()); // Exceeds 4 digits.
        vec_4_0.try_push(-9999).unwrap(); // Negative within bounds.
        assert_eq!(vec_4_0.len(), 2);

        // Test with different underlying types.
        // i8 with small precision/scale (max precision for i8 is 2).
        let ps_2_1 = PrecisionScale::<i8>::new(2, 1);
        let mut vec_i8 = DVector::<i8>::with_capacity(ps_2_1, 3);
        vec_i8.try_push(99).unwrap(); // 9.9.
        assert!(vec_i8.try_push(100).is_err()); // 10.0 exceeds precision.

        // i16 with moderate precision/scale (max precision for i16 is 4).
        let ps_4_2 = PrecisionScale::<i16>::new(4, 2);
        let mut vec_i16 = DVector::<i16>::with_capacity(ps_4_2, 3);
        vec_i16.try_push(999).unwrap(); // 9.99.
        vec_i16.try_push(9999).unwrap(); // 99.99.
        assert_eq!(vec_i16.len(), 2);
    }

    #[test]
    fn test_empty_and_edge_cases() {
        let ps = PrecisionScale::<i32>::new(9, 2);

        // Test empty vector creation and operations.
        let empty_vec = DVector::<i32>::with_capacity(ps, 0);
        assert_eq!(empty_vec.len(), 0);

        // Test single element vector.
        let mut single = DVector::<i32>::with_capacity(ps, 1);
        single.try_push(42).unwrap();
        assert_eq!(single.len(), 1);
        assert_eq!(single.get(0), Some(&42));

        // Split single element vector at index 1.
        // Original keeps [0, 1) = the element, split gets [1, len) = nothing.
        let split_single = single.split_off(1);
        assert_eq!(single.len(), 1); // Original keeps the element.
        assert_eq!(split_single.len(), 0); // Split gets nothing.

        // Test all-null vector.
        let mut all_nulls = DVector::<i32>::with_capacity(ps, 5);
        all_nulls.append_nulls(5);
        assert_eq!(all_nulls.len(), 5);
        for i in 0..5 {
            assert_eq!(all_nulls.get(i), None);
        }
    }

    #[test]
    fn test_nulls_with_validity_mask() {
        let ps = PrecisionScale::<i32>::new(8, 3);

        // Create vector with specific null pattern using validity mask.
        let elements = Buffer::from_iter([1000_i32, 0, 2000, 0, 3000]); // 0s will be null.
        let validity = Mask::from_iter([true, false, true, false, true]);
        let mut vec = DVector::new(ps, elements.into(), validity.into());

        assert_eq!(vec.len(), 5);
        assert_eq!(vec.get(0), Some(&1000)); // 1.000.
        assert_eq!(vec.get(1), None); // Null.
        assert_eq!(vec.get(2), Some(&2000)); // 2.000.
        assert_eq!(vec.get(3), None); // Null.
        assert_eq!(vec.get(4), Some(&3000)); // 3.000.

        // Extend with more values and nulls.
        vec.try_push(4000).unwrap();
        vec.append_nulls(2);
        assert_eq!(vec.len(), 8);
        assert_eq!(vec.get(5), Some(&4000));
        assert_eq!(vec.get(6), None);
        assert_eq!(vec.get(7), None);

        // Split and verify nulls are preserved.
        let split = vec.split_off(4);
        assert_eq!(vec.len(), 4);
        assert_eq!(split.len(), 4);

        // Original vec should have: valid, null, valid, null.
        assert_eq!(vec.get(1), None);
        assert_eq!(vec.get(3), None);

        // Split should have: valid, valid, null, null.
        assert_eq!(split.get(0), Some(&3000));
        assert_eq!(split.get(1), Some(&4000));
        assert_eq!(split.get(2), None);
        assert_eq!(split.get(3), None);
    }
}
