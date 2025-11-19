// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use crate::filter::{Filter, FilterMask};
use vortex_mask::Mask;
use vortex_vector::null::NullVector;
use vortex_vector::Cow;

impl<M: FilterMask> Filter<M> for &NullVector
where
    for<'a> &'a Cow<Mask>: Filter<M, Output = Mask>,
{
    type Output = NullVector;

    fn filter(self, selection: &M) -> Self::Output {
        NullVector::new(selection.true_count())
    }
}

impl<M: FilterMask> Filter<M> for &mut NullVector
where
    for<'a> &'a mut Cow<Mask>: Filter<M, Output = ()>,
{
    type Output = ();

    fn filter(self, selection: &M) -> Self::Output {
        *self = NullVector::new(selection.true_count())
    }
}

#[cfg(test)]
mod tests {
    use vortex_mask::Mask;
    use vortex_vector::VectorOps;

    use super::*;

    #[test]
    fn test_filter_null_vector_with_mask() {
        let vec = NullVector::new(5);
        let mask = Mask::from_iter([true, false, true, false, true]);

        let filtered = vec.filter(&mask);

        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered.validity().true_count(), 0);
    }

    #[test]
    fn test_filter_null_vector_all_true() {
        let vec = NullVector::new(3);
        let mask = Mask::new_true(3);

        let filtered = vec.filter(&mask);

        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered.validity().true_count(), 0);
    }

    #[test]
    fn test_filter_null_vector_all_false() {
        let vec = NullVector::new(3);
        let mask = Mask::new_false(3);

        let filtered = vec.filter(&mask);

        assert_eq!(filtered.len(), 0);
    }
}
