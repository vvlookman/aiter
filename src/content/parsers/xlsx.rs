use std::path::Path;

use calamine::{open_workbook_auto, Reader};

use crate::{
    content::doc::sheet::{SheetData, SheetDoc},
    error::AiterResult,
};

pub fn to_sheet_doc(path: &Path, _source: &str) -> AiterResult<SheetDoc> {
    let mut workbook = open_workbook_auto(path)?;
    let sheet_names = workbook.sheet_names();

    let mut pages: Vec<(String, SheetData)> = Vec::with_capacity(sheet_names.len());
    for sheet_name in sheet_names {
        let range = workbook.worksheet_range(&sheet_name)?;

        let sheet_data = SheetData {
            headers: range.headers(),
            rows: range
                .rows()
                .map(|row| row.iter().map(|cell| cell.to_string()).collect())
                .collect(),
        };

        pages.push((sheet_name.to_string(), sheet_data));
    }

    Ok(SheetDoc { pages })
}
