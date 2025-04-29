use std::path::{Path, PathBuf};

use csv::Reader;

use crate::{
    content::doc::sheet::{SheetData, SheetDoc},
    error::AiterResult,
    utils::fs::extract_filestem_from_path,
};

pub fn to_sheet_doc(path: &Path, source: &str) -> AiterResult<SheetDoc> {
    let mut rdr = Reader::from_path(path)?;

    let headers = rdr
        .headers()
        .ok()
        .map(|sr| sr.iter().map(|s| s.to_string()).collect());

    let rows = rdr
        .records()
        .filter_map(|record| {
            record
                .ok()
                .map(|sr| sr.iter().map(|s| s.to_string()).collect())
        })
        .collect();

    let sheet_data = SheetData { headers, rows };

    Ok(SheetDoc {
        pages: vec![(
            extract_filestem_from_path(&PathBuf::from(source)),
            sheet_data,
        )],
    })
}
