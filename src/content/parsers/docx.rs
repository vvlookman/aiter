use std::{fs::File, io::Read, path::Path};

use docx_rs::{
    read_docx, DocumentChild, Table, TableCellContent, TableChild::TableRow,
    TableRowChild::TableCell,
};

use crate::{
    content::doc::text::TextDoc,
    error::{AiterError, AiterResult},
};

pub fn to_text_doc(path: &Path, source: &str) -> AiterResult<TextDoc> {
    let mut file = File::open(path)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let docx_doc = read_docx(&buffer)?;

    let mut page = String::new();

    for child in docx_doc.document.children {
        match child {
            DocumentChild::Paragraph(paragraph) => {
                page.push_str(&format!("{}\n\n", &paragraph.raw_text()));
            }
            DocumentChild::Table(table) => {
                page.push_str(&format!("{}\n\n", &format_table(&table)));
            }
            _ => {}
        }
    }

    if page.trim().is_empty() {
        return Err(AiterError::Invalid(format!("{} is empty", source)));
    }

    Ok(TextDoc {
        title: None,
        pages: vec![page],
        outlines: vec![],
    })
}

fn format_table(table: &Table) -> String {
    let mut s = String::new();

    for (i, TableRow(row)) in table.rows.iter().enumerate() {
        s.push_str("| ");
        for TableCell(cell) in &row.cells {
            for cell_content in &cell.children {
                match cell_content {
                    TableCellContent::Paragraph(paragraph) => {
                        let cell_text = paragraph.raw_text();
                        s.push_str(&format!(" {} |", &cell_text.replace("\n", " ").trim()));
                    }
                    TableCellContent::Table(table) => {
                        let cell_text = format_table(table);
                        s.push_str(&format!(" {} |", &cell_text.replace("\n", " ").trim()));
                    }
                    TableCellContent::StructuredDataTag(_)
                    | TableCellContent::TableOfContents(_) => {}
                }
            }
        }
        s.push('\n');

        if i == 0 {
            s.push_str("| ");
            for _ in &row.cells {
                s.push_str(" --- |");
            }
            s.push('\n');
        }
    }

    s
}
