use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use chardetng::EncodingDetector;

use crate::{
    content::doc::text::TextDoc,
    error::{AiterError, AiterResult},
    utils::fs::extract_filestem_from_path,
};

pub fn to_text_doc(path: &Path, source: &str) -> AiterResult<TextDoc> {
    let mut file = File::open(path)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut detector = EncodingDetector::new();
    detector.feed(&buffer, false);
    let encoding = detector.guess(None, true);

    let (text, _, _) = encoding.decode(&buffer);
    let trimmed_text = text.trim();

    if trimmed_text.is_empty() {
        return Err(AiterError::Invalid(format!("{} is empty", source)));
    }

    Ok(TextDoc {
        title: Some(extract_filestem_from_path(&PathBuf::from(source))),
        pages: vec![trimmed_text.to_string()],
        outlines: vec![],
    })
}
