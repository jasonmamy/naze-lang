use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::traits::*;
use crate::types::*;

pub struct SqliteStorage {
    conn: Mutex<Connection>,
    data_dir: Option<String>,
}

impl SqliteStorage {
    pub fn open(data_dir: &str) -> Result<Self, StorageError> {
        std::fs::create_dir_all(data_dir).map_err(|e| StorageError::new(e.to_string()))?;
        let db_path = std::path::Path::new(data_dir).join("discovery.db");
        let conn = Connection::open(db_path).map_err(|e| StorageError::new(e.to_string()))?;
        let storage = Self {
            conn: Mutex::new(conn),
            data_dir: Some(data_dir.to_string()),
        };
        storage.init_schema()?;
        Ok(storage)
    }

    pub fn open_in_memory() -> Result<Self, StorageError> {
        let conn =
            Connection::open_in_memory().map_err(|e| StorageError::new(e.to_string()))?;
        let storage = Self {
            conn: Mutex::new(conn),
            data_dir: None,
        };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            -- Storage Layer
            CREATE TABLE IF NOT EXISTS services (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                domain          TEXT NOT NULL,
                name            TEXT NOT NULL,
                version         TEXT NOT NULL DEFAULT '0.1.0',
                manifest_hash   TEXT NOT NULL,
                manifest_json   TEXT NOT NULL,
                headless_hash   TEXT,
                visibility      TEXT NOT NULL DEFAULT 'public',
                publisher       TEXT,
                active          INTEGER NOT NULL DEFAULT 1,
                registered_at   TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
                last_activity   TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(domain, name)
            );

            CREATE TABLE IF NOT EXISTS capabilities (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                service_id      INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
                kind            TEXT NOT NULL,
                name            TEXT NOT NULL,
                value_type      TEXT,
                metadata        TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_cap_kind_name ON capabilities(kind, name);
            CREATE INDEX IF NOT EXISTS idx_cap_service ON capabilities(service_id);

            CREATE TABLE IF NOT EXISTS trust_profiles (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                name            TEXT NOT NULL UNIQUE,
                weights         TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS trust_scores (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                service_id      INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
                profile_name    TEXT NOT NULL,
                score           REAL NOT NULL,
                base_score      REAL NOT NULL,
                adjustment      REAL NOT NULL DEFAULT 0.0,
                breakdown       TEXT NOT NULL,
                scorer          TEXT NOT NULL DEFAULT 'simple-v1',
                computed_at     TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(service_id, profile_name)
            );

            -- Observation Layer
            CREATE TABLE IF NOT EXISTS observations (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                observation_id  TEXT,
                kind            TEXT NOT NULL,
                service_id      INTEGER REFERENCES services(id),
                agent_id        TEXT,
                payload         TEXT NOT NULL,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(observation_id)
            );
            CREATE INDEX IF NOT EXISTS idx_obs_kind ON observations(kind);
            CREATE INDEX IF NOT EXISTS idx_obs_service ON observations(service_id);

            CREATE TABLE IF NOT EXISTS compositions (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                service_refs    TEXT NOT NULL,
                frequency       INTEGER NOT NULL DEFAULT 1,
                first_seen      TEXT NOT NULL DEFAULT (datetime('now')),
                last_seen       TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(service_refs)
            );

            -- Federation Layer
            CREATE TABLE IF NOT EXISTS peers (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                url             TEXT NOT NULL UNIQUE,
                name            TEXT,
                trust_profile   TEXT,
                last_sync       TEXT,
                active          INTEGER NOT NULL DEFAULT 1,
                added_at        TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- Provenance
            CREATE TABLE IF NOT EXISTS provenance (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                service_id      INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
                source_domain   TEXT NOT NULL,
                source_name     TEXT NOT NULL,
                UNIQUE(service_id, source_domain, source_name)
            );

            -- Version History
            CREATE TABLE IF NOT EXISTS service_versions (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                service_id      INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
                version         TEXT NOT NULL,
                manifest_hash   TEXT NOT NULL,
                headless_hash   TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(service_id, version)
            );

            -- Pattern Templates
            CREATE TABLE IF NOT EXISTS pattern_templates (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                composition_id  INTEGER NOT NULL REFERENCES compositions(id),
                name            TEXT,
                description     TEXT,
                promoted_at     TEXT NOT NULL DEFAULT (datetime('now')),
                discovery_count INTEGER NOT NULL DEFAULT 0
            );
            ",
        )
        .map_err(|e| StorageError::new(e.to_string()))?;

        // Seed default trust profiles
        self.seed_profiles(&conn)?;

        Ok(())
    }

    fn seed_profiles(&self, conn: &Connection) -> Result<(), StorageError> {
        let profiles = [
            ("default", r#"{"external_domains":0.25,"personal_data":0.25,"device_apis":0.25,"data_flow":0.25}"#),
            ("healthcare", r#"{"external_domains":0.20,"personal_data":0.40,"device_apis":0.10,"data_flow":0.30}"#),
            ("ecommerce", r#"{"external_domains":0.30,"personal_data":0.30,"device_apis":0.20,"data_flow":0.20}"#),
            ("iot", r#"{"external_domains":0.20,"personal_data":0.30,"device_apis":0.10,"data_flow":0.40}"#),
            ("finance", r#"{"external_domains":0.20,"personal_data":0.35,"device_apis":0.10,"data_flow":0.35}"#),
            ("education", r#"{"external_domains":0.20,"personal_data":0.50,"device_apis":0.10,"data_flow":0.20}"#),
        ];

        for (name, weights) in &profiles {
            conn.execute(
                "INSERT OR IGNORE INTO trust_profiles (name, weights) VALUES (?1, ?2)",
                params![name, weights],
            )
            .map_err(|e| StorageError::new(e.to_string()))?;
        }

        Ok(())
    }

    /// Get the internal service ID for a ServiceRef. Returns None if not found.
    fn service_id(&self, conn: &Connection, sref: &ServiceRef) -> Result<Option<i64>, StorageError> {
        let mut stmt = conn
            .prepare("SELECT id FROM services WHERE domain = ?1 AND name = ?2")
            .map_err(|e| StorageError::new(e.to_string()))?;
        let mut rows = stmt
            .query(params![sref.domain, sref.name])
            .map_err(|e| StorageError::new(e.to_string()))?;
        match rows.next().map_err(|e| StorageError::new(e.to_string()))? {
            Some(row) => Ok(Some(row.get(0).map_err(|e| StorageError::new(e.to_string()))?)),
            None => Ok(None),
        }
    }

    /// Get service ID, returning error if not found.
    fn require_service_id(&self, conn: &Connection, sref: &ServiceRef) -> Result<i64, StorageError> {
        self.service_id(conn, sref)?
            .ok_or_else(|| StorageError::new(format!("service not found: {}:{}", sref.domain, sref.name)))
    }

    /// Store headless binary on filesystem if data_dir is set.
    fn store_headless(&self, domain: &str, name: &str, data: &[u8]) -> Result<String, StorageError> {
        if let Some(ref dir) = self.data_dir {
            let binaries_dir = std::path::Path::new(dir).join("headless");
            std::fs::create_dir_all(&binaries_dir).map_err(|e| StorageError::new(e.to_string()))?;
            let safe_name = format!("{}_{}", domain.replace('.', "_"), name.replace(' ', "_"));
            let path = binaries_dir.join(&safe_name);
            std::fs::write(&path, data).map_err(|e| StorageError::new(e.to_string()))?;
            Ok(path.to_string_lossy().to_string())
        } else {
            // In-memory mode: no filesystem storage
            Ok(String::new())
        }
    }
}

impl StorageBackend for SqliteStorage {
    fn upsert_service(&self, service: &ServiceRecord) -> Result<ServiceRef, StorageError> {
        let conn = self.conn.lock().unwrap();
        let manifest_str = serde_json::to_string(&service.manifest)
            .map_err(|e| StorageError::new(e.to_string()))?;

        // Store headless binary if present
        if let Some(ref data) = service.headless {
            self.store_headless(&service.domain, &service.name, data)?;
        }

        conn.execute(
            "INSERT INTO services (domain, name, version, manifest_hash, manifest_json, headless_hash, visibility, publisher)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(domain, name) DO UPDATE SET
                version = excluded.version,
                manifest_hash = excluded.manifest_hash,
                manifest_json = excluded.manifest_json,
                headless_hash = excluded.headless_hash,
                visibility = excluded.visibility,
                publisher = excluded.publisher,
                updated_at = datetime('now'),
                last_activity = datetime('now')",
            params![
                service.domain,
                service.name,
                service.version,
                service.manifest_hash,
                manifest_str,
                service.headless_hash,
                service.visibility,
                service.publisher,
            ],
        )
        .map_err(|e| StorageError::new(e.to_string()))?;

        Ok(ServiceRef {
            domain: service.domain.clone(),
            name: service.name.clone(),
        })
    }

    fn get_service(&self, sref: &ServiceRef) -> Result<Option<ServiceRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT domain, name, version, manifest_hash, manifest_json, headless_hash,
                        visibility, publisher, active, registered_at, updated_at, last_activity
                 FROM services WHERE domain = ?1 AND name = ?2",
            )
            .map_err(|e| StorageError::new(e.to_string()))?;

        let mut rows = stmt
            .query(params![sref.domain, sref.name])
            .map_err(|e| StorageError::new(e.to_string()))?;

        match rows.next().map_err(|e| StorageError::new(e.to_string()))? {
            Some(row) => {
                let manifest_str: String = row.get(4).map_err(|e| StorageError::new(e.to_string()))?;
                let manifest: serde_json::Value = serde_json::from_str(&manifest_str)
                    .map_err(|e| StorageError::new(e.to_string()))?;
                let active_int: i32 = row.get(8).map_err(|e| StorageError::new(e.to_string()))?;

                Ok(Some(ServiceRecord {
                    domain: row.get(0).map_err(|e| StorageError::new(e.to_string()))?,
                    name: row.get(1).map_err(|e| StorageError::new(e.to_string()))?,
                    version: row.get(2).map_err(|e| StorageError::new(e.to_string()))?,
                    manifest_hash: row.get(3).map_err(|e| StorageError::new(e.to_string()))?,
                    manifest,
                    headless_hash: row.get(5).map_err(|e| StorageError::new(e.to_string()))?,
                    headless: None, // not loaded from DB; fetched separately
                    visibility: row.get(6).map_err(|e| StorageError::new(e.to_string()))?,
                    publisher: row.get(7).map_err(|e| StorageError::new(e.to_string()))?,
                    active: active_int != 0,
                    registered_at: row.get(9).map_err(|e| StorageError::new(e.to_string()))?,
                    updated_at: row.get(10).map_err(|e| StorageError::new(e.to_string()))?,
                    last_activity: row.get(11).map_err(|e| StorageError::new(e.to_string()))?,
                }))
            }
            None => Ok(None),
        }
    }

    fn deactivate_service(&self, sref: &ServiceRef) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE services SET active = 0, updated_at = datetime('now') WHERE domain = ?1 AND name = ?2",
            params![sref.domain, sref.name],
        )
        .map_err(|e| StorageError::new(e.to_string()))?;
        Ok(())
    }

    fn list_services(&self, filter: &ServiceFilter) -> Result<Vec<ServiceRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from(
            "SELECT domain, name, version, manifest_hash, manifest_json, headless_hash,
                    visibility, publisher, active, registered_at, updated_at, last_activity
             FROM services WHERE 1=1",
        );
        if filter.active_only {
            sql.push_str(" AND active = 1");
        }
        if let Some(ref vis) = filter.visibility {
            sql.push_str(&format!(" AND visibility = '{}'", vis.replace('\'', "''")));
        }
        if let Some(ref domain) = filter.domain {
            sql.push_str(&format!(" AND domain = '{}'", domain.replace('\'', "''")));
        }
        sql.push_str(" ORDER BY name");

        let mut stmt = conn.prepare(&sql).map_err(|e| StorageError::new(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                let manifest_str: String = row.get(4)?;
                let manifest: serde_json::Value =
                    serde_json::from_str(&manifest_str).unwrap_or_default();
                let active_int: i32 = row.get(8)?;
                Ok(ServiceRecord {
                    domain: row.get(0)?,
                    name: row.get(1)?,
                    version: row.get(2)?,
                    manifest_hash: row.get(3)?,
                    manifest,
                    headless_hash: row.get(5)?,
                    headless: None,
                    visibility: row.get(6)?,
                    publisher: row.get(7)?,
                    active: active_int != 0,
                    registered_at: row.get(9)?,
                    updated_at: row.get(10)?,
                    last_activity: row.get(11)?,
                })
            })
            .map_err(|e| StorageError::new(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| StorageError::new(e.to_string()))?);
        }
        Ok(result)
    }

    fn replace_capabilities(
        &self,
        sref: &ServiceRef,
        caps: &[Capability],
    ) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let sid = self.require_service_id(&conn, sref)?;

        conn.execute("DELETE FROM capabilities WHERE service_id = ?1", params![sid])
            .map_err(|e| StorageError::new(e.to_string()))?;

        for cap in caps {
            conn.execute(
                "INSERT INTO capabilities (service_id, kind, name, value_type, metadata)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![sid, cap.kind, cap.name, cap.value_type, cap.metadata],
            )
            .map_err(|e| StorageError::new(e.to_string()))?;
        }

        Ok(())
    }

    fn query_capabilities(
        &self,
        matchers: &[CapabilityMatcher],
    ) -> Result<Vec<ServiceRef>, StorageError> {
        if matchers.is_empty() {
            return Ok(Vec::new());
        }

        let conn = self.conn.lock().unwrap();

        // Build INTERSECT query: each matcher produces a set of service_ids,
        // INTERSECT gives us services matching ALL matchers.
        let mut queries = Vec::new();
        for m in matchers {
            let mut conditions = vec![format!("kind = '{}'", m.kind.replace('\'', "''"))];
            if let Some(ref name) = m.name {
                conditions.push(format!("name = '{}'", name.replace('\'', "''")));
            }
            if let Some(ref like) = m.name_like {
                conditions.push(format!("name LIKE '{}'", like.replace('\'', "''")));
            }
            if let Some(ref vt) = m.value_type {
                conditions.push(format!("value_type = '{}'", vt.replace('\'', "''")));
            }
            queries.push(format!(
                "SELECT DISTINCT service_id FROM capabilities WHERE {}",
                conditions.join(" AND ")
            ));
        }

        let sql = format!(
            "SELECT s.domain, s.name FROM services s
             WHERE s.active = 1 AND s.id IN ({})
             ORDER BY s.name",
            queries.join(" INTERSECT ")
        );

        let mut stmt = conn.prepare(&sql).map_err(|e| StorageError::new(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ServiceRef {
                    domain: row.get(0)?,
                    name: row.get(1)?,
                })
            })
            .map_err(|e| StorageError::new(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| StorageError::new(e.to_string()))?);
        }
        Ok(result)
    }

    fn upsert_trust_score(
        &self,
        sref: &ServiceRef,
        profile: &str,
        output: &TrustOutput,
    ) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let sid = self.require_service_id(&conn, sref)?;
        let breakdown_str =
            serde_json::to_string(&output.breakdown).map_err(|e| StorageError::new(e.to_string()))?;

        conn.execute(
            "INSERT INTO trust_scores (service_id, profile_name, score, base_score, adjustment, breakdown, scorer)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(service_id, profile_name) DO UPDATE SET
                score = excluded.score,
                base_score = excluded.base_score,
                adjustment = excluded.adjustment,
                breakdown = excluded.breakdown,
                scorer = excluded.scorer,
                computed_at = datetime('now')",
            params![sid, profile, output.score, output.base_score, output.adjustment, breakdown_str, output.scorer],
        )
        .map_err(|e| StorageError::new(e.to_string()))?;

        Ok(())
    }

    fn get_trust_scores(
        &self,
        sref: &ServiceRef,
    ) -> Result<HashMap<String, TrustOutput>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let sid = match self.service_id(&conn, sref)? {
            Some(id) => id,
            None => return Ok(HashMap::new()),
        };

        let mut stmt = conn
            .prepare(
                "SELECT profile_name, score, base_score, adjustment, breakdown, scorer
                 FROM trust_scores WHERE service_id = ?1",
            )
            .map_err(|e| StorageError::new(e.to_string()))?;

        let rows = stmt
            .query_map(params![sid], |row| {
                let profile: String = row.get(0)?;
                let breakdown_str: String = row.get(4)?;
                let breakdown: HashMap<String, f64> =
                    serde_json::from_str(&breakdown_str).unwrap_or_default();
                Ok((
                    profile,
                    TrustOutput {
                        score: row.get(1)?,
                        base_score: row.get(2)?,
                        adjustment: row.get(3)?,
                        breakdown,
                        scorer: row.get(5)?,
                    },
                ))
            })
            .map_err(|e| StorageError::new(e.to_string()))?;

        let mut result = HashMap::new();
        for row in rows {
            let (k, v) = row.map_err(|e| StorageError::new(e.to_string()))?;
            result.insert(k, v);
        }
        Ok(result)
    }

    fn record_observation(&self, obs: &Observation) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let sid = self.service_id(&conn, &obs.service)?;
        let payload_str =
            serde_json::to_string(&obs.payload).map_err(|e| StorageError::new(e.to_string()))?;

        conn.execute(
            "INSERT OR IGNORE INTO observations (observation_id, kind, service_id, agent_id, payload)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![obs.observation_id, obs.kind, sid, obs.agent_id, payload_str],
        )
        .map_err(|e| StorageError::new(e.to_string()))?;

        // Update last_activity on the service
        if let Some(id) = sid {
            conn.execute(
                "UPDATE services SET last_activity = datetime('now') WHERE id = ?1",
                params![id],
            )
            .map_err(|e| StorageError::new(e.to_string()))?;
        }

        Ok(())
    }

    fn get_observation_signals(
        &self,
        sref: &ServiceRef,
    ) -> Result<ObservationSignals, StorageError> {
        let conn = self.conn.lock().unwrap();
        let sid = match self.service_id(&conn, sref)? {
            Some(id) => id,
            None => return Ok(ObservationSignals::default()),
        };

        let usage_count: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM observations WHERE service_id = ?1 AND kind = 'usage'",
                params![sid],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let discovery_count: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM observations WHERE service_id = ?1 AND kind = 'discovery'",
                params![sid],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let flag_count: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM observations WHERE service_id = ?1 AND kind = 'flag'",
                params![sid],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Get flag reasons
        let mut stmt = conn
            .prepare("SELECT payload FROM observations WHERE service_id = ?1 AND kind = 'flag'")
            .map_err(|e| StorageError::new(e.to_string()))?;
        let flag_reasons: Vec<String> = stmt
            .query_map(params![sid], |row| {
                let payload_str: String = row.get(0)?;
                Ok(payload_str)
            })
            .map_err(|e| StorageError::new(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        // Count compositions this service is part of
        let composition_count: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM compositions WHERE service_refs LIKE ?1",
                params![format!("%{}%", sref.domain)],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let last_activity: Option<String> = conn
            .query_row(
                "SELECT last_activity FROM services WHERE id = ?1",
                params![sid],
                |row| row.get(0),
            )
            .ok();

        // Days since last activity
        let days_since_activity: u64 = conn
            .query_row(
                "SELECT CAST(julianday('now') - julianday(last_activity) AS INTEGER)
                 FROM services WHERE id = ?1",
                params![sid],
                |row| row.get::<_, i64>(0),
            )
            .map(|d| d.max(0) as u64)
            .unwrap_or(0);

        // Source flag count (flags on services in provenance)
        let source_flag_count: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM observations o
                 JOIN services s ON o.service_id = s.id
                 JOIN provenance p ON p.source_domain = s.domain AND p.source_name = s.name
                 JOIN services target ON p.service_id = target.id
                 WHERE target.domain = ?1 AND target.name = ?2 AND o.kind = 'flag'",
                params![sref.domain, sref.name],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(ObservationSignals {
            usage_count,
            discovery_count,
            flag_count,
            flag_reasons,
            composition_count,
            last_activity,
            days_since_activity,
            source_flag_count,
        })
    }

    fn upsert_composition(&self, services: &[ServiceRef]) -> Result<(), StorageError> {
        let mut sorted: Vec<_> = services.to_vec();
        sorted.sort_by(|a, b| (&a.domain, &a.name).cmp(&(&b.domain, &b.name)));
        let refs_json = serde_json::to_string(&sorted).map_err(|e| StorageError::new(e.to_string()))?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO compositions (service_refs, frequency)
             VALUES (?1, 1)
             ON CONFLICT(service_refs) DO UPDATE SET
                frequency = frequency + 1,
                last_seen = datetime('now')",
            params![refs_json],
        )
        .map_err(|e| StorageError::new(e.to_string()))?;

        Ok(())
    }

    fn get_top_patterns(&self, limit: u32) -> Result<Vec<CompositionPattern>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT service_refs, frequency, first_seen, last_seen
                 FROM compositions ORDER BY frequency DESC LIMIT ?1",
            )
            .map_err(|e| StorageError::new(e.to_string()))?;

        let rows = stmt
            .query_map(params![limit], |row| {
                let refs_str: String = row.get(0)?;
                let services: Vec<ServiceRef> =
                    serde_json::from_str(&refs_str).unwrap_or_default();
                Ok(CompositionPattern {
                    services,
                    frequency: row.get::<_, i64>(1)? as u64,
                    first_seen: row.get(2)?,
                    last_seen: row.get(3)?,
                })
            })
            .map_err(|e| StorageError::new(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| StorageError::new(e.to_string()))?);
        }
        Ok(result)
    }

    fn set_provenance(
        &self,
        sref: &ServiceRef,
        sources: &[ServiceRef],
    ) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let sid = self.require_service_id(&conn, sref)?;

        conn.execute("DELETE FROM provenance WHERE service_id = ?1", params![sid])
            .map_err(|e| StorageError::new(e.to_string()))?;

        for source in sources {
            conn.execute(
                "INSERT INTO provenance (service_id, source_domain, source_name)
                 VALUES (?1, ?2, ?3)",
                params![sid, source.domain, source.name],
            )
            .map_err(|e| StorageError::new(e.to_string()))?;
        }

        Ok(())
    }

    fn get_provenance(&self, sref: &ServiceRef) -> Result<Vec<ServiceRef>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let sid = match self.service_id(&conn, sref)? {
            Some(id) => id,
            None => return Ok(Vec::new()),
        };

        let mut stmt = conn
            .prepare("SELECT source_domain, source_name FROM provenance WHERE service_id = ?1")
            .map_err(|e| StorageError::new(e.to_string()))?;

        let rows = stmt
            .query_map(params![sid], |row| {
                Ok(ServiceRef {
                    domain: row.get(0)?,
                    name: row.get(1)?,
                })
            })
            .map_err(|e| StorageError::new(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| StorageError::new(e.to_string()))?);
        }
        Ok(result)
    }

    fn archive_version(&self, sref: &ServiceRef) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let sid = self.require_service_id(&conn, sref)?;

        conn.execute(
            "INSERT OR IGNORE INTO service_versions (service_id, version, manifest_hash, headless_hash)
             SELECT id, version, manifest_hash, headless_hash FROM services WHERE id = ?1",
            params![sid],
        )
        .map_err(|e| StorageError::new(e.to_string()))?;

        Ok(())
    }

    fn list_versions(&self, sref: &ServiceRef) -> Result<Vec<VersionRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let sid = match self.service_id(&conn, sref)? {
            Some(id) => id,
            None => return Ok(Vec::new()),
        };

        let mut stmt = conn
            .prepare(
                "SELECT version, manifest_hash, headless_hash, created_at
                 FROM service_versions WHERE service_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| StorageError::new(e.to_string()))?;

        let rows = stmt
            .query_map(params![sid], |row| {
                Ok(VersionRecord {
                    version: row.get(0)?,
                    manifest_hash: row.get(1)?,
                    headless_hash: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| StorageError::new(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| StorageError::new(e.to_string()))?);
        }
        Ok(result)
    }

    fn add_peer(&self, peer: &PeerRecord) -> Result<String, StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO peers (url, name, trust_profile) VALUES (?1, ?2, ?3)
             ON CONFLICT(url) DO UPDATE SET name = excluded.name, trust_profile = excluded.trust_profile",
            params![peer.url, peer.name, peer.trust_profile],
        )
        .map_err(|e| StorageError::new(e.to_string()))?;

        Ok(peer.url.clone())
    }

    fn list_peers(&self) -> Result<Vec<PeerRecord>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT url, name, trust_profile, last_sync, active FROM peers WHERE active = 1")
            .map_err(|e| StorageError::new(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let active_int: i32 = row.get(4)?;
                Ok(PeerRecord {
                    url: row.get(0)?,
                    name: row.get(1)?,
                    trust_profile: row.get(2)?,
                    last_sync: row.get(3)?,
                    active: active_int != 0,
                })
            })
            .map_err(|e| StorageError::new(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| StorageError::new(e.to_string()))?);
        }
        Ok(result)
    }

    fn remove_peer(&self, peer_url: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE peers SET active = 0 WHERE url = ?1",
            params![peer_url],
        )
        .map_err(|e| StorageError::new(e.to_string()))?;
        Ok(())
    }

    fn list_profiles(&self) -> Result<Vec<TrustProfile>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT name, weights FROM trust_profiles ORDER BY name")
            .map_err(|e| StorageError::new(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let name: String = row.get(0)?;
                let weights_str: String = row.get(1)?;
                let weights: TrustWeights =
                    serde_json::from_str(&weights_str).unwrap_or(TrustWeights {
                        external_domains: 0.25,
                        personal_data: 0.25,
                        device_apis: 0.25,
                        data_flow: 0.25,
                    });
                Ok(TrustProfile { name, weights })
            })
            .map_err(|e| StorageError::new(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| StorageError::new(e.to_string()))?);
        }
        Ok(result)
    }

    fn create_profile(&self, profile: &TrustProfile) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let weights_str =
            serde_json::to_string(&profile.weights).map_err(|e| StorageError::new(e.to_string()))?;

        conn.execute(
            "INSERT INTO trust_profiles (name, weights) VALUES (?1, ?2)
             ON CONFLICT(name) DO UPDATE SET weights = excluded.weights",
            params![profile.name, weights_str],
        )
        .map_err(|e| StorageError::new(e.to_string()))?;

        Ok(())
    }

    fn export_public_services(
        &self,
        since: Option<&str>,
    ) -> Result<Vec<ServiceExport>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from(
            "SELECT domain, name, version, manifest_json, manifest_hash, headless_hash, publisher, updated_at
             FROM services WHERE active = 1 AND visibility = 'public'",
        );
        if let Some(ts) = since {
            sql.push_str(&format!(" AND updated_at > '{}'", ts.replace('\'', "''")));
        }
        sql.push_str(" ORDER BY updated_at");

        let mut stmt = conn.prepare(&sql).map_err(|e| StorageError::new(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                let manifest_str: String = row.get(3)?;
                let manifest: serde_json::Value =
                    serde_json::from_str(&manifest_str).unwrap_or_default();
                Ok(ServiceExport {
                    domain: row.get(0)?,
                    name: row.get(1)?,
                    version: row.get(2)?,
                    manifest,
                    manifest_hash: row.get(4)?,
                    headless_hash: row.get(5)?,
                    publisher: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| StorageError::new(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| StorageError::new(e.to_string()))?);
        }
        Ok(result)
    }

    fn get_stats(&self) -> Result<(u64, u64), StorageError> {
        let conn = self.conn.lock().unwrap();
        let services: u64 = conn
            .query_row("SELECT COUNT(*) FROM services WHERE active = 1", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);
        let peers: u64 = conn
            .query_row("SELECT COUNT(*) FROM peers WHERE active = 1", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);
        Ok((services, peers))
    }

    fn name(&self) -> &str {
        "sqlite-v1"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_storage() -> SqliteStorage {
        SqliteStorage::open_in_memory().unwrap()
    }

    fn bakery_service() -> ServiceRecord {
        ServiceRecord {
            domain: "bakery.example.com".into(),
            name: "Sweet Cakes".into(),
            version: "0.1.0".into(),
            manifest_hash: "abc123".into(),
            manifest: serde_json::json!({
                "name": "Sweet Cakes",
                "state": {"price": {"type": "number"}, "items": {"type": "list"}},
                "server_functions": ["order", "get_menu"],
                "actions": ["add_to_cart"]
            }),
            headless_hash: None,
            headless: None,
            visibility: "public".into(),
            publisher: Some("human:baker@example.com".into()),
            active: true,
            registered_at: None,
            updated_at: None,
            last_activity: None,
        }
    }

    #[test]
    fn test_schema_creates_and_seeds_profiles() {
        let s = test_storage();
        let profiles = s.list_profiles().unwrap();
        assert_eq!(profiles.len(), 6);
        let names: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"default"));
        assert!(names.contains(&"healthcare"));
        assert!(names.contains(&"ecommerce"));
    }

    #[test]
    fn test_upsert_and_get_service() {
        let s = test_storage();
        let sref = s.upsert_service(&bakery_service()).unwrap();
        assert_eq!(sref.domain, "bakery.example.com");

        let got = s.get_service(&sref).unwrap().unwrap();
        assert_eq!(got.name, "Sweet Cakes");
        assert_eq!(got.version, "0.1.0");
        assert!(got.active);
        assert_eq!(got.visibility, "public");
    }

    #[test]
    fn test_upsert_updates_existing() {
        let s = test_storage();
        let mut svc = bakery_service();
        s.upsert_service(&svc).unwrap();

        svc.version = "0.2.0".into();
        svc.manifest_hash = "def456".into();
        let sref = s.upsert_service(&svc).unwrap();

        let got = s.get_service(&sref).unwrap().unwrap();
        assert_eq!(got.version, "0.2.0");
        assert_eq!(got.manifest_hash, "def456");
    }

    #[test]
    fn test_deactivate_service() {
        let s = test_storage();
        let sref = s.upsert_service(&bakery_service()).unwrap();
        s.deactivate_service(&sref).unwrap();

        let got = s.get_service(&sref).unwrap().unwrap();
        assert!(!got.active);
    }

    #[test]
    fn test_list_services_active_filter() {
        let s = test_storage();
        let sref = s.upsert_service(&bakery_service()).unwrap();

        let all = s.list_services(&ServiceFilter::default()).unwrap();
        assert_eq!(all.len(), 1);

        s.deactivate_service(&sref).unwrap();
        let active = s
            .list_services(&ServiceFilter {
                active_only: true,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_capabilities_replace_and_query() {
        let s = test_storage();
        let sref = s.upsert_service(&bakery_service()).unwrap();

        let caps = vec![
            Capability { kind: "state_field".into(), name: "price".into(), value_type: Some("number".into()), metadata: None },
            Capability { kind: "server_function".into(), name: "order".into(), value_type: None, metadata: None },
        ];
        s.replace_capabilities(&sref, &caps).unwrap();

        // Query for state_field:price:number
        let results = s
            .query_capabilities(&[CapabilityMatcher {
                kind: "state_field".into(),
                name: Some("price".into()),
                name_like: None,
                value_type: Some("number".into()),
            }])
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].domain, "bakery.example.com");
    }

    #[test]
    fn test_query_capabilities_intersect() {
        let s = test_storage();
        let sref = s.upsert_service(&bakery_service()).unwrap();
        s.replace_capabilities(
            &sref,
            &[
                Capability { kind: "state_field".into(), name: "price".into(), value_type: Some("number".into()), metadata: None },
                Capability { kind: "server_function".into(), name: "order".into(), value_type: None, metadata: None },
            ],
        )
        .unwrap();

        // Both match
        let results = s
            .query_capabilities(&[
                CapabilityMatcher { kind: "state_field".into(), name: Some("price".into()), name_like: None, value_type: None },
                CapabilityMatcher { kind: "server_function".into(), name: Some("order".into()), name_like: None, value_type: None },
            ])
            .unwrap();
        assert_eq!(results.len(), 1);

        // Second doesn't match
        let results = s
            .query_capabilities(&[
                CapabilityMatcher { kind: "state_field".into(), name: Some("price".into()), name_like: None, value_type: None },
                CapabilityMatcher { kind: "server_function".into(), name: Some("nonexistent".into()), name_like: None, value_type: None },
            ])
            .unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_trust_scores() {
        let s = test_storage();
        let sref = s.upsert_service(&bakery_service()).unwrap();

        let output = TrustOutput {
            score: 0.92,
            base_score: 0.90,
            adjustment: 0.02,
            breakdown: HashMap::from([("external_domains".into(), 0.95), ("personal_data".into(), 1.0)]),
            scorer: "simple-v1".into(),
        };
        s.upsert_trust_score(&sref, "default", &output).unwrap();

        let scores = s.get_trust_scores(&sref).unwrap();
        assert_eq!(scores.len(), 1);
        let default = scores.get("default").unwrap();
        assert!((default.score - 0.92).abs() < 0.001);
        assert!((default.base_score - 0.90).abs() < 0.001);
    }

    #[test]
    fn test_observations_and_signals() {
        let s = test_storage();
        let sref = s.upsert_service(&bakery_service()).unwrap();

        s.record_observation(&Observation {
            observation_id: None,
            kind: "usage".into(),
            service: sref.clone(),
            agent_id: Some("agent:test".into()),
            payload: serde_json::json!({"success": true}),
        })
        .unwrap();

        s.record_observation(&Observation {
            observation_id: None,
            kind: "discovery".into(),
            service: sref.clone(),
            agent_id: None,
            payload: serde_json::json!({}),
        })
        .unwrap();

        let signals = s.get_observation_signals(&sref).unwrap();
        assert_eq!(signals.usage_count, 1);
        assert_eq!(signals.discovery_count, 1);
        assert_eq!(signals.flag_count, 0);
    }

    #[test]
    fn test_compositions() {
        let s = test_storage();
        let a = ServiceRef { domain: "a.com".into(), name: "A".into() };
        let b = ServiceRef { domain: "b.com".into(), name: "B".into() };

        s.upsert_composition(&[a.clone(), b.clone()]).unwrap();
        s.upsert_composition(&[a.clone(), b.clone()]).unwrap();
        s.upsert_composition(&[a, b]).unwrap();

        let patterns = s.get_top_patterns(10).unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].frequency, 3);
    }

    #[test]
    fn test_provenance() {
        let s = test_storage();
        let sref = s.upsert_service(&bakery_service()).unwrap();
        let source = ServiceRef { domain: "flour.com".into(), name: "Flour Mill".into() };

        s.set_provenance(&sref, &[source.clone()]).unwrap();
        let prov = s.get_provenance(&sref).unwrap();
        assert_eq!(prov.len(), 1);
        assert_eq!(prov[0].domain, "flour.com");
    }

    #[test]
    fn test_version_history() {
        let s = test_storage();
        let sref = s.upsert_service(&bakery_service()).unwrap();
        s.archive_version(&sref).unwrap();

        let versions = s.list_versions(&sref).unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].version, "0.1.0");
    }

    #[test]
    fn test_peers() {
        let s = test_storage();
        s.add_peer(&PeerRecord {
            url: "https://peer1.example.com".into(),
            name: Some("Peer 1".into()),
            trust_profile: Some("ecommerce".into()),
            last_sync: None,
            active: true,
        })
        .unwrap();

        let peers = s.list_peers().unwrap();
        assert_eq!(peers.len(), 1);

        s.remove_peer("https://peer1.example.com").unwrap();
        let peers = s.list_peers().unwrap();
        assert_eq!(peers.len(), 0);
    }

    #[test]
    fn test_export_only_public() {
        let s = test_storage();
        s.upsert_service(&bakery_service()).unwrap();

        let mut internal = bakery_service();
        internal.domain = "internal.corp".into();
        internal.name = "Payroll".into();
        internal.visibility = "internal".into();
        internal.manifest_hash = "xyz789".into();
        s.upsert_service(&internal).unwrap();

        let exported = s.export_public_services(None).unwrap();
        assert_eq!(exported.len(), 1);
        assert_eq!(exported[0].domain, "bakery.example.com");
    }

    #[test]
    fn test_stats() {
        let s = test_storage();
        s.upsert_service(&bakery_service()).unwrap();
        let (services, peers) = s.get_stats().unwrap();
        assert_eq!(services, 1);
        assert_eq!(peers, 0);
    }
}
