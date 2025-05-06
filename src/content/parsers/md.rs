use std::{collections::HashMap, fs::File, io::Read, path::Path, sync::LazyLock};

use pulldown_cmark::*;
use regex::Regex;

use crate::{
    content::doc::markdown::{MarkdownDoc, MarkdownDocOutline},
    error::{AiterError, AiterResult},
};

pub fn to_markdown_doc(path: &Path, _source: &str) -> AiterResult<MarkdownDoc> {
    let mut file = File::open(path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let mut text = String::new();
    let mut start_tags: Vec<(Tag, String)> = vec![];

    let mut heading_texts: Vec<(usize, String)> = vec![];

    let stream = TextMergeStream::new(Parser::new_ext(&content, Options::all()));
    for event in stream {
        let mut process_text = |s: &str| {
            if let Some(mut last) = start_tags.pop() {
                last.1.push_str(s);
                start_tags.push(last);
            } else {
                text.push_str(s);
            }
        };

        match event {
            Event::Code(s) => {
                process_text(&format!("`{}`", s));
            }
            Event::DisplayMath(s) => {
                process_text(&format!("$$\n{}\n$$", s));
            }
            Event::End(end_tag) => {
                let mut current_text = String::new();
                while let Some((tag, text_follow_tag)) = &start_tags.pop() {
                    let s = format!("{}{}", &text_follow_tag, &current_text);
                    current_text = format_with_tag(&s, Some(tag));

                    if let Tag::Heading { level, .. } = tag {
                        let n = match level {
                            HeadingLevel::H1 => 1,
                            HeadingLevel::H2 => 2,
                            HeadingLevel::H3 => 3,
                            HeadingLevel::H4 => 4,
                            HeadingLevel::H5 => 5,
                            HeadingLevel::H6 => 6,
                        };
                        heading_texts.push((n, s));
                    }

                    if tag.to_end() == end_tag {
                        break;
                    }
                }

                if start_tags.is_empty() {
                    text.push_str(&current_text);
                } else {
                    let (last_tag, last_text) = start_tags.pop().unwrap();
                    start_tags.push((last_tag, format!("{}{}", &last_text, &current_text)));
                }
            }
            Event::HardBreak | Event::SoftBreak => {
                process_text("\n");
            }
            Event::Html(s) => {
                process_text(&s);
            }
            Event::InlineHtml(s) => {
                process_text(&s);
            }
            Event::InlineMath(s) => {
                process_text(&format!("$ {} $", s));
            }
            Event::Rule => {
                process_text("---\n\n");
            }
            Event::Start(start_tag) => {
                start_tags.push((start_tag, String::new()));
            }
            Event::TaskListMarker(checked) => {
                process_text(&format!("[{}] ", if checked { "x" } else { " " }));
            }
            Event::Text(s) => {
                process_text(&s);
            }
            _ => {}
        }
    }

    let mut trimmed_text = text.trim();

    if trimmed_text.is_empty() {
        return Err(AiterError::Invalid(format!("{} is empty", path.display())));
    }

    let mut heading_count_map: HashMap<usize, usize> = HashMap::new();
    for (n, _) in &heading_texts {
        *heading_count_map.entry(*n).or_insert(0) += 1;
    }

    let heading_title = if let Some((n, trimmed_text)) = heading_texts.first() {
        if heading_texts.iter().filter(|(level, _)| level == n).count() == 1 {
            Some((*n, trimmed_text.clone()))
        } else {
            None
        }
    } else {
        None
    };

    let heading_level = heading_title.clone().map(|(level, _)| level).unwrap_or(0);
    let paragraph_level = heading_texts
        .iter()
        .map(|(level, _)| *level)
        .filter(|n| *n > heading_level)
        .min();

    let title = heading_title.clone().map(|(_, text)| text);
    if let Some((n, title)) = heading_title {
        trimmed_text = trimmed_text
            .strip_prefix(&format!("{} {}", "#".repeat(n), &title))
            .unwrap_or(trimmed_text);
    }

    let mut outlines: Vec<MarkdownDocOutline> = vec![];

    let pages = if let Some(n) = paragraph_level {
        let mut pages: Vec<String> = vec![];

        let indicator = format!("{} ", "#".repeat(n));
        let mut page = String::new();

        for line in trimmed_text.lines() {
            if line.starts_with(&indicator) {
                if !page.trim().is_empty() {
                    pages.push(page.to_string());
                    page.clear();
                }

                outlines.push(MarkdownDocOutline {
                    title: line.strip_prefix(&indicator).unwrap_or(line).to_string(),
                    page: pages.len(),
                    children: vec![],
                });
            }

            page.push_str(line);
            page.push('\n');
        }

        if !page.trim().is_empty() {
            pages.push(page.to_string());
        }

        pages
    } else {
        vec![text]
    };

    Ok(MarkdownDoc {
        title,
        pages,
        outlines,
    })
}

static REGEX_NOT_TABLE_SEP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^\n|]").expect("NOT_TABLE_SEP regex is invalid"));

fn format_with_tag(text: &str, tag: Option<&Tag>) -> String {
    if let Some(tag) = tag {
        match tag {
            Tag::BlockQuote(_) => format!(
                "{}\n",
                text.trim_start_matches('\n')
                    .trim_end_matches('\n')
                    .split('\n')
                    .map(|s| format!("> {}", s))
                    .collect::<Vec<String>>()
                    .join("\n\n")
            ),
            Tag::CodeBlock(kind) => match kind {
                CodeBlockKind::Indented => {
                    format!("```\n{}```\n\n", text)
                }
                CodeBlockKind::Fenced(s) => {
                    format!("```{}\n{}```\n\n", s, text)
                }
            },
            Tag::Emphasis => format!("*{}*", text),
            Tag::Heading { level, .. } => {
                let n = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                format!("{} {}\n\n", "#".repeat(n), text)
            }
            Tag::HtmlBlock => format!("{}\n\n", text),
            Tag::Image { .. } => "".to_string(),
            Tag::Item => format!("- {}\n", text),
            Tag::Link {
                link_type,
                dest_url,
                ..
            } => match link_type {
                LinkType::Autolink | LinkType::Email => format!("<{}>", text),
                LinkType::Collapsed => format!("[{}][]", text),
                LinkType::Inline => format!("[{}]({})", text, dest_url),
                LinkType::Reference => format!("[{}][{}]", text, dest_url),
                LinkType::Shortcut => format!("[{}]", text),
                LinkType::WikiLink { has_pothole } => {
                    if *has_pothole {
                        format!("[[{}|{}]]", text, dest_url)
                    } else {
                        format!("[[{}]]", text)
                    }
                }
                _ => text.to_string(),
            },
            Tag::List(_) => format!("{}\n", text),
            Tag::MetadataBlock(_) => format!("{}\n\n", text),
            Tag::Paragraph => format!("{}\n\n", text),
            Tag::Strikethrough => format!("~~{}~~", text),
            Tag::Strong => format!("**{}**", text),
            Tag::Subscript => format!("~{}~", text),
            Tag::Superscript => format!("^{}^", text),
            Tag::TableCell => format!(" {} |", text),
            Tag::TableHead => {
                let head = format!("|{}", text);
                let sep = REGEX_NOT_TABLE_SEP.replace_all(&head, "-");
                format!("{}\n{}\n", head, sep)
            }
            Tag::TableRow => format!("|{}\n", text),
            _ => text.to_string(),
        }
    } else {
        text.to_string()
    }
}
