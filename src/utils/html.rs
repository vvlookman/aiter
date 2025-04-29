use rayon::prelude::*;
use scraper::{ElementRef, Html, Node};

pub fn extract_texts_from_html(html: &str) -> Vec<String> {
    let doc = Html::parse_document(html);

    extract_node_texts(&doc.root_element())
}

static IGNORE_TAGS: &[&str] = &["head", "a", "button", "img", "input", "script", "style"];

fn extract_node_texts(node: &ElementRef) -> Vec<String> {
    let mut texts: Vec<String> = vec![];

    for child in node.children() {
        match child.value() {
            Node::Text(text) => {
                let s = text.trim();
                if !s.is_empty() {
                    texts.push(s.to_string());
                }
            }
            Node::Element(element) => {
                if IGNORE_TAGS.contains(&element.name()) {
                    continue;
                }

                if let Some(child_element) = ElementRef::wrap(child) {
                    texts.par_extend(extract_node_texts(&child_element));
                }
            }
            _ => {}
        }
    }

    texts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_texts_from_html() {
        assert_eq!(
            extract_texts_from_html(
                r##"
                <!DOCTYPE html>
                <html lang="en">
                    <head>
                        <meta content="text/html; charset=utf-8" http-equiv="content-type"/>
                        <link rel="preconnect" href="https://r.bing.com" />
                        <title>Bing</title>
                    </head>
                    <body>
                        <span>Hello</span> to <span>Bing</span>
                        <a href="https://bing.com">Bing</a>
                        <img src="https://bing.com/rp/kAwiv9gc4HPfHSU3xUQp2Xqm5wA.png"></img>
                        <input type="text"></input>
                        <button>Search</button>
                        <script type="text/javascript" src="https://r.bing.com/rp/1xoNdSsiOsQnITofHebvgzOZ2HM.br.js"></script>
                        <script type="text/javascript">//<![CDATA[_G.HT=new Date;//]]></script>
                    </body>
                </html>
                "##
            ),
            vec!["Hello".to_string(), "to".to_string(), "Bing".to_string(),]
        );
    }
}
