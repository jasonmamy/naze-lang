use axum::http::HeaderMap;

use crate::traits::IdentityVerifier;
use crate::types::{AuthError, PublisherIdentity};

pub struct ApiKeyVerifier {
    write_key: Option<String>,
    read_key: Option<String>,
}

impl ApiKeyVerifier {
    pub fn new(write_key: Option<String>, read_key: Option<String>) -> Self {
        Self {
            write_key,
            read_key,
        }
    }

    pub fn verify_write(&self, headers: &HeaderMap) -> Result<Option<PublisherIdentity>, AuthError> {
        self.verify_key(headers, &self.write_key)
    }

    pub fn verify_read(&self, headers: &HeaderMap) -> Result<Option<PublisherIdentity>, AuthError> {
        self.verify_key(headers, &self.read_key)
    }

    fn verify_key(
        &self,
        headers: &HeaderMap,
        expected: &Option<String>,
    ) -> Result<Option<PublisherIdentity>, AuthError> {
        let Some(expected_key) = expected else {
            return Ok(None); // no key configured, all pass
        };

        let provided = headers
            .get("x-api-key")
            .and_then(|v| v.to_str().ok());

        match provided {
            Some(key) if key == expected_key => Ok(Some(PublisherIdentity {
                id: format!("apikey:{}", &key[..8.min(key.len())]),
            })),
            Some(_) => Err(AuthError {
                message: "invalid API key".into(),
            }),
            None => Err(AuthError {
                message: "missing X-Api-Key header".into(),
            }),
        }
    }
}

impl IdentityVerifier for ApiKeyVerifier {
    fn verify(&self, headers: &HeaderMap) -> Result<Option<PublisherIdentity>, AuthError> {
        self.verify_write(headers)
    }

    fn name(&self) -> &str {
        "apikey-v1"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_key_configured_passes() {
        let v = ApiKeyVerifier::new(None, None);
        let headers = HeaderMap::new();
        assert!(v.verify(&headers).is_ok());
    }

    #[test]
    fn test_correct_key_passes() {
        let v = ApiKeyVerifier::new(Some("secret123".into()), None);
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", "secret123".parse().unwrap());
        assert!(v.verify(&headers).is_ok());
    }

    #[test]
    fn test_wrong_key_rejected() {
        let v = ApiKeyVerifier::new(Some("secret123".into()), None);
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", "wrong".parse().unwrap());
        assert!(v.verify(&headers).is_err());
    }

    #[test]
    fn test_missing_key_rejected() {
        let v = ApiKeyVerifier::new(Some("secret123".into()), None);
        let headers = HeaderMap::new();
        assert!(v.verify(&headers).is_err());
    }
}
