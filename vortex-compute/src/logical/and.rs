// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use std::ops::BitAnd;
use vortex_buffer::BitBuffer;
use vortex_vector::bool::BoolVector;
use vortex_vector::{Cow, VectorOps};

use crate::logical::LogicalAnd;

// TODO(ngates): should we try to into_mut and reuse the existing buffer? Let's benchmark.
impl LogicalAnd for &BoolVector {
    type Output = BoolVector;

    fn and(self, other: &BoolVector) -> BoolVector {
        BoolVector::new(
            self.bits().and(other.bits()),
            self.validity().and(other.validity()),
        )
    }
}

impl LogicalAnd<&BoolVector> for BoolVector {
    type Output = BoolVector;

    fn and(self, other: &BoolVector) -> BoolVector {
        (&self).and(other)
    }
}

impl LogicalAnd for &Cow<BitBuffer> {
    type Output = Cow<BitBuffer>;

    fn and(self, other: Self) -> Self::Output {
        // FIXME(ngates): create a BitView to avoid cloning mutable buffers here
        let buffer = match (self, other) {
            (Cow::Frozen(a), Cow::Frozen(b)) => a.bitand(b),
            (Cow::Mutable(a), Cow::Frozen(b)) => a.clone().freeze().bitand(b),
            (Cow::Frozen(a), Cow::Mutable(b)) => a.bitand(b.clone().freeze()),
            (Cow::Mutable(a), Cow::Mutable(b)) => a.clone().freeze().bitand(&b.clone().freeze()),
        };
        Cow::Frozen(buffer)
    }
}

#[cfg(test)]
mod tests {
    use vortex_buffer::bitbuffer;
    use vortex_mask::Mask;
    use vortex_vector::bool::BoolVector;

    use super::*;

    #[test]
    fn test_and_basic() {
        let left = BoolVector::new(bitbuffer![1 1 0 0], Mask::new_true(4));
        let right = BoolVector::new(bitbuffer![1 0 1 0], Mask::new_true(4));

        let result = left.and(&right);
        assert_eq!(result.bits(), &bitbuffer![1 0 0 0]);
    }

    #[test]
    fn test_and_with_nulls() {
        let left = BoolVector::new(bitbuffer![1 0], Mask::from(bitbuffer![1 0]));
        let right = BoolVector::new(bitbuffer![1 1], Mask::new_true(2));

        let result = left.and(&right);
        // Validity is AND'd, so if either side is null, result is null
        assert_eq!(result.validity(), &Mask::from(bitbuffer![1 0]));
    }
}
