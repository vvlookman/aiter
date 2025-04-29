use std::sync::LazyLock;

use regex::Regex;

pub fn extract_code_block(text: &str) -> String {
    REGEX_CODE_BLOCK.replace_all(text, "").trim().to_string()
}

static REGEX_CODE_BLOCK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(^\s*```.*\n\s*)|(\s*```\s*$)").unwrap());

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_extract_code_block() {
        assert_eq!(extract_code_block("foobar"), "foobar");

        assert_eq!(
            extract_code_block(
                r#"
```
foobar
```
"#
            ),
            "foobar"
        );

        let json_text = extract_code_block(
            r#"
```json
{
    "foo": "bar"
}
```
"#,
        );
        if let Ok(json) = serde_json::from_str::<HashMap<&str, &str>>(&json_text) {
            assert_eq!(json.get("foo"), Some(&"bar"));
        } else {
            assert!(false);
        }
    }
}
