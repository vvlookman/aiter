use std::{collections::HashMap, path::Path, str::from_utf8};

use epub::doc::{EpubDoc, NavPoint};
use rayon::prelude::*;

use crate::{
    content::doc::text::{TextDoc, TextDocOutline},
    error::{AiterError, AiterResult},
    utils::html::extract_texts_from_html,
};

pub fn to_text_doc(path: &Path, source: &str) -> AiterResult<TextDoc> {
    let mut pages_map: HashMap<String, usize> = HashMap::new();
    let mut pages: Vec<String> = vec![];

    let mut epub_doc = EpubDoc::new(path)?;
    while epub_doc.go_next() {
        if let Some(current_path) = epub_doc.get_current_path() {
            if let Some(html_data) = epub_doc.get_resource_by_path(&current_path) {
                if let Ok(html) = from_utf8(&html_data) {
                    let text = extract_texts_from_html(html).join("\n\n");
                    if !text.trim().is_empty() {
                        pages_map.insert(current_path.to_string_lossy().to_string(), pages.len());
                        pages.push(text);
                    }
                }
            }
        }
    }

    if pages.is_empty() {
        return Err(AiterError::Invalid(format!("{source} is empty")));
    }

    let outlines = epub_doc
        .toc
        .par_iter()
        .filter_map(|epub_navpoint| to_outline(epub_navpoint, &pages_map))
        .collect();

    let title = epub_doc
        .metadata
        .get("title")
        .map(|titles| titles.join(" "));

    Ok(TextDoc {
        title,
        pages,
        outlines,
    })
}

fn to_outline(
    epub_navpoint: &NavPoint,
    pages_map: &HashMap<String, usize>,
) -> Option<TextDocOutline> {
    let uri = epub_navpoint.content.to_string_lossy();
    let navpoint_path = uri.split('#').next().unwrap_or(&uri).to_string();
    if let Some(index) = pages_map.get(&navpoint_path) {
        let mut children: Vec<TextDocOutline> = vec![];

        for epub_sub_navpoint in epub_navpoint.children.iter() {
            children.push(to_outline(epub_sub_navpoint, pages_map)?);
        }

        return Some(TextDocOutline {
            title: epub_navpoint.label.clone(),
            page: *index,
            children,
        });
    };

    None
}
