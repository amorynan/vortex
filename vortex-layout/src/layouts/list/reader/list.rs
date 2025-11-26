// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

//! Reader implementation for ListLayouts containing lists with reified offsets buffers.

use std::collections::BTreeSet;
use std::ops::Range;
use std::sync::Arc;

use async_trait::async_trait;
use vortex_array::expr::Expression;
use vortex_array::MaskFuture;
use vortex_dtype::DType;
use vortex_dtype::FieldMask;
use vortex_error::VortexResult;
use vortex_mask::Mask;

use crate::layouts::list::ListLayout;
use crate::ArrayFuture;
use crate::LayoutReader;
use crate::LayoutReaderRef;

/// `LayoutReader` for the list layout holding
/// `List`-typed data.
pub struct ListReader {
    name: Arc<str>,
    layout: ListLayout,
    offsets: LayoutReaderRef,
    elements: LayoutReaderRef,
    validity: Option<LayoutReaderRef>,
}

impl ListReader {
    pub fn new(
        name: Arc<str>,
        layout: ListLayout,
        offsets: LayoutReaderRef,
        elements: LayoutReaderRef,
        validity: Option<LayoutReaderRef>,
    ) -> Self {
        Self {
            name,
            layout,
            offsets,
            elements,
            validity,
        }
    }

    // TODO(aduffy): does it make more sense to read offsets in their entirety, or
    //  read them in ranges?
}

#[async_trait]
impl LayoutReader for ListReader {
    fn name(&self) -> &Arc<str> {
        &self.name
    }

    fn dtype(&self) -> &DType {
        &self.layout.dtype
    }

    fn row_count(&self) -> u64 {
        self.layout.row_count
    }

    fn register_splits(
        &self,
        field_mask: &[FieldMask],
        row_range: &Range<u64>,
        splits: &mut BTreeSet<u64>,
    ) -> VortexResult<()> {
        // TODO(aduffy): how do I handle a field_mask here?

        // accessing element i requires reading offset i and i+1
        // so splits for row_range of offset start..end
        // must register on the offsets row range start..end+1
        let offsets_range = row_range.start..row_range.end + 1;

        self.offsets
            .register_splits(field_mask, &offsets_range, splits)?;

        Ok(())
    }

    fn pruning_evaluation(
        &self,
        _row_range: &Range<u64>,
        _expr: &Expression,
        mask: Mask,
    ) -> VortexResult<MaskFuture> {
        // TODO(aduffy): support scalar functions over Lists once we have a MapList expression.
        Ok(MaskFuture::ready(mask))
    }

    fn filter_evaluation(
        &self,
        _row_range: &Range<u64>,
        _expr: &Expression,
        _mask: MaskFuture,
    ) -> VortexResult<MaskFuture> {
        // TODO(aduffy): pushdown IsNull / IsNotNull over validity
        todo!()
    }

    fn projection_evaluation(
        &self,
        row_range: &Range<u64>,
        expr: &Expression,
        mask: MaskFuture,
    ) -> VortexResult<ArrayFuture> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    //! List Layouts give us the opportunity to have deeply nested data accesses with the benefits
    //! of full vectorization.
    //!
    //! Say that we have the following schema: `{a:list({b:i32?}?)}`
    //!
    //! This can be written with the following layout tree (some simple nodes elided):
    //!
    //! ```text
    //! StructLayout
    //! |__ a: ListLayout
    //!     |__ offsets: FlatLayout
    //!     |__ elements: StructLayout
    //!         |__ b: ChunkedLayout
    //!             |_ [0]: FlatLayout
    //!             |_ [1]: FlatLayout
    //!             |_ ...
    //!             |_ [N]: FlatLayout
    //! ```
    //!
    //! The ListLayout
}
