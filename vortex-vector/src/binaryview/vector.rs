// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

//! Variable-length binary vector implementation.

use std::fmt::Debug;
use std::ops::RangeBounds;
use std::sync::Arc;

use vortex_buffer::{Alignment, Buffer, ByteBuffer};
use vortex_error::{vortex_ensure, VortexExpect, VortexResult};
use vortex_mask::Mask;

use crate::binaryview::vector_mut::BinaryViewVectorMut;
use crate::binaryview::view::{validate_views, BinaryView};
use crate::binaryview::{BinaryViewScalar, BinaryViewType};
use crate::{Scalar, VectorOps};

/// A variable-length binary vector.
///
/// This is the core vector for string and binary data.
#[derive(Debug, Clone)]
pub struct BinaryViewVector<T: BinaryViewType> {
    /// Views into the binary data.
    views: Buffer<BinaryView>,
    /// Buffers holding the referenced binary data.
    buffers: Arc<Box<[ByteBuffer]>>,
    /// Validity mask for the vector.
    validity: Mask,
    /// Marker trait for the [`BinaryViewType`].
    _marker: std::marker::PhantomData<T>,
}

impl<T: BinaryViewType> BinaryViewVector<T> {
    /// Buffers
    pub fn buffers(&self) -> &Arc<Box<[ByteBuffer]>> {
        &self.buffers
    }

    /// Views
    pub fn views(&self) -> &Buffer<BinaryView> {
        &self.views
    }
}

impl<T: BinaryViewType> VectorOps for BinaryViewVector<T> {
    type Mutable = BinaryViewVectorMut<T>;

    fn len(&self) -> usize {
        self.views.len()
    }

    fn validity(&self) -> &Mask {
        &self.validity
    }

    fn scalar_at(&self, index: usize) -> Scalar {
        assert!(index < self.len());
        BinaryViewScalar::<T>::from(self.get(index)).into()
    }

    fn slice(&self, _range: impl RangeBounds<usize> + Clone + Debug) -> Self {
        todo!()
    }

    fn try_into_mut(self) -> Result<BinaryViewVectorMut<T>, Self> {
        let views_mut = match self.views.try_into_mut() {
            Ok(views_mut) => views_mut,
            Err(views) => {
                return Err(Self {
                    views,
                    validity: self.validity,
                    buffers: self.buffers,
                    _marker: std::marker::PhantomData,
                });
            }
        };

        let validity_mut = match self.validity.try_into_mut() {
            Ok(validity_mut) => validity_mut,
            Err(validity) => {
                return Err(Self {
                    views: views_mut.freeze(),
                    validity,
                    buffers: self.buffers,
                    _marker: std::marker::PhantomData,
                });
            }
        };

        let buffers_mut = match Arc::try_unwrap(self.buffers) {
            Ok(buffers) => buffers.into_vec(),
            Err(buffers) => {
                // Backup: collect a new Vec with clones of each buffer
                buffers.iter().cloned().collect()
            }
        };

        // SAFETY: the BinaryViewVector maintains the same invariants that are
        //  otherwise checked in the safe BinaryViewVectorMut constructor.
        unsafe {
            Ok(BinaryViewVectorMut::new_unchecked(
                views_mut,
                validity_mut,
                buffers_mut,
            ))
        }
    }

    fn into_mut(self) -> BinaryViewVectorMut<T> {
        let views_mut = self.views.into_mut();
        let validity_mut = self.validity.into_mut();

        // If someone else has a strong reference to the `Arc`, clone the underlying data (which is
        // just a **different** reference count increment).
        let buffers_mut = Arc::try_unwrap(self.buffers)
            .unwrap_or_else(|arc| (*arc).clone())
            .into_vec();

        // SAFETY: The BinaryViewVector maintains the exact same invariants as the immutable
        // version, so all invariants are still upheld.
        unsafe { BinaryViewVectorMut::new_unchecked(views_mut, validity_mut, buffers_mut) }
    }
}

#[cfg(test)]
mod tests {
    use vortex_buffer::{buffer, ByteBuffer};
    use vortex_mask::Mask;

    use crate::binaryview::view::BinaryView;
    use crate::binaryview::{StringVector, StringVectorMut};
    use crate::{VectorMutOps, VectorOps};
}
