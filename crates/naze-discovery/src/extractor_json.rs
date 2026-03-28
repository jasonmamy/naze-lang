use crate::traits::CapabilityExtractor;
use crate::types::Capability;

pub struct JsonExtractor;

impl JsonExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl CapabilityExtractor for JsonExtractor {
    fn extract(&self, manifest: &serde_json::Value) -> Vec<Capability> {
        let mut caps = Vec::new();

        // State fields → kind="state_field"
        if let Some(state) = manifest.get("state").and_then(|s| s.as_object()) {
            for (name, def) in state {
                caps.push(Capability {
                    kind: "state_field".into(),
                    name: name.clone(),
                    value_type: def
                        .get("type")
                        .and_then(|t| t.as_str())
                        .map(String::from),
                    metadata: Some(serde_json::to_string(def).unwrap_or_default()),
                });
            }
        }

        // Server functions → kind="server_function"
        if let Some(fns) = manifest.get("server_functions") {
            if let Some(arr) = fns.as_array() {
                for f in arr {
                    if let Some(name) = f.as_str() {
                        caps.push(Capability {
                            kind: "server_function".into(),
                            name: name.to_string(),
                            value_type: None,
                            metadata: None,
                        });
                    } else if let Some(obj) = f.as_object() {
                        if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
                            caps.push(Capability {
                                kind: "server_function".into(),
                                name: name.to_string(),
                                value_type: None,
                                metadata: Some(serde_json::to_string(obj).unwrap_or_default()),
                            });
                        }
                    }
                }
            }
        }

        // Actions → kind="action"
        if let Some(actions) = manifest.get("actions") {
            if let Some(arr) = actions.as_array() {
                for a in arr {
                    if let Some(name) = a.as_str() {
                        caps.push(Capability {
                            kind: "action".into(),
                            name: name.to_string(),
                            value_type: None,
                            metadata: None,
                        });
                    } else if let Some(obj) = a.as_object() {
                        if let Some(name) = obj.get("action").and_then(|n| n.as_str()) {
                            caps.push(Capability {
                                kind: "action".into(),
                                name: name.to_string(),
                                value_type: None,
                                metadata: Some(serde_json::to_string(obj).unwrap_or_default()),
                            });
                        }
                    }
                }
            }
        }

        // Data sources → kind="data_source"
        if let Some(sources) = manifest.get("data_sources") {
            if let Some(arr) = sources.as_array() {
                for ds in arr {
                    if let Some(obj) = ds.as_object() {
                        let name = obj
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("unnamed")
                            .to_string();
                        let source_type = obj
                            .get("type")
                            .and_then(|t| t.as_str())
                            .map(String::from);
                        caps.push(Capability {
                            kind: "data_source".into(),
                            name,
                            value_type: source_type,
                            metadata: Some(serde_json::to_string(obj).unwrap_or_default()),
                        });
                    }
                }
            }
        }

        // Models → kind="model_field"
        if let Some(models) = manifest.get("models") {
            if let Some(arr) = models.as_array() {
                for model in arr {
                    if let Some(obj) = model.as_object() {
                        let model_name = obj
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("unknown");
                        if let Some(fields) = obj.get("fields").and_then(|f| f.as_array()) {
                            for field in fields {
                                if let Some(field_obj) = field.as_object() {
                                    let field_name = field_obj
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("unknown");
                                    let field_type = field_obj
                                        .get("type")
                                        .and_then(|t| t.as_str())
                                        .map(String::from);
                                    caps.push(Capability {
                                        kind: "model_field".into(),
                                        name: format!("{}.{}", model_name, field_name),
                                        value_type: field_type,
                                        metadata: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        caps
    }

    fn name(&self) -> &str {
        "json-v1"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_state_fields() {
        let extractor = JsonExtractor::new();
        let manifest = serde_json::json!({
            "state": {
                "price": {"type": "number"},
                "name": {"type": "text"}
            }
        });
        let caps = extractor.extract(&manifest);
        assert_eq!(caps.len(), 2);
        assert!(caps.iter().all(|c| c.kind == "state_field"));
        assert!(caps.iter().any(|c| c.name == "price" && c.value_type.as_deref() == Some("number")));
    }

    #[test]
    fn test_extract_server_functions() {
        let extractor = JsonExtractor::new();
        let manifest = serde_json::json!({
            "server_functions": ["order", "get_menu"]
        });
        let caps = extractor.extract(&manifest);
        assert_eq!(caps.len(), 2);
        assert!(caps.iter().all(|c| c.kind == "server_function"));
    }

    #[test]
    fn test_extract_actions() {
        let extractor = JsonExtractor::new();
        let manifest = serde_json::json!({
            "actions": ["add_to_cart", "checkout"]
        });
        let caps = extractor.extract(&manifest);
        assert_eq!(caps.len(), 2);
        assert!(caps.iter().any(|c| c.name == "add_to_cart"));
    }

    #[test]
    fn test_extract_data_sources() {
        let extractor = JsonExtractor::new();
        let manifest = serde_json::json!({
            "data_sources": [
                {"name": "menu", "url": "https://api.bakery.com/menu", "type": "fetch"}
            ]
        });
        let caps = extractor.extract(&manifest);
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0].kind, "data_source");
        assert_eq!(caps[0].value_type.as_deref(), Some("fetch"));
    }

    #[test]
    fn test_empty_manifest() {
        let extractor = JsonExtractor::new();
        let caps = extractor.extract(&serde_json::json!({}));
        assert!(caps.is_empty());
    }
}
