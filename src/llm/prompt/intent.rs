pub fn make_extract_queries_prompt(question: &str, history_questions: &[String]) -> String {
    let mut prompt = format!(
        r#"
理解下面用户的指令，提取其中涉及的所有相关查询。注意在每个查询中明确表达所有对象，不要使用指代词，使其在没有上下文的时候也能被准确理解。结果以标准的 JSON 数组格式返回，其中每个数组项是一个相关查询：
```
{}
```

返回的 JSON 格式示例如下：
```
["<query_1>", "<query_2>"]
```
"#,
        question.replace("```", "")
    );

    if !history_questions.is_empty() {
        prompt.push_str(&format!(
            r#"
注意结合历史消息正确地理解用户的指令。下面是用户最近发送的内容，按时间从早到晚排序：
```
{}
```
"#,
            history_questions
                .iter()
                .map(|s| s.replace("```", ""))
                .collect::<Vec<_>>()
                .join("\n\n")
        ));
    }

    prompt.push_str(
        r#"
在处理时，注意以下几点：
- 用最简洁的方式描述查询。
- 去除助词、连词 、介词 、叹词等没有意义的部分。
- 描述数据类查询时，去除数据、信息、数量、结果等没有意义的部分。
- 不要包含任何额外的解释或文本，仅返回 JSON 数据。
- 确保返回的结果是合法的 JSON 格式。
"#,
    );

    prompt
}

pub fn make_simplify_queries_prompt(queries: &[String]) -> String {
    let mut prompt = format!(
        r#"
依次简化下面所有的查询，使每个查询尽可能多地变为更简洁的几种表达方式。然后，将所有的简化结果放到一起，以标准的 JSON 数组格式返回，其中每个数组项是一个简化后的查询：
```
{}
```


返回的 JSON 格式示例如下：
```
["<query_1>", "<query_2>"]
```
"#,
        queries
            .iter()
            .map(|s| s.replace("```", ""))
            .collect::<Vec<_>>()
            .join("\n\n")
    );

    prompt.push_str(
        r#"
在处理每个查询的时候，注意以下几点：
- 尝试用简称、同义词来替代部分内容，生成不同的简化结果。
- 去除助词、连词 、介词 、叹词等没有意义的部分。
- 描述数据类查询时，去除数据、信息、数量、结果等没有意义的部分。
- 不要包含任何额外的解释或文本，仅返回 JSON 数据。
- 确保返回的结果是合法的 JSON 格式。
"#,
    );

    prompt
}
