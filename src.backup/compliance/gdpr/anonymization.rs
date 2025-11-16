// Data Anonymization Service

use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::compliance::error::GdprResult;
use super::types::{AnonymizationMethod, AnonymizedRecord};

/// Data anonymizer
pub struct DataAnonymizer;

impl DataAnonymizer {
    pub fn new() -> Self {
        Self
    }

    /// Anonymize a user ID
    pub fn anonymize_user_id(&self, user_id: &str, method: AnonymizationMethod) -> String {
        match method {
            AnonymizationMethod::Hashing => self.hash_value(user_id),
            AnonymizationMethod::Masking => self.mask_value(user_id),
            AnonymizationMethod::Generalization => "anonymized-user".to_string(),
            AnonymizationMethod::Suppression => "***".to_string(),
        }
    }

    /// Anonymize an email address
    pub fn anonymize_email(&self, email: &str) -> String {
        if let Some(at_pos) = email.find('@') {
            let (local, domain) = email.split_at(at_pos);
            if local.len() > 2 {
                format!("{}***{}", &local[..1], domain)
            } else {
                format!("***{}", domain)
            }
        } else {
            "***@***.com".to_string()
        }
    }

    /// Anonymize an IP address
    pub fn anonymize_ip(&self, ip: &str) -> String {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() == 4 {
            format!("{}.{}.***", parts[0], parts[1])
        } else {
            "***".to_string()
        }
    }

    /// Hash a value using SHA-256
    fn hash_value(&self, value: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        format!("hash-{:x}", hasher.finalize())
    }

    /// Mask a value (show first and last character)
    fn mask_value(&self, value: &str) -> String {
        if value.len() <= 2 {
            "*".repeat(value.len())
        } else {
            let first = &value[..1];
            let last = &value[value.len() - 1..];
            format!("{}***{}", first, last)
        }
    }

    /// Create an anonymized record marker
    pub fn create_marker(&self, original_id: &str, method: AnonymizationMethod, reason: &str) -> AnonymizedRecord {
        AnonymizedRecord {
            original_id: original_id.to_string(),
            anonymized_at: Utc::now(),
            method,
            reason: reason.to_string(),
        }
    }
}

impl Default for DataAnonymizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Anonymize a record (helper function)
pub fn anonymize_record(record_id: &str, method: AnonymizationMethod) -> GdprResult<String> {
    let anonymizer = DataAnonymizer::new();
    Ok(anonymizer.anonymize_user_id(record_id, method))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anonymize_user_id_hashing() {
        let anonymizer = DataAnonymizer::new();
        let result = anonymizer.anonymize_user_id("user-123", AnonymizationMethod::Hashing);
        assert!(result.starts_with("hash-"));
    }

    #[test]
    fn test_anonymize_user_id_masking() {
        let anonymizer = DataAnonymizer::new();
        let result = anonymizer.anonymize_user_id("user-123", AnonymizationMethod::Masking);
        assert_eq!(result, "u***3");
    }

    #[test]
    fn test_anonymize_email() {
        let anonymizer = DataAnonymizer::new();
        let result = anonymizer.anonymize_email("john.doe@example.com");
        assert_eq!(result, "j***@example.com");
    }

    #[test]
    fn test_anonymize_ip() {
        let anonymizer = DataAnonymizer::new();
        let result = anonymizer.anonymize_ip("192.168.1.100");
        assert_eq!(result, "192.168.***");
    }
}
