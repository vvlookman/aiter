use std::fmt::Display;

use csv::WriterBuilder;
use serde::{Deserialize, Serialize};

use crate::{
    content::{
        doc::{DocContent, DocContentType},
        seg::{sheet::SheetSegContent, SegContent},
    },
    error::AiterResult,
    utils::{
        compress,
        text::{to_tokens, truncate_format},
    },
    Tokenizer, SPLIT_TOKENS_OF_SEG, TRUNCATE_PREVIEW,
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SheetDoc {
    pub pages: Vec<(String, SheetData)>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SheetData {
    pub headers: Option<Vec<String>>,
    pub rows: Vec<Vec<String>>,
}

impl DocContent for SheetDoc {
    fn get_title(&self) -> Option<String> {
        None
    }

    fn get_preview(&self) -> String {
        self.pages.first().map_or(String::new(), |(_, data)| {
            let mut s = String::new();

            if let Some(headers) = &data.headers {
                s.push_str(&headers.join(" , "));
                s.push('\n');
            }

            let max_rows = 3;
            for i in 0..max_rows {
                if let Some(row) = &data.rows.get(i) {
                    s.push_str(&row.join(" , "));
                    if i < max_rows - 1 {
                        s.push('\n');
                    }
                }
            }

            truncate_format(s.trim(), TRUNCATE_PREVIEW, false)
        })
    }

    fn get_type(&self) -> DocContentType {
        DocContentType::Sheet
    }

    fn split(&self, tokenizer: &Tokenizer) -> Vec<Vec<Box<dyn SegContent>>> {
        self.pages
            .iter()
            .map(|(_, data)| {
                let mut segs = vec![];

                // Split columns, make each column as a segment, split the column again if it is bigger than the token window
                {
                    let col_num = if let Some(headers) = &data.headers {
                        headers.len()
                    } else {
                        data.rows.first().map(|row| row.len()).unwrap_or(0)
                    };

                    if col_num > 1 {
                        for i in 0..col_num {
                            let header_vec: Option<Vec<String>> =
                                if let Some(headers) = &data.headers {
                                    headers.get(i).map(|header| vec![header.to_string()])
                                } else {
                                    None
                                };

                            let rows: Vec<Vec<String>> = data
                                .rows
                                .iter()
                                .map(|row| {
                                    row.get(i)
                                        .map_or(vec!["".to_string()], |v| vec![v.to_string()])
                                })
                                .collect();

                            segs.extend(split_to_segs_by_max_tokens(&header_vec, &rows, tokenizer));
                        }
                    }
                }

                // Split rows, make several rows as a segment
                if data.rows.len() > 1 {
                    segs.extend(split_to_segs_by_max_tokens(
                        &data.headers,
                        &data.rows,
                        tokenizer,
                    ));
                }

                segs
            })
            .filter(|segs| !segs.is_empty())
            .collect()
    }

    fn to_string(&self) -> String {
        let mut s = String::new();

        for (title, sheet_data) in &self.pages {
            s.push_str(&format!("# {}", title));
            s.push_str("\n\n");

            s.push_str(&sheet_data.to_string());
            s.push_str("\n\n");
        }

        s
    }

    fn try_from_bytes(bytes: &[u8]) -> AiterResult<Self> {
        let json = compress::decode(bytes)?;
        Ok(serde_json::from_slice(&json)?)
    }

    fn try_into_bytes(&self) -> AiterResult<Vec<u8>> {
        let json = serde_json::to_vec(self)?;
        Ok(compress::encode(&json)?)
    }
}

fn split_to_segs_by_max_tokens(
    headers: &Option<Vec<String>>,
    rows: &[Vec<String>],
    tokenizer: &Tokenizer,
) -> Vec<Box<dyn SegContent>> {
    let headers_tokens_count: usize = if let Some(headers) = headers {
        headers
            .iter()
            .map(|s| to_tokens(s, tokenizer).len() + 1)
            .sum()
    } else {
        0
    };

    let mut segs: Vec<Box<dyn SegContent>> = vec![];

    let mut prev_rows: Vec<Vec<String>> = vec![];
    let mut prev_tokens_count: usize = headers_tokens_count;
    for row in rows {
        let row_tokens_count: usize = row.iter().map(|s| to_tokens(s, tokenizer).len() + 1).sum();

        if prev_tokens_count + row_tokens_count > SPLIT_TOKENS_OF_SEG {
            if !prev_rows.is_empty() {
                segs.push(Box::new(SheetSegContent {
                    data: SheetData {
                        headers: headers.clone(),
                        rows: prev_rows.to_vec(),
                    },
                }) as Box<dyn SegContent>);
            }

            prev_rows = vec![];
            prev_tokens_count = headers_tokens_count;
        } else {
            prev_rows.push(row.to_vec());
            prev_tokens_count += row_tokens_count;
        }
    }

    if !prev_rows.is_empty() {
        segs.push(Box::new(SheetSegContent {
            data: SheetData {
                headers: headers.clone(),
                rows: prev_rows.to_vec(),
            },
        }) as Box<dyn SegContent>);
    }

    segs
}

impl Display for SheetData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);

        if let Some(headers) = &self.headers {
            let _ = wtr.write_record(headers);
        }

        for row in &self.rows {
            let _ = wtr.write_record(row);
        }

        let _ = wtr.flush();

        if let Ok(bytes) = wtr.into_inner() {
            write!(f, "{}", String::from_utf8_lossy(&bytes))
        } else {
            write!(f, "")
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::CURRENT_TOKENIZER;

    #[test]
    fn test_split() {
        let doc = SheetDoc {
            pages: vec![(
                "Sheet1".to_string(),
                SheetData {
                    headers: Some(vec!["name".to_string(), "value".to_string()]),
                    rows: vec![
                        ["a".to_string(), "1".to_string()].to_vec(),
                        ["b".to_string(), "2".to_string()].to_vec(),
                        ["c".to_string(), "3".to_string()].to_vec(),
                    ],
                },
            )],
        };

        let segs = doc.split(&CURRENT_TOKENIZER);

        println!("{:?}", &segs);

        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].len(), 3);
    }
}
