// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use vortex_dtype::i256;
use vortex_vector::decimal::{DVector, DecimalVector};
use vortex_vector::match_each_dvector;

use crate::filter::{Filter, FilterInPlace, FilterMask};

impl<M: FilterMask> Filter<M> for &DecimalVector
where
    for<'a> &'a DVector<i8>: Filter<M, Output = DVector<i8>>,
    for<'a> &'a DVector<i16>: Filter<M, Output = DVector<i16>>,
    for<'a> &'a DVector<i32>: Filter<M, Output = DVector<i32>>,
    for<'a> &'a DVector<i64>: Filter<M, Output = DVector<i64>>,
    for<'a> &'a DVector<i128>: Filter<M, Output = DVector<i128>>,
    for<'a> &'a DVector<i256>: Filter<M, Output = DVector<i256>>,
{
    type Output = DecimalVector;

    fn filter(self, selection: &M) -> Self::Output {
        match_each_dvector!(self, |d| { Filter::<M>::filter(d, selection).into() })
    }
}

impl<M: FilterMask> FilterInPlace<M> for DecimalVector
where
    DVector<i8>: FilterInPlace<M>,
    DVector<i16>: FilterInPlace<M>,
    DVector<i32>: FilterInPlace<M>,
    DVector<i64>: FilterInPlace<M>,
    DVector<i128>: FilterInPlace<M>,
    DVector<i256>: FilterInPlace<M>,
{
    fn filter_in_place(&mut self, selection: &M) {
        match_each_dvector!(self, |d| {
            FilterInPlace::<M>::filter_in_place(d, selection)
        });
    }
}
