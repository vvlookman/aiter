use serde_json::Value;

pub fn json_value_to_string(json_value: &Value) -> String {
    match json_value {
        Value::Null => "".to_string(),
        Value::Bool(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_string(),
        Value::Array(arr) => serde_json::to_string(arr).unwrap_or("".to_string()),
        Value::Object(obj) => serde_json::to_string(obj).unwrap_or("".to_string()),
    }
}

pub fn string_to_json_value(s: &str, value_type: &str) -> Value {
    match value_type.to_lowercase().as_str() {
        "boolean" | "bool" => Value::Bool(["true", "t"].contains(&s.to_lowercase().as_str())),
        "float" | "double" => {
            if let Ok(f) = s.parse::<f64>() {
                if let Some(number) = serde_json::Number::from_f64(f) {
                    Value::Number(number)
                } else {
                    Value::Null
                }
            } else {
                Value::Null
            }
        }
        "integer" | "int" => s
            .parse::<i64>()
            .map(|i| Value::Number(serde_json::Number::from(i)))
            .unwrap_or(Value::Null),

        "string" | "text" => Value::String(s.to_string()),
        _ => serde_json::from_str(s).unwrap_or(Value::Null),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Number};

    use super::*;

    #[test]
    fn test_json_value_to_string() {
        assert_eq!(json_value_to_string(&Value::Null), "");
        assert_eq!(json_value_to_string(&Value::Bool(true)), "true");
        assert_eq!(json_value_to_string(&Value::Number(Number::from(1))), "1");
        assert_eq!(
            json_value_to_string(&Value::Number(Number::from_f64(3.14).unwrap())),
            "3.14"
        );
        assert_eq!(
            json_value_to_string(&Value::String("foo".to_string())),
            "foo"
        );

        let arr = vec![
            Value::Number(Number::from(1)),
            Value::Number(Number::from(2)),
        ];
        assert_eq!(json_value_to_string(&Value::Array(arr)), "[1,2]");

        let mut map: Map<String, Value> = Map::new();
        map.insert("foo".to_string(), Value::String("bar".to_string()));
        assert_eq!(
            json_value_to_string(&Value::Object(map)),
            "{\"foo\":\"bar\"}"
        );
    }

    #[test]
    fn test_string_to_json_value() {
        assert_eq!(string_to_json_value("true", "bool"), Value::Bool(true));
        assert_eq!(
            string_to_json_value("1", "int"),
            Value::Number(Number::from(1))
        );
        assert_eq!(
            string_to_json_value("3.14", "float"),
            Value::Number(Number::from_f64(3.14).unwrap())
        );
        assert_eq!(
            string_to_json_value("foo", "string"),
            Value::String("foo".to_string())
        );

        let arr = vec![
            Value::Number(Number::from(1)),
            Value::Number(Number::from(2)),
        ];
        assert_eq!(string_to_json_value("[1,2]", "array"), Value::Array(arr));

        let mut map: Map<String, Value> = Map::new();
        map.insert("foo".to_string(), Value::String("bar".to_string()));
        assert_eq!(
            string_to_json_value("{\"foo\":\"bar\"}", "object"),
            Value::Object(map)
        );
    }
}
