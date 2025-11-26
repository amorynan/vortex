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
        // Convert codes and values to their preferred Arrow representation
        let (key_type, value_type) = match arrow_type {
            // When a specific dictionary type is requested, use those types
            Some(DataType::Dictionary(key_type, value_type)) => {
                (key_type.as_ref().clone(), value_type.as_ref().clone())
            }
            // When no type is specified, emit dictionary with preferred types
            None => {
                let codes_arrow = array.codes().clone().into_arrow_preferred()?;
                let values_arrow = array.values().clone().into_arrow_preferred()?;
                (
                    codes_arrow.data_type().clone(),
                    values_arrow.data_type().clone(),
                )
            }
            // For non-dictionary target types, fall back to canonicalization
            Some(_) => return Ok(None),
        };

        // Convert codes to the key type
        let codes_arrow = array.codes().clone().into_arrow(&key_type)?;

        // Convert values to the value type
        let values_arrow = array.values().clone().into_arrow(&value_type)?;

        // Build the Arrow dictionary array
        let codes_data = codes_arrow.to_data();
        let dict_type = DataType::Dictionary(Box::new(key_type), Box::new(value_type));

        // Create the dictionary array data
        let array_data = ArrayDataBuilder::new(dict_type)
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
    fn test_dict_to_arrow_preferred_produces_dictionary() {
        // Create a dict-encoded string array
        let values = VarBinViewArray::from_iter_str(["hello", "world"]);
        let codes = buffer![0u32, 1, 0, 1, 0].into_array();
        let dict = DictArray::new(codes, values.into_array());

        // When no target type is specified, it should produce a Dictionary array
        // with the preferred value type (Utf8View for strings)
        let arrow = to_arrow_preferred(dict.as_ref()).unwrap();

        // The result should be Dictionary<_, Utf8View>
        match arrow.data_type() {
            DataType::Dictionary(_, value_type) => {
                assert_eq!(value_type.as_ref(), &DataType::Utf8View);
            }
            other => panic!("Expected Dictionary type, got {:?}", other),
        }
        assert_eq!(arrow.len(), 5);
    }

    #[test]
    fn test_dict_to_arrow_explicit_dictionary() {
        // Create a dict-encoded string array
        let values = VarBinViewArray::from_iter_str(["hello", "world"]);
        let codes = buffer![0u32, 1, 0, 1, 0].into_array();
        let dict = DictArray::new(codes, values.into_array());

        // When explicitly requesting Dictionary type, it should produce one
        let dict_type =
            DataType::Dictionary(Box::new(DataType::UInt32), Box::new(DataType::Utf8View));
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

    /// Minimal reproduction of the Arrow cast issue:
    /// Casting Dictionary<UInt32, Utf8View> -> Utf8View
    #[test]
    fn test_arrow_cast_dict_utf8view_to_utf8view() {
        use arrow_array::StringViewArray;
        use arrow_array::builder::StringViewBuilder;
        use arrow_cast::cast;

        // Build a Dictionary<UInt32, Utf8View> array directly in Arrow
        let mut values_builder = StringViewBuilder::new();
        values_builder.append_value("hello");
        values_builder.append_value("world");
        let values: StringViewArray = values_builder.finish();

        let keys = arrow_array::UInt32Array::from(vec![0u32, 1, 0, 1, 0]);

        let dict_array = arrow_array::DictionaryArray::new(keys, std::sync::Arc::new(values));

        // Verify we have Dictionary<UInt32, Utf8View>
        assert!(matches!(
            dict_array.data_type(),
            DataType::Dictionary(k, v) if **k == DataType::UInt32 && **v == DataType::Utf8View
        ));

        // Try to cast Dictionary<UInt32, Utf8View> -> Utf8View
        // This is where the error "Cannot cast Utf8View to StringArray" occurs
        let result = cast(&dict_array, &DataType::Utf8View);

        match result {
            Ok(casted) => {
                assert_eq!(casted.data_type(), &DataType::Utf8View);
                assert_eq!(casted.len(), 5);
            }
            Err(e) => {
                panic!(
                    "Arrow cast from Dictionary<UInt32, Utf8View> to Utf8View failed: {}",
                    e
                );
            }
        }
    }
}
