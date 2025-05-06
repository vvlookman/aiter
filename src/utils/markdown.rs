use std::sync::LazyLock;

use regex::Regex;

pub fn extract_code_block(s: &str) -> String {
    let s = REGEX_XML_TAG.replace_all(s, "");
    let s = REGEX_CODE_BLOCK_START.replace(&s, "");
    let s = REGEX_CODE_BLOCK_END.replace(&s, "");

    s.trim().to_string()
}

static REGEX_CODE_BLOCK_START: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:([\s\S]*?))(\s*```.*\n)([\s\S]*?)").expect("CODE_BLOCK_START regex is invalid")
});
static REGEX_CODE_BLOCK_END: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s*```[\s\S]*").expect("CODE_BLOCK_END regex is invalid"));
static REGEX_XML_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<[^>]+>[\s\S]*?<\/[^>]+>").expect("XML_TAG regex is invalid"));

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

        if let Ok(json) = serde_json::from_str::<HashMap<&str, &str>>(&extract_code_block(
            r#"
<think>
I am thinking...
</think>

{
    "foo": "bar"
}
"#,
        )) {
            assert_eq!(json.get("foo"), Some(&"bar"));
        } else {
            assert!(false);
        }

        if let Ok(json) = serde_json::from_str::<HashMap<&str, &str>>(&extract_code_block(
            r#"
<think>
I am thinking...
</think>

```json
{
    "foo": "bar"
}
```

Some more text
"#,
        )) {
            assert_eq!(json.get("foo"), Some(&"bar"));
        } else {
            assert!(false);
        }
    }
}
