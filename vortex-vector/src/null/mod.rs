// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

//! Definition and implementation of [`NullVector`] and [`NullVector`].

// mod scalar;
// pub use scalar::NullScalar;

mod vector;
pub use vector::NullVector;

use crate::VectorMut;

impl From<NullVector> for VectorMut {
    fn from(v: NullVector) -> Self {
        Self::Null(v)
    }
}
