// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use vortex_dtype::half::f16;
use vortex_vector::match_each_pvector;
use vortex_vector::primitive::{PVector, PrimitiveVector};

use crate::filter::{Filter, FilterMask};

impl<M: FilterMask> Filter<M> for &PrimitiveVector
where
    for<'a> &'a PVector<i8>: Filter<M, Output = PVector<i8>>,
    for<'a> &'a PVector<i16>: Filter<M, Output = PVector<i16>>,
    for<'a> &'a PVector<i32>: Filter<M, Output = PVector<i32>>,
    for<'a> &'a PVector<i64>: Filter<M, Output = PVector<i64>>,
    for<'a> &'a PVector<u8>: Filter<M, Output = PVector<u8>>,
    for<'a> &'a PVector<u16>: Filter<M, Output = PVector<u16>>,
    for<'a> &'a PVector<u32>: Filter<M, Output = PVector<u32>>,
    for<'a> &'a PVector<u64>: Filter<M, Output = PVector<u64>>,
    for<'a> &'a PVector<f16>: Filter<M, Output = PVector<f16>>,
    for<'a> &'a PVector<f32>: Filter<M, Output = PVector<f32>>,
    for<'a> &'a PVector<f64>: Filter<M, Output = PVector<f64>>,
{
    type Output = PrimitiveVector;

    fn filter(self, selection: &M) -> Self::Output {
        match_each_pvector!(self, |v| { Filter::<M>::filter(v, selection).into() })
    }
}

impl<M: FilterMask> Filter<M> for &mut PrimitiveVector
where
    for<'a> &'a mut PVector<i8>: Filter<M, Output = ()>,
    for<'a> &'a mut PVector<i16>: Filter<M, Output = ()>,
    for<'a> &'a mut PVector<i32>: Filter<M, Output = ()>,
    for<'a> &'a mut PVector<i64>: Filter<M, Output = ()>,
    for<'a> &'a mut PVector<u8>: Filter<M, Output = ()>,
    for<'a> &'a mut PVector<u16>: Filter<M, Output = ()>,
    for<'a> &'a mut PVector<u32>: Filter<M, Output = ()>,
    for<'a> &'a mut PVector<u64>: Filter<M, Output = ()>,
    for<'a> &'a mut PVector<f16>: Filter<M, Output = ()>,
    for<'a> &'a mut PVector<f32>: Filter<M, Output = ()>,
    for<'a> &'a mut PVector<f64>: Filter<M, Output = ()>,
{
    type Output = ();

    fn filter(self, selection: &M) -> Self::Output {
        match_each_pvector!(self, |v| { Filter::<M>::filter(v, selection) })
    }
}
