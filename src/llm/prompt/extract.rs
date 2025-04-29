pub fn make_extract_questions_prompt(text: &str, refers: &[String]) -> String {
    let mut prompt = format!(
        r#"
根据下面的内容，生成所有可能引发这个回答的问题。注意在问题中明确表达所有对象，不要使用指代词，要使问题在没有上下文的时候也能被准确理解。结果以标准的 JSON 数组格式返回，其中每个数组项是一个问题：
```
{}
```

返回的 JSON 格式示例如下：
```
["<question_1>", "<question_2>"]
```
"#,
        text.replace("```", "")
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
- 每个问题的描述尽可能保持简洁明了。
- 不要包含任何额外的解释或文本，仅返回 JSON 数据。
- 确保返回的结果是合法的 JSON 格式。
"#,
    );

    prompt
}

pub fn make_extract_implicit_knowledges_prompt(text: &str, refers: &[String]) -> String {
    let mut prompt = format!(
        r#"
从下面的内容中提取所有隐含的知识点，并以每个知识点作为回答，生成各种可能引发这个回答的问题。注意在问题中明确表达所有对象，不要使用指代词，要使问题在没有上下文的时候也能被准确理解。结果以标准的 JSON 对象格式返回，JSON 对象的键为知识点的详细内容，JSON 对象的值为引发知识点的问题数组，每个数组项是一个问题：
```
{}
```

返回的 JSON 格式示例如下：
```
{{"<implicit>": ["<trigger>", ...], "<implicit>": ["<trigger>", ...]}}
```
"#,
        text.replace("```", "")
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
- 对数值类的描述尽可能进行理解并统计。
- 每个问题的描述尽可能保持简洁明了。
- 不要包含任何额外的解释或文本，仅返回 JSON 数据。
- 确保返回的结果是合法的 JSON 格式。
"#,
    );

    prompt
}
