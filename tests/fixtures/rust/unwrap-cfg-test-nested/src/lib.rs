use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Effect {
    Spawn { workspace_id: String, command: String },
    Kill { session_id: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_serialization_roundtrip() {
        let effects = vec![
            Effect::Spawn {
                workspace_id: "ws-1".to_string(),
                command: "claude".to_string(),
            },
            Effect::Kill {
                session_id: "sess-1".to_string(),
            },
        ];

        for effect in effects {
            let json = serde_json::to_string(&effect).unwrap();
            let parsed: Effect = serde_json::from_str(&json).unwrap();
            assert_eq!(effect, parsed);
        }
    }
}
