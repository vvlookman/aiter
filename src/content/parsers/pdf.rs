use std::{collections::HashMap, path::Path};

use lopdf::Document;
use rayon::prelude::*;

use crate::{
    content::doc::text::{TextDoc, TextDocOutline},
    error::{AiterError, AiterResult},
};

pub fn to_text_doc(path: &Path, _source: &str) -> AiterResult<TextDoc> {
    let mut pages_map: HashMap<usize, usize> = HashMap::new();
    let mut pages: Vec<String> = vec![];

    let pdf_doc = Document::load(&*path.to_string_lossy())?;
    for (i, _) in pdf_doc.page_iter().enumerate() {
        let page_number = (i + 1) as u32;
        if let Ok(text) = pdf_doc.extract_text(&[page_number]) {
            if !text.trim().is_empty() {
                pages_map.insert(i, pages.len());
                pages.push(text);
            }
        }
    }

    if pages.is_empty() {
        return Err(AiterError::Invalid(format!("{} is empty", path.display())));
    }

    let outlines: Vec<TextDocOutline> = if let Ok(pdf_toc) = pdf_doc.get_toc() {
        pdf_toc
            .toc
            .par_iter()
            .filter_map(|toc_item| {
                pages_map.get(&toc_item.page).map(|index| TextDocOutline {
                    title: toc_item.title.clone(),
                    page: *index,
                    children: vec![],
                })
            })
            .collect()
    } else {
        vec![]
    };

    Ok(TextDoc {
        title: None,
        pages,
        outlines,
    })
}
