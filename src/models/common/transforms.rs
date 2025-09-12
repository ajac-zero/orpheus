use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Transform {
    MiddleOut,
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize() {
        let target = json!(["middle-out"]);

        let value = vec![Transform::MiddleOut];

        let result = serde_json::to_value(value).unwrap();

        assert_eq!(target, result);
    }
}
