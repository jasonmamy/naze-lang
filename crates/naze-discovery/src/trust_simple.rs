use std::collections::HashMap;

use crate::traits::TrustScorer;
use crate::types::*;

pub struct SimpleScorer;

impl SimpleScorer {
    pub fn new() -> Self {
        Self
    }
}

/// PII patterns that reduce personal_data trust signal.
const PII_PATTERNS: &[&str] = &[
    "email", "phone", "address", "ssn", "password", "credit_card",
    "social_security", "date_of_birth", "passport", "license",
];

/// Device API names that reduce device_apis trust signal.
const DEVICE_APIS: &[&str] = &[
    "geolocation", "camera", "microphone", "accelerometer",
    "contacts", "bluetooth", "nfc",
];

impl SimpleScorer {
    /// Extract trust signals from a manifest.
    fn extract_signals(&self, manifest: &serde_json::Value) -> (f64, f64, f64, f64) {
        // 1. External domain count
        let mut domains = std::collections::HashSet::new();
        if let Some(sources) = manifest.get("data_sources").and_then(|s| s.as_array()) {
            for ds in sources {
                if let Some(url) = ds.get("url").and_then(|u| u.as_str()) {
                    // Extract domain from URL
                    if let Some(domain) = url
                        .trim_start_matches("https://")
                        .trim_start_matches("http://")
                        .split('/')
                        .next()
                    {
                        domains.insert(domain.to_string());
                    }
                }
            }
        }
        let external_domains_score = match domains.len() {
            0 => 1.0,
            1 => 0.9,
            2 => 0.75,
            3 => 0.6,
            _ => 0.3_f64.max(1.0 - domains.len() as f64 * 0.15),
        };

        // 2. Personal data score — pattern match state field names against PII
        let mut pii_count = 0;
        if let Some(state) = manifest.get("state").and_then(|s| s.as_object()) {
            for field_name in state.keys() {
                let lower = field_name.to_lowercase();
                if PII_PATTERNS.iter().any(|p| lower.contains(p)) {
                    pii_count += 1;
                }
            }
        }
        let personal_data_score = match pii_count {
            0 => 1.0,
            1 => 0.8,
            2 => 0.6,
            _ => 0.2_f64.max(1.0 - pii_count as f64 * 0.25),
        };

        // 3. Device API score — count device-related data sources
        let mut device_count = 0;
        if let Some(sources) = manifest.get("data_sources").and_then(|s| s.as_array()) {
            for ds in sources {
                let ds_type = ds.get("type").and_then(|t| t.as_str()).unwrap_or("");
                let ds_name = ds.get("name").and_then(|n| n.as_str()).unwrap_or("");
                if ds_type == "device"
                    || DEVICE_APIS
                        .iter()
                        .any(|api| ds_name.to_lowercase().contains(api))
                {
                    device_count += 1;
                }
            }
        }
        let device_apis_score = match device_count {
            0 => 1.0,
            1 => 0.8,
            2 => 0.6,
            _ => 0.3_f64.max(1.0 - device_count as f64 * 0.2),
        };

        // 4. Data flow score — count outbound server functions and external fetches
        let mut outbound_count = 0;
        if let Some(fns) = manifest.get("server_functions").and_then(|f| f.as_array()) {
            outbound_count += fns.len();
        }
        // Also count data sources as outbound (they fetch external data)
        outbound_count += domains.len();

        let data_flow_score = match outbound_count {
            0 => 1.0,
            1..=2 => 0.85,
            3..=4 => 0.7,
            _ => 0.3_f64.max(1.0 - outbound_count as f64 * 0.1),
        };

        (
            external_domains_score,
            personal_data_score,
            device_apis_score,
            data_flow_score,
        )
    }

    /// Compute dynamic adjustment from observation signals.
    fn compute_adjustment(&self, signals: &ObservationSignals) -> f64 {
        let mut adj = 0.0;

        // Usage boost: capped at +0.1
        let usage_boost = (signals.usage_count as f64 * 0.002).min(0.1);
        adj += usage_boost;

        // Discovery boost: capped at +0.05
        let discovery_boost = (signals.discovery_count as f64 * 0.001).min(0.05);
        adj += discovery_boost;

        // Composition boost: capped at +0.05
        let comp_boost = (signals.composition_count as f64 * 0.005).min(0.05);
        adj += comp_boost;

        // Flag penalty: -0.1 per flag, uncapped
        let flag_penalty = signals.flag_count as f64 * 0.1;
        adj -= flag_penalty;

        // Source flag penalty: -0.05 per flagged source
        let source_penalty = signals.source_flag_count as f64 * 0.05;
        adj -= source_penalty;

        // Staleness decay: only if no activity for 90+ days AND low usage
        // Active usage resets the staleness clock
        if signals.days_since_activity > 90 && signals.usage_count < 10 {
            let weeks_stale = (signals.days_since_activity - 90) / 7;
            let staleness = (weeks_stale as f64 * 0.01).min(0.2);
            adj -= staleness;
        }

        adj
    }
}

impl TrustScorer for SimpleScorer {
    fn score(&self, input: &TrustInput) -> TrustOutput {
        let (ext, pii, dev, flow) = self.extract_signals(&input.manifest);
        let w = &input.profile.weights;

        let base_score =
            ext * w.external_domains + pii * w.personal_data + dev * w.device_apis + flow * w.data_flow;

        let adjustment = self.compute_adjustment(&input.signals);
        let score = (base_score + adjustment).clamp(0.0, 1.0);

        let breakdown = HashMap::from([
            ("external_domains".into(), ext),
            ("personal_data".into(), pii),
            ("device_apis".into(), dev),
            ("data_flow".into(), flow),
        ]);

        TrustOutput {
            score,
            base_score,
            adjustment,
            breakdown,
            scorer: "simple-v1".into(),
        }
    }

    fn name(&self) -> &str {
        "simple-v1"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_profile() -> TrustProfile {
        TrustProfile {
            name: "default".into(),
            weights: TrustWeights {
                external_domains: 0.25,
                personal_data: 0.25,
                device_apis: 0.25,
                data_flow: 0.25,
            },
        }
    }

    fn ecommerce_profile() -> TrustProfile {
        TrustProfile {
            name: "ecommerce".into(),
            weights: TrustWeights {
                external_domains: 0.30,
                personal_data: 0.30,
                device_apis: 0.20,
                data_flow: 0.20,
            },
        }
    }

    fn clean_manifest() -> serde_json::Value {
        serde_json::json!({
            "name": "Clean Bakery",
            "state": {"items": {"type": "list"}, "price": {"type": "number"}},
            "server_functions": ["get_menu"],
            "data_sources": [{"name": "menu", "url": "https://bakery.com/api/menu", "type": "fetch"}]
        })
    }

    fn risky_manifest() -> serde_json::Value {
        serde_json::json!({
            "name": "Sketchy Service",
            "state": {
                "email": {"type": "text"},
                "phone": {"type": "text"},
                "ssn": {"type": "text"},
                "credit_card": {"type": "text"}
            },
            "server_functions": ["submit", "track", "report"],
            "data_sources": [
                {"name": "ads", "url": "https://adtracker1.com/pixel", "type": "fetch"},
                {"name": "analytics", "url": "https://analytics2.com/track", "type": "fetch"},
                {"name": "more_ads", "url": "https://adnetwork3.com/beacon", "type": "fetch"}
            ]
        })
    }

    #[test]
    fn test_clean_manifest_scores_high() {
        let scorer = SimpleScorer::new();
        let output = scorer.score(&TrustInput {
            manifest: clean_manifest(),
            profile: default_profile(),
            signals: ObservationSignals::default(),
        });
        assert!(output.score > 0.8, "clean manifest should score > 0.8, got {}", output.score);
    }

    #[test]
    fn test_risky_manifest_scores_low() {
        let scorer = SimpleScorer::new();
        let output = scorer.score(&TrustInput {
            manifest: risky_manifest(),
            profile: default_profile(),
            signals: ObservationSignals::default(),
        });
        assert!(output.score < 0.6, "risky manifest should score < 0.6, got {}", output.score);
    }

    #[test]
    fn test_different_profiles_different_scores() {
        let scorer = SimpleScorer::new();
        let manifest = risky_manifest();

        let default_out = scorer.score(&TrustInput {
            manifest: manifest.clone(),
            profile: default_profile(),
            signals: ObservationSignals::default(),
        });
        let ecommerce_out = scorer.score(&TrustInput {
            manifest,
            profile: ecommerce_profile(),
            signals: ObservationSignals::default(),
        });

        // Scores should differ because weights differ
        assert!(
            (default_out.score - ecommerce_out.score).abs() > 0.001,
            "different profiles should produce different scores"
        );
    }

    #[test]
    fn test_usage_boost() {
        let scorer = SimpleScorer::new();
        let no_usage = scorer.score(&TrustInput {
            manifest: clean_manifest(),
            profile: default_profile(),
            signals: ObservationSignals::default(),
        });

        let with_usage = scorer.score(&TrustInput {
            manifest: clean_manifest(),
            profile: default_profile(),
            signals: ObservationSignals {
                usage_count: 100,
                ..Default::default()
            },
        });

        assert!(with_usage.score > no_usage.score, "usage should boost score");
        assert!(with_usage.adjustment > 0.0);
    }

    #[test]
    fn test_flag_penalty() {
        let scorer = SimpleScorer::new();
        let no_flags = scorer.score(&TrustInput {
            manifest: clean_manifest(),
            profile: default_profile(),
            signals: ObservationSignals::default(),
        });

        let with_flags = scorer.score(&TrustInput {
            manifest: clean_manifest(),
            profile: default_profile(),
            signals: ObservationSignals {
                flag_count: 3,
                ..Default::default()
            },
        });

        assert!(with_flags.score < no_flags.score, "flags should reduce score");
        assert!(with_flags.adjustment < 0.0);
    }

    #[test]
    fn test_staleness_only_when_inactive() {
        let scorer = SimpleScorer::new();

        // Old but heavily used → no decay
        let active_old = scorer.score(&TrustInput {
            manifest: clean_manifest(),
            profile: default_profile(),
            signals: ObservationSignals {
                days_since_activity: 180,
                usage_count: 100, // active
                ..Default::default()
            },
        });

        // Old and unused → decay
        let inactive_old = scorer.score(&TrustInput {
            manifest: clean_manifest(),
            profile: default_profile(),
            signals: ObservationSignals {
                days_since_activity: 180,
                usage_count: 0, // inactive
                ..Default::default()
            },
        });

        assert!(
            active_old.score > inactive_old.score,
            "active usage should prevent staleness decay"
        );
    }

    #[test]
    fn test_source_flag_penalty() {
        let scorer = SimpleScorer::new();
        let output = scorer.score(&TrustInput {
            manifest: clean_manifest(),
            profile: default_profile(),
            signals: ObservationSignals {
                source_flag_count: 2,
                ..Default::default()
            },
        });
        assert!(output.adjustment < 0.0, "source flags should reduce adjustment");
    }

    #[test]
    fn test_score_clamped_to_range() {
        let scorer = SimpleScorer::new();
        let output = scorer.score(&TrustInput {
            manifest: clean_manifest(),
            profile: default_profile(),
            signals: ObservationSignals {
                flag_count: 100, // massive penalty
                ..Default::default()
            },
        });
        assert!(output.score >= 0.0, "score should not go below 0");
        assert!(output.score <= 1.0, "score should not exceed 1");
    }
}
