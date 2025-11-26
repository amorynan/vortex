// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

use arrow_array::ArrayRef as ArrowArrayRef;
use arrow_array::make_array;
use arrow_data::ArrayDataBuilder;
use arrow_schema::DataType;
use vortex_error::VortexResult;

use super::DictVTable;
use crate::arrays::DictArray;
use crate::arrow::IntoArrowArray;
use crate::arrow::compute::ToArrowKernel;
use crate::arrow::compute::ToArrowKernelAdapter;
use crate::register_kernel;

impl ToArrowKernel for DictVTable {
    fn to_arrow(
        &self,
        array: &DictArray,
        arrow_type: Option<&DataType>,
    ) -> VortexResult<Option<ArrowArrayRef>> {
        // IMPORTANT: When no target arrow_type is specified, return None to fall back to
        // canonicalization. This avoids producing Arrow Dictionary<_, Utf8View> arrays which
        // can cause issues when DataFusion's schema adapter tries to cast them to Utf8
        // (StringArray). The canonical form (VarBinView -> Utf8View) is more portable and
        // doesn't have these casting issues.
        //
        // See: https://github.com/vortex-data/vortex/pull/4254
        let Some(arrow_type) = arrow_type else {
            return Ok(None);
        };

        // Only handle Dictionary types explicitly. For all other types, fall back to
        // canonicalization which will produce the appropriate canonical form.
        let DataType::Dictionary(key_type, value_type) = arrow_type else {
            return Ok(None);
        };

        // Convert codes to the requested key type
        let codes_arrow = array.codes().clone().into_arrow(key_type.as_ref())?;

        // Convert values to the requested value type
        let values_arrow = array.values().clone().into_arrow(value_type.as_ref())?;

        // Build the Arrow dictionary array
        let codes_data = codes_arrow.to_data();

        // Create the dictionary array data
        let array_data = ArrayDataBuilder::new(arrow_type.clone())
            .len(array.len())
            .buffers(codes_data.buffers().to_vec())
            .nulls(codes_data.nulls().cloned())
            .child_data(vec![values_arrow.to_data()])
            .build()?;

        Ok(Some(make_array(array_data)))
    }
}

register_kernel!(ToArrowKernelAdapter(DictVTable).lift());

#[cfg(test)]
mod tests {
    use arrow_schema::DataType;
    use vortex_buffer::buffer;

    use crate::Array;
    use crate::IntoArray;
    use crate::arrays::DictArray;
    use crate::arrays::VarBinViewArray;
    use crate::arrow::compute::to_arrow::to_arrow;
    use crate::arrow::compute::to_arrow::to_arrow_preferred;

    #[test]
    fn test_dict_to_arrow_preferred_canonicalizes() {
        // Create a dict-encoded string array
        let values = VarBinViewArray::from_iter_str(["hello", "world"]);
        let codes = buffer![0u32, 1, 0, 1, 0].into_array();
        let dict = DictArray::new(codes, values.into_array());

        // When no target type is specified, it should fall back to canonicalization
        // and produce Utf8View (not Dictionary<_, Utf8View>)
        let arrow = to_arrow_preferred(dict.as_ref()).unwrap();

        // The result should be Utf8View, NOT a Dictionary
        assert_eq!(arrow.data_type(), &DataType::Utf8View);
        assert_eq!(arrow.len(), 5);
    }

    #[test]
    fn test_dict_to_arrow_explicit_dictionary() {
        // Create a dict-encoded string array
        let values = VarBinViewArray::from_iter_str(["hello", "world"]);
        let codes = buffer![0u32, 1, 0, 1, 0].into_array();
        let dict = DictArray::new(codes, values.into_array());

        // When explicitly requesting Dictionary type, it should produce one
        let dict_type = DataType::Dictionary(
            Box::new(DataType::UInt32),
            Box::new(DataType::Utf8View),
        );
        let arrow = to_arrow(dict.as_ref(), &dict_type).unwrap();

        // The result should be a Dictionary
        assert!(matches!(arrow.data_type(), DataType::Dictionary(_, _)));
        assert_eq!(arrow.len(), 5);
    }

    #[test]
    fn test_dict_to_arrow_non_dictionary_type_canonicalizes() {
        // Create a dict-encoded string array
        let values = VarBinViewArray::from_iter_str(["hello", "world"]);
        let codes = buffer![0u32, 1, 0, 1, 0].into_array();
        let dict = DictArray::new(codes, values.into_array());

        // When requesting a non-dictionary type like Utf8View, it should canonicalize
        let arrow = to_arrow(dict.as_ref(), &DataType::Utf8View).unwrap();

        // The result should be Utf8View
        assert_eq!(arrow.data_type(), &DataType::Utf8View);
        assert_eq!(arrow.len(), 5);
    }
}
