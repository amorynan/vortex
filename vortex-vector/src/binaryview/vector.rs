// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

//! Mutable variable-length binary vector.

use vortex_buffer::{Alignment, Buffer, BufferMut, ByteBuffer, ByteBufferMut};
use vortex_error::{vortex_ensure, VortexExpect, VortexResult};
use vortex_mask::{Mask, MaskMut};

use crate::binaryview::view::{validate_views, BinaryView};
use crate::binaryview::BinaryViewType;
use crate::{Cow, VectorOps};

// Default capacity for new string data buffers of 2MiB.
const BUFFER_CAPACITY: usize = 2 * 1024 * 1024;

/// A mutable vector of binary view data.
///
/// The immutable equivalent of this type is [`BinaryViewVector`].
#[derive(Clone, Debug)]
pub struct BinaryViewVector<T: BinaryViewType> {
    /// Views into the binary data.
    views: Cow<Buffer<BinaryView>>,
    /// Validity mask for the vector.
    validity: Cow<Mask>,

    /// The completed buffers holding referenced binary data.
    buffers: Vec<ByteBuffer>,
    /// The current buffer being appended to, if any.
    open_buffer: Option<ByteBufferMut>,

    /// Marker trait for the [`BinaryViewType`].
    _marker: std::marker::PhantomData<T>,
}

impl<T: BinaryViewType> BinaryViewVector<T> {
    /// Create a new [`BinaryViewVector`] from its components, panicking if validation fails.
    ///
    /// # Errors
    ///
    /// This function will panic if any of the validation checks performed by [`try_new`][Self::try_new]
    /// fails.
    pub fn new(
        views: Cow<Buffer<BinaryView>>,
        buffers: Vec<ByteBuffer>,
        validity: Cow<Mask>,
    ) -> Self {
        Self::try_new(views, buffers, validity)
            .vortex_expect("Failed to create `BinaryViewVectorMut`")
    }

    /// Create a new empty [`BinaryViewVector`], pre-allocated to hold the specified number
    /// of items. This does not reserve any memory for string data itself, only for the binary views
    /// and the validity bits.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::new(
            Cow::Mutable(BufferMut::with_capacity(capacity)),
            Vec::new(),
            Cow::Mutable(MaskMut::with_capacity(capacity)),
        )
    }

    /// Tries to create a new [`BinaryViewVector`] from its components.
    ///
    /// # Errors
    ///
    /// Returns an error if the length of the validity mask does not match the length of the views.
    ///
    /// Returns an error if the views reference any data that is not a valid buffer
    pub fn try_new(
        views: Cow<Buffer<BinaryView>>,
        buffers: Vec<ByteBuffer>,
        validity: Cow<Mask>,
    ) -> VortexResult<Self> {
        vortex_ensure!(
            views.len() == validity.len(),
            "views buffer length {} != validity length {}",
            views.len(),
            validity.len()
        );

        validate_views(
            views.as_ref(),
            buffers.as_ref(),
            |index| validity.value(index),
            T::validate,
        )?;

        Ok(Self {
            views,
            buffers,
            validity,
            open_buffer: None,
            _marker: std::marker::PhantomData,
        })
    }

    /// Creates a new [`BinaryViewVector`] from the given bits and validity mask without validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the validity mask has the same length as the views.
    pub unsafe fn new_unchecked(
        views: Cow<Buffer<BinaryView>>,
        validity: Cow<Mask>,
        buffers: Vec<ByteBuffer>,
    ) -> Self {
        if cfg!(debug_assertions) {
            Self::new(views, buffers, validity)
        } else {
            Self {
                views,
                buffers,
                validity,
                open_buffer: None,
                _marker: std::marker::PhantomData,
            }
        }
    }

    /// Get a handle to the buffer holding the [views][BinaryView] of the vector.
    pub fn views(&self) -> &Cow<Buffer<BinaryView>> {
        &self.views
    }

    /// Get a mutable handle to the buffer holding the [views][BinaryView] of the vector.
    ///
    /// # Safety
    ///
    /// Caller must make sure that length of the views always matches
    /// length of the validity mask.
    pub unsafe fn views_mut(&mut self) -> &mut Cow<Buffer<BinaryView>> {
        &mut self.views
    }

    /// Get a handle to the vector of buffers backing the string data of the vector.
    pub fn buffers(&self) -> &[ByteBuffer] {
        &self.buffers
    }

    /// Get a mutable handle to the vector of buffers backing the string data of the vector.
    ///
    /// # Safety
    ///
    /// Caller must ensure that no existing views into the buffers are invalidated.
    pub unsafe fn buffers_mut(&mut self) -> &mut Vec<ByteBuffer> {
        &mut self.buffers
    }

    /// Get the `index` item from the vector as an owned `Scalar` type with zero-copy.
    ///
    /// This function will panic is `index` is out of range for the vector's length.
    pub fn get(&self, index: usize) -> Option<T::Scalar> {
        if !self.validity.value(index) {
            return None;
        }

        let view = &self.views[index];
        if view.is_inlined() {
            let view = view.as_inlined();

            // For frozen views, we can return a slice zero-copy.
            // For mutable views, we need to copy the data into a new buffer.
            let buffer = match &self.views {
                Cow::Frozen(frozen) => frozen
                    .clone()
                    .into_byte_buffer()
                    .slice_ref_with_alignment(&view.data[..view.size as usize], Alignment::none()),
                Cow::Mutable(_) => {
                    let mut buffer = ByteBufferMut::with_capacity(view.size as usize);
                    buffer.extend_from_slice(&view.data[..view.size as usize]);
                    buffer.freeze()
                }
            };

            // SAFETY: validation that the string data contained in this vector is performed
            //  at construction time, either in the constructor for safe construction, or by
            //  the caller (when using the unchecked constructor).
            Some(unsafe { T::scalar_from_buffer_unchecked(buffer) })
        } else {
            // Get a pointer into the buffer range
            let view_ref = view.as_view();
            let buffer = &self.buffers[view_ref.buffer_index as usize];

            let start = view_ref.offset as usize;
            let length = view_ref.size as usize;
            let buffer_slice = buffer.slice(start..start + length);

            // SAFETY: validation that the string data contained in this vector is performed
            //  at construction time, either in the constructor for safe construction, or by
            //  the caller (when using the unchecked constructor).
            Some(unsafe { T::scalar_from_buffer_unchecked(buffer_slice) })
        }
    }

    /// Get the `index` item from the vector as a native `Slice` type.
    ///
    /// This function will panic is `index` is out of range for the vector's length.
    pub fn get_ref(&self, index: usize) -> Option<&T::Slice> {
        if !self.validity.value(index) {
            return None;
        }

        let view = &self.views[index];
        if view.is_inlined() {
            let view = view.as_inlined();
            // SAFETY: validation that the string data contained in this vector is performed
            //  at construction time, either in the constructor for safe construction, or by
            //  the caller (when using the unchecked constructor).
            Some(unsafe { T::from_bytes_unchecked(&view.data[..view.size as usize]) })
        } else {
            // Get a pointer into the buffer range
            let view_ref = view.as_view();
            let buffer = &self.buffers[view_ref.buffer_index as usize];

            let start = view_ref.offset as usize;
            let length = view_ref.size as usize;

            // SAFETY: validation that the string data contained in this vector is performed
            //  at construction time, either in the constructor for safe construction, or by
            //  the caller (when using the unchecked constructor).
            Some(unsafe { T::from_bytes_unchecked(&buffer.as_bytes()[start..start + length]) })
        }
    }

    /// Append a repeated sequence of binary data to a vector.
    ///
    /// ```
    /// # use vortex_vector::binaryview::StringVector;
    /// # use vortex_vector::VectorOps;
    /// let mut strings = StringVector::with_capacity(4);
    /// strings.append_values("inlined", 2);
    /// strings.append_nulls(1);
    /// strings.append_values("large not inlined", 1);
    ///
    /// let strings = strings.freeze();
    ///
    /// assert_eq!(
    ///     [strings.get_ref(0), strings.get_ref(1), strings.get_ref(2), strings.get_ref(3)],
    ///     [Some("inlined"), Some("inlined"), None, Some("large not inlined")],
    /// );
    /// ```
    pub fn append_values(&mut self, value: &T::Slice, n: usize) {
        let views = self.views.ensure_mut();

        let bytes = value.as_ref();
        if bytes.len() <= BinaryView::MAX_INLINED_SIZE {
            views.push_n(BinaryView::new_inlined(bytes), n);
        } else {
            let buffer_index =
                u32::try_from(self.buffers.len()).vortex_expect("buffer count exceeds u32::MAX");

            let buf = self
                .open_buffer
                .get_or_insert_with(|| ByteBufferMut::with_capacity(BUFFER_CAPACITY));
            let offset = u32::try_from(buf.len()).vortex_expect("buffer length exceeds u32::MAX");
            buf.extend_from_slice(value.as_ref());

            views.push_n(BinaryView::make_view(bytes, buffer_index, offset), n);
        }

        self.validity.ensure_mut().append_n(true, n);
    }

    /// Append a repeated sequence of binary data to a vector, from an owned buffer.
    ///
    /// The buffer will be used directly if possible, avoiding a copy.
    pub fn append_owned_values(&mut self, value: T::Scalar, n: usize) {
        let buffer: ByteBuffer = value.into();

        if buffer.len() <= BinaryView::MAX_INLINED_SIZE {
            self.views
                .ensure_mut()
                .push_n(BinaryView::new_inlined(buffer.as_ref()), n);
        } else {
            self.flush_open_buffer();
            let buffer_index = u32::try_from(self.buffers.len())
                .vortex_expect("buffer count exceeds u32::MAX")
                + 1;
            self.views
                .ensure_mut()
                .push_n(BinaryView::make_view(buffer.as_ref(), buffer_index, 0), n);
            self.buffers.push(buffer);
        }

        self.validity.ensure_mut().append_n(true, n);
    }

    fn flush_open_buffer(&mut self) {
        if let Some(open) = self.open_buffer.take() {
            self.buffers.push(open.freeze());
        }
    }
}

impl<T: BinaryViewType> VectorOps for BinaryViewVector<T> {
    fn len(&self) -> usize {
        self.views.len()
    }

    fn validity(&self) -> &Cow<Mask> {
        &self.validity
    }

    unsafe fn validity_mut(&mut self) -> &mut Cow<Mask> {
        &mut self.validity
    }

    fn clear(&mut self) {
        self.views.clear();
        self.validity.clear();
        self.buffers.clear();
        self.open_buffer = None;
    }

    fn truncate(&mut self, len: usize) {
        self.views.truncate(len);
        self.validity.truncate(len);
    }

    fn append_zeros(&mut self, n: usize) {
        self.views.ensure_mut().push_n(BinaryView::empty_view(), n);
        self.validity.ensure_mut().append_n(true, n);
    }

    fn append_nulls(&mut self, n: usize) {
        self.views.ensure_mut().push_n(BinaryView::empty_view(), n);
        self.validity.ensure_mut().append_n(false, n);
    }

    fn ensure_frozen(&mut self) {
        self.views.ensure_frozen();
        self.validity.ensure_frozen();
    }

    fn split_off(&mut self, _at: usize) -> Self {
        todo!()
    }

    fn unsplit(&mut self, other: Self) {
        if self.is_empty() {
            *self = other;
            return;
        }

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::binaryview::{BinaryView, StringVector};
    use vortex_buffer::{buffer, ByteBuffer};
    use vortex_mask::Mask;

    #[test]
    #[should_panic(expected = "views buffer length 1 != validity length 100")]
    fn test_try_new_mismatch_validity_len() {
        StringVector::try_new(
            buffer![BinaryView::new_inlined(b"inlined")].into(),
            vec![],
            Mask::new_true(100).into(),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "view at index 0 references invalid buffer: 100 out of bounds for BinaryViewVector with 0 buffers"
    )]
    fn test_try_new_invalid_buffer_offset() {
        StringVector::try_new(
            buffer![BinaryView::make_view(b"bad buffer ptr", 100, 0)].into(),
            vec![],
            Mask::new_true(1).into(),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "start offset 4294967295 out of bounds for buffer 0 with size 19")]
    fn test_try_new_invalid_length() {
        StringVector::try_new(
            buffer![BinaryView::make_view(b"bad buffer ptr", 0, u32::MAX)].into(),
            vec![ByteBuffer::copy_from(b"a very short buffer")],
            Mask::new_true(1).into(),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "view at index 0: inlined bytes failed utf-8 validation")]
    fn test_try_new_invalid_utf8_inlined() {
        StringVector::try_new(
            buffer![BinaryView::new_inlined(b"\x80")].into(),
            vec![],
            Mask::new_true(1).into(),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "view at index 0: outlined bytes failed utf-8 validation")]
    fn test_try_new_invalid_utf8_outlined() {
        // 0xFF is never valid in UTF-8
        let sequence = b"\xff".repeat(13);
        StringVector::try_new(
            buffer![BinaryView::make_view(&sequence, 0, 0)].into(),
            vec![ByteBuffer::copy_from(sequence)],
            Mask::new_true(1).into(),
        )
        .unwrap();
    }

    #[test]
    fn test_basic() {
        let strings = StringVector::new(
            buffer![
                BinaryView::new_inlined(b"inlined1"),
                BinaryView::make_view(b"long string 1", 0, 0),
                BinaryView::new_inlined(b"inlined2"),
                BinaryView::make_view(b"long string 2", 0, 13),
                BinaryView::new_inlined(b"inlined3"),
                BinaryView::make_view(b"long string 3", 0, 26),
            ]
            .into(),
            vec![ByteBuffer::copy_from(
                "long string 1long string 2long string 3",
            )],
            Mask::new_true(6).into(),
        );

        assert_eq!(strings.get_ref(0), Some("inlined1"));
        assert_eq!(strings.get_ref(1), Some("long string 1"));
        assert_eq!(strings.get_ref(2), Some("inlined2"));
        assert_eq!(strings.get_ref(3), Some("long string 2"));
        assert_eq!(strings.get_ref(4), Some("inlined3"));
        assert_eq!(strings.get_ref(5), Some("long string 3"));
    }
}
