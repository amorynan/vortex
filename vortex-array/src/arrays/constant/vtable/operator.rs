// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use vortex_dtype::{
    match_each_decimal_value_type, match_each_native_ptype, DType, DecimalType, PrecisionScale,
};
use vortex_error::{VortexExpect, VortexResult};
use vortex_scalar::{BinaryScalar, BoolScalar, DecimalScalar, PrimitiveScalar, Scalar, Utf8Scalar};
use vortex_vector::binaryview::{BinaryVector, StringVector};
use vortex_vector::bool::BoolVector;
use vortex_vector::decimal::{DVectorMut, DecimalVectorMut};
use vortex_vector::null::NullVectorMut;
use vortex_vector::primitive::{PVectorMut, PrimitiveVectorMut};
use vortex_vector::{VectorMut, VectorMutOps};

use crate::arrays::{ConstantArray, ConstantVTable};
use crate::execution::{kernel, BatchKernelRef, BindCtx};
use crate::vtable::OperatorVTable;
use crate::ArrayRef;

impl OperatorVTable<ConstantVTable> for ConstantVTable {
    fn bind(
        array: &ConstantArray,
        selection: Option<&ArrayRef>,
        ctx: &mut dyn BindCtx,
    ) -> VortexResult<BatchKernelRef> {
        let mask = ctx.bind_selection(array.len, selection)?;
        let scalar = array.scalar().clone();

        Ok(kernel(move || {
            // TODO(ngates): would be good to do a sum aggregation, rather than execution.
            let mask = mask.execute()?;
            Ok(to_vector(scalar, mask.true_count()).freeze())
        }))
    }
}

fn to_vector(scalar: Scalar, len: usize) -> VectorMut {
    match scalar.dtype() {
        DType::Null => NullVectorMut::new(len).into(),
        DType::Bool(_) => to_vector_bool(scalar.as_bool(), len).into(),
        DType::Primitive(..) => to_vector_primitive(scalar.as_primitive(), len).into(),
        DType::Decimal(..) => to_vector_decimal(scalar.as_decimal(), len).into(),
        DType::Utf8(_) => to_vector_utf8(scalar.as_utf8(), len).into(),
        DType::Binary(_) => to_vector_binary(scalar.as_binary(), len).into(),
        DType::List(..) => unimplemented!("List constant vectorization"),
        DType::FixedSizeList(..) => unimplemented!("FixedSizeList constant vectorization"),
        DType::Struct(..) => unimplemented!("Struct constant vectorization"),
        DType::Extension(_) => to_vector(scalar.as_extension().storage(), len),
    }
}

fn to_vector_bool(scalar: BoolScalar, len: usize) -> BoolVector {
    let mut vec = BoolVector::with_capacity(len);
    match scalar.value() {
        Some(v) => vec.append_values(v, len),
        None => vec.append_nulls(len),
    }
    vec
}

fn to_vector_primitive(scalar: PrimitiveScalar, len: usize) -> PrimitiveVectorMut {
    match_each_native_ptype!(scalar.ptype(), |T| {
        let mut vec = PVectorMut::<T>::with_capacity(len);
        match scalar.typed_value::<T>() {
            Some(v) => vec.append_values(v, len),
            None => vec.append_nulls(len),
        }
        vec.into()
    })
}

fn to_vector_decimal(scalar: DecimalScalar, len: usize) -> DecimalVectorMut {
    let decimal_dtype = scalar
        .dtype()
        .as_decimal_opt()
        .vortex_expect("Decimal scalar must have a decimal type");
    let decimal_type = DecimalType::smallest_decimal_value_type(decimal_dtype);

    match_each_decimal_value_type!(decimal_type, |D| {
        let ps = PrecisionScale::<D>::new(decimal_dtype.precision(), decimal_dtype.scale());
        let mut vec = DVectorMut::<D>::with_capacity(ps, len);
        match scalar.decimal_value() {
            Some(v) => vec
                .try_append_n(v.cast::<D>().vortex_expect("known to fit"), len)
                .vortex_expect("known to fit"),
            None => vec.append_nulls(len),
        }
        vec.into()
    })
}

fn to_vector_utf8(scalar: Utf8Scalar, len: usize) -> StringVector {
    let mut vec = StringVector::with_capacity(len);
    match scalar.value() {
        Some(v) => vec.append_values(v.as_ref(), len),
        None => vec.append_nulls(len),
    }
    vec
}

fn to_vector_binary(scalar: BinaryScalar, len: usize) -> BinaryVector {
    let mut vec = BinaryVector::with_capacity(len);
    match scalar.value() {
        Some(v) => vec.append_values(v.as_ref(), len),
        None => vec.append_nulls(len),
    }
    vec
}
