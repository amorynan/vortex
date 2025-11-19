// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

//! Definition and implementation of [`NullVector`] and [`NullVectorMut`].

// mod scalar;
// pub use scalar::NullScalar;

mod vector;
pub use vector::NullVectorMut;

use crate::VectorMut;

impl From<NullVectorMut> for VectorMut {
    fn from(v: NullVectorMut) -> Self {
        Self::Null(v)
    }
}
