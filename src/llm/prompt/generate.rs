pub fn make_answer_by_candidates_prompt(
    question: &str,
    history_questions: &[String],
    contents: &[String],
    strict: bool,
) -> String {
    let mut prompt = format!(
        r#"
以下是根据用户的问题查询到可能相关的内容，基于这些内容进行回答：
```
{}
```
"#,
        contents
            .iter()
            .map(|s| s.replace("```", ""))
            .collect::<Vec<_>>()
            .join("\n\n")
    );

    prompt.push_str(&format!(
        r#"
在回答时，注意以下几点：
- 并非所有的内容都与问题密切相关，你需要结合问题，对内容进行甄别、筛选。
- 如果所有可能的内容都和问题无关，{}。
- 如果内容中提及相对时间，注意进行正确的理解和换算。
- 除非用户另有要求，否则回答的语言需要和用户提问的语言保持一致。
{}
"#,
        if strict {
            "那么告诉用户没有查询到相关的内容"
        } else {
            "那么自行回答该问题"
        },
        if strict {
            ""
        } else {
            "- 对于客观类的问题，如果回答内容非常简短，可以适当补充一点相关的客观信息，以丰富内容。"
        }
    ));

    prompt.push_str(&format!(
        r#"
用户的问题是：
```
{}
```
"#,
        question.replace("```", "")
    ));

    if !history_questions.is_empty() {
        prompt.push_str(&format!(
            r#"
注意结合用户的历史消息正确地理解问题。下面是用户最近发送的内容，按时间从早到晚排序：
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

    prompt
}

pub fn make_no_answer_prompt(question: &str) -> String {
    let prompt = format!(
        r#"
用户提了下面的问题，请告诉用户没有查询到相关的内容：
```
{}
```

在回答时，注意：
- 除非用户另有要求，否则回答的语言需要和用户提问的语言保持一致。
"#,
        question.replace("```", "")
    );

    prompt
}

pub fn make_fix_json_prompt(json: &str) -> String {
    let prompt = format!(
        r#"
下面这个 JSON 数据存在格式问题无法解析，尝试在保留数据结构不变的前提下修复数据格式，使其可以被正常解析：
```
{}
```

在处理时，注意以下几点：
- 不要包含任何额外的解释或文本，仅返回 JSON 数据。
- 确保返回的结果是合法的 JSON 格式。
"#,
        json
    );

    prompt
}
