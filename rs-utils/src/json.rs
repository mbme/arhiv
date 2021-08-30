use anyhow::*;
use serde_json::{Map, Value};

pub fn merge_json(o1: Value, o2: Value) -> Result<Value> {
    match (o1, o2) {
        (Value::Object(mut o1), Value::Object(mut o2)) => {
            let mut map = Map::new();

            map.append(&mut o1);
            map.append(&mut o2);

            Ok(Value::Object(map))
        }

        _ => bail!("failed to merge json values"),
    }
}
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_merge_json() -> Result<()> {
        assert_eq!(
            merge_json(
                json!({
                    "test1": 1,
                }),
                json!({
                    "test2": 2,
                })
            )?,
            json!({
                "test1": 1,
                "test2": 2,
            })
        );

        assert_eq!(
            merge_json(
                json!({
                    "test1": 1,
                }),
                json!({
                    "test1": 2,
                })
            )?,
            json!({
                "test1": 2,
            })
        );

        assert!(merge_json(
            json!({
                "test1": 1,
            }),
            Value::Null,
        )
        .is_err());

        Ok(())
    }
}
