use crate::traits::{CapabilityMatcherTrait, StorageBackend};
use crate::types::*;

pub struct SqlMatcher;

impl SqlMatcher {
    pub fn new() -> Self {
        Self
    }
}

impl CapabilityMatcherTrait for SqlMatcher {
    fn search(
        &self,
        query: &CapabilityQuery,
        storage: &dyn StorageBackend,
    ) -> Vec<MatchResult> {
        if query.require.is_empty() {
            return Vec::new();
        }

        // Use storage's query_capabilities for the require matchers (AND semantics)
        let matching_refs = match storage.query_capabilities(&query.require) {
            Ok(refs) => refs,
            Err(_) => return Vec::new(),
        };

        // For each matching service, build the MatchResult
        let mut results = Vec::new();
        for sref in &matching_refs {
            // Get trust score for ranking
            let trust_score = storage
                .get_trust_scores(sref)
                .ok()
                .and_then(|scores| {
                    let profile = query.trust_profile.as_deref().unwrap_or("default");
                    scores.get(profile).map(|t| t.score)
                })
                .unwrap_or(0.5); // default if no score computed yet

            // Apply min_trust filter
            if let Some(min) = query.min_trust {
                if trust_score < min {
                    continue;
                }
            }

            // Count preferred matches (how many prefer matchers this service satisfies)
            let preferred_matches = if let Some(ref prefer) = query.prefer {
                let mut count = 0u32;
                for pm in prefer {
                    let matches = storage
                        .query_capabilities(&[pm.clone()])
                        .unwrap_or_default();
                    if matches.iter().any(|r| r == sref) {
                        count += 1;
                    }
                }
                count
            } else {
                0
            };

            // Build matched capabilities list from the require matchers
            let matched_capabilities: Vec<MatchedCapability> = query
                .require
                .iter()
                .map(|m| MatchedCapability {
                    kind: m.kind.clone(),
                    name: m.name.clone().or_else(|| m.name_like.clone()).unwrap_or_default(),
                    value_type: m.value_type.clone(),
                })
                .collect();

            results.push(MatchResult {
                service: sref.clone(),
                matched_capabilities,
                preferred_matches,
            });
        }

        // Sort by trust_score * (1 + 0.1 * preferred_matches), descending
        // Re-fetch scores for sorting (could cache, but keep it simple)
        results.sort_by(|a, b| {
            let score_a = storage
                .get_trust_scores(&a.service)
                .ok()
                .and_then(|s| {
                    let p = query.trust_profile.as_deref().unwrap_or("default");
                    s.get(p).map(|t| t.score)
                })
                .unwrap_or(0.5)
                * (1.0 + 0.1 * a.preferred_matches as f64);
            let score_b = storage
                .get_trust_scores(&b.service)
                .ok()
                .and_then(|s| {
                    let p = query.trust_profile.as_deref().unwrap_or("default");
                    s.get(p).map(|t| t.score)
                })
                .unwrap_or(0.5)
                * (1.0 + 0.1 * b.preferred_matches as f64);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit
        let limit = query.limit.unwrap_or(20) as usize;
        results.truncate(limit);

        results
    }

    fn name(&self) -> &str {
        "sql-v1"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage_sqlite::SqliteStorage;

    fn setup() -> (SqliteStorage, ServiceRef, ServiceRef) {
        let s = SqliteStorage::open_in_memory().unwrap();

        let bakery = ServiceRecord {
            domain: "bakery.com".into(),
            name: "Bakery".into(),
            version: "0.1.0".into(),
            manifest_hash: "h1".into(),
            manifest: serde_json::json!({}),
            headless_hash: None,
            headless: None,
            visibility: "public".into(),
            publisher: None,
            active: true,
            registered_at: None,
            updated_at: None,
            last_activity: None,
        };
        let venue = ServiceRecord {
            domain: "venue.com".into(),
            name: "Venue".into(),
            version: "0.1.0".into(),
            manifest_hash: "h2".into(),
            manifest: serde_json::json!({}),
            headless_hash: None,
            headless: None,
            visibility: "public".into(),
            publisher: None,
            active: true,
            registered_at: None,
            updated_at: None,
            last_activity: None,
        };

        let bref = s.upsert_service(&bakery).unwrap();
        let vref = s.upsert_service(&venue).unwrap();

        // Bakery: has state:price:number + fn:order
        s.replace_capabilities(&bref, &[
            Capability { kind: "state_field".into(), name: "price".into(), value_type: Some("number".into()), metadata: None },
            Capability { kind: "server_function".into(), name: "order".into(), value_type: None, metadata: None },
            Capability { kind: "state_field".into(), name: "location".into(), value_type: Some("text".into()), metadata: None },
        ]).unwrap();

        // Venue: has state:capacity:number + fn:book
        s.replace_capabilities(&vref, &[
            Capability { kind: "state_field".into(), name: "capacity".into(), value_type: Some("number".into()), metadata: None },
            Capability { kind: "server_function".into(), name: "book".into(), value_type: None, metadata: None },
            Capability { kind: "state_field".into(), name: "location".into(), value_type: Some("text".into()), metadata: None },
        ]).unwrap();

        (s, bref, vref)
    }

    #[test]
    fn test_search_single_require() {
        let (s, _, _) = setup();
        let matcher = SqlMatcher::new();
        let results = matcher.search(
            &CapabilityQuery {
                require: vec![CapabilityMatcher {
                    kind: "server_function".into(),
                    name: Some("order".into()),
                    name_like: None,
                    value_type: None,
                }],
                prefer: None,
                trust_profile: None,
                min_trust: None,
                limit: None,
            },
            &s,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].service.domain, "bakery.com");
    }

    #[test]
    fn test_search_no_matches() {
        let (s, _, _) = setup();
        let matcher = SqlMatcher::new();
        let results = matcher.search(
            &CapabilityQuery {
                require: vec![CapabilityMatcher {
                    kind: "server_function".into(),
                    name: Some("nonexistent".into()),
                    name_like: None,
                    value_type: None,
                }],
                prefer: None,
                trust_profile: None,
                min_trust: None,
                limit: None,
            },
            &s,
        );
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_multiple_require_and_semantics() {
        let (s, _, _) = setup();
        let matcher = SqlMatcher::new();

        // Require fn:order AND state:price → only bakery
        let results = matcher.search(
            &CapabilityQuery {
                require: vec![
                    CapabilityMatcher { kind: "server_function".into(), name: Some("order".into()), name_like: None, value_type: None },
                    CapabilityMatcher { kind: "state_field".into(), name: Some("price".into()), name_like: None, value_type: None },
                ],
                prefer: None,
                trust_profile: None,
                min_trust: None,
                limit: None,
            },
            &s,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].service.domain, "bakery.com");
    }

    #[test]
    fn test_search_with_prefer() {
        let (s, _, _) = setup();
        let matcher = SqlMatcher::new();

        // Both bakery and venue have location, search by location, prefer fn:order
        let results = matcher.search(
            &CapabilityQuery {
                require: vec![CapabilityMatcher {
                    kind: "state_field".into(),
                    name: Some("location".into()),
                    name_like: None,
                    value_type: None,
                }],
                prefer: Some(vec![CapabilityMatcher {
                    kind: "server_function".into(),
                    name: Some("order".into()),
                    name_like: None,
                    value_type: None,
                }]),
                trust_profile: None,
                min_trust: None,
                limit: None,
            },
            &s,
        );
        assert_eq!(results.len(), 2);
        // Bakery should have preferred_matches=1, venue=0
        let bakery_result = results.iter().find(|r| r.service.domain == "bakery.com").unwrap();
        let venue_result = results.iter().find(|r| r.service.domain == "venue.com").unwrap();
        assert_eq!(bakery_result.preferred_matches, 1);
        assert_eq!(venue_result.preferred_matches, 0);
    }

    #[test]
    fn test_search_with_limit() {
        let (s, _, _) = setup();
        let matcher = SqlMatcher::new();
        let results = matcher.search(
            &CapabilityQuery {
                require: vec![CapabilityMatcher {
                    kind: "state_field".into(),
                    name: Some("location".into()),
                    name_like: None,
                    value_type: None,
                }],
                prefer: None,
                trust_profile: None,
                min_trust: None,
                limit: Some(1),
            },
            &s,
        );
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_name_like_pattern() {
        let (s, _, _) = setup();
        let matcher = SqlMatcher::new();
        let results = matcher.search(
            &CapabilityQuery {
                require: vec![CapabilityMatcher {
                    kind: "server_function".into(),
                    name: None,
                    name_like: Some("%ord%".into()),
                    value_type: None,
                }],
                prefer: None,
                trust_profile: None,
                min_trust: None,
                limit: None,
            },
            &s,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].service.domain, "bakery.com");
    }
}
