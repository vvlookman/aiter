pub fn make_summarize_sheet_prompt(text: &str, refers: &[String]) -> String {
    let mut prompt = format!(
        r#"
概括下面的表格数据，表格数据的格式为 CSV：
```
{}
```
"#,
        text.replace("```", ""),
    );

    if !refers.is_empty() {
        prompt.push_str(&format!(
            r#"
这个表格还有以下的背景信息作为参考：
```
{}
```
"#,
            refers
                .iter()
                .map(|s| s.replace("```", ""))
                .collect::<Vec<_>>()
                .join("\n\n")
        ));
    }

    prompt.push_str(
        r#"
在处理时，注意以下几点：
- 尽可能自动识别表格的字段头。
- 对于数值型的字段，尽可能生成其重要的统计值用于总结。
"#,
    );

    prompt
}

pub fn make_summarize_text_prompt(text: &str, refers: &[String]) -> String {
    let mut prompt = format!(
        r#"
概括下面的内容：
```
{}
```
"#,
        text.replace("```", ""),
    );

    if !refers.is_empty() {
        prompt.push_str(&format!(
            r#"
这段内容还有以下的背景信息作为参考：
```
{}
```
"#,
            refers
                .iter()
                .map(|s| s.replace("```", ""))
                .collect::<Vec<_>>()
                .join("\n\n")
        ));
    }

    prompt.push_str(
        r#"
在处理时，注意以下几点：
- 确保概括后内容比原始内容大幅精简。
- 如果原始内容中提及相对时间，注意尽量换算为绝对时间。
- 如果原始内容很少无需概括，则输出` `。
"#,
    );

    prompt
}
