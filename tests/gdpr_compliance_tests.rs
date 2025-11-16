// GDPR Compliance Integration Tests

use chrono::Utc;
use llm_cost_ops::compliance::{
    AnonymizationMethod, BreachSeverity, ConsentPurpose, DataAnonymizer, DataExportFormat,
    DataExportRequest, DeletionRequest, GdprService, InMemoryGdprRepository,
    PersonalDataCategory,
};
use std::sync::Arc;

#[tokio::test]
async fn test_data_export_right_to_access() {
    // Article 15 - Right to Access
    let repo = Arc::new(InMemoryGdprRepository::new());
    let service = GdprService::new(repo);

    let request = DataExportRequest {
        user_id: "user-test-123".to_string(),
        organization_id: "org-test-123".to_string(),
        format: DataExportFormat::Json,
        categories: vec![PersonalDataCategory::All],
        requested_at: Utc::now(),
        requested_by: "user-test-123".to_string(),
    };

    let result = service.export_user_data(request).await;
    assert!(result.is_ok(), "Data export should succeed");

    let response = result.unwrap();
    assert_eq!(response.user_id, "user-test-123");
    assert_eq!(response.format, DataExportFormat::Json);
    assert!(!response.data.is_empty(), "Export data should not be empty");
}

#[tokio::test]
async fn test_data_export_csv_format() {
    let repo = Arc::new(InMemoryGdprRepository::new());
    let service = GdprService::new(repo);

    let request = DataExportRequest {
        user_id: "user-csv-test".to_string(),
        organization_id: "org-csv-test".to_string(),
        format: DataExportFormat::Csv,
        categories: vec![PersonalDataCategory::UsageRecords],
        requested_at: Utc::now(),
        requested_by: "admin".to_string(),
    };

    let result = service.export_user_data(request).await;
    assert!(result.is_ok(), "CSV export should succeed");
}

#[tokio::test]
async fn test_data_deletion_right_to_erasure() {
    // Article 17 - Right to Erasure
    let repo = Arc::new(InMemoryGdprRepository::new());
    let service = GdprService::new(repo);

    let request = DeletionRequest {
        user_id: "user-delete-123".to_string(),
        organization_id: "org-delete-123".to_string(),
        categories: vec![PersonalDataCategory::All],
        reason: "User requested deletion".to_string(),
        requested_at: Utc::now(),
        requested_by: "user-delete-123".to_string(),
        retain_audit_log: true,
    };

    let result = service.delete_user_data(request).await;
    assert!(result.is_ok(), "Data deletion should succeed");

    let response = result.unwrap();
    assert_eq!(response.user_id, "user-delete-123");
}

#[tokio::test]
async fn test_consent_management() {
    let repo = Arc::new(InMemoryGdprRepository::new());
    let service = GdprService::new(repo);

    // Record consent
    let consent_result = service
        .record_consent(
            "user-consent-123".to_string(),
            "org-consent-123".to_string(),
            ConsentPurpose::DataProcessing,
            "I consent to data processing for cost analysis".to_string(),
            "1.0".to_string(),
            Some("192.168.1.100".to_string()),
            Some("Mozilla/5.0".to_string()),
        )
        .await;

    assert!(consent_result.is_ok(), "Consent recording should succeed");

    // Check consent
    let has_consent = service
        .has_consent("user-consent-123", &ConsentPurpose::DataProcessing)
        .await;
    assert!(has_consent.is_ok());
    assert!(has_consent.unwrap(), "User should have given consent");

    // Withdraw consent
    let withdraw_result = service
        .withdraw_consent("user-consent-123", ConsentPurpose::DataProcessing)
        .await;
    assert!(withdraw_result.is_ok(), "Consent withdrawal should succeed");

    // Check consent after withdrawal
    let has_consent_after = service
        .has_consent("user-consent-123", &ConsentPurpose::DataProcessing)
        .await;
    assert!(has_consent_after.is_ok());
    assert!(
        !has_consent_after.unwrap(),
        "User should not have consent after withdrawal"
    );
}

#[tokio::test]
async fn test_multiple_consent_purposes() {
    let repo = Arc::new(InMemoryGdprRepository::new());
    let service = GdprService::new(repo);

    // Record consent for multiple purposes
    let purposes = vec![
        ConsentPurpose::DataProcessing,
        ConsentPurpose::Analytics,
        ConsentPurpose::Marketing,
    ];

    for purpose in purposes {
        let result = service
            .record_consent(
                "user-multi-123".to_string(),
                "org-multi-123".to_string(),
                purpose.clone(),
                format!("Consent for {:?}", purpose),
                "1.0".to_string(),
                None,
                None,
            )
            .await;
        assert!(result.is_ok());
    }

    // Get all consents
    let consents = service.get_user_consents("user-multi-123").await;
    assert!(consents.is_ok());
    assert_eq!(consents.unwrap().len(), 3, "Should have 3 consent records");
}

#[tokio::test]
async fn test_data_breach_notification() {
    // Articles 33-34 - Data Breach Notification
    let repo = Arc::new(InMemoryGdprRepository::new());
    let service = GdprService::new(repo);

    let breach = llm_cost_ops::compliance::BreachNotification::new(
        "unauthorized_access".to_string(),
        BreachSeverity::High,
        50,
        500,
        "Unauthorized access to user data detected".to_string(),
    );

    let breach_id = breach.id.clone();

    let result = service.report_breach(breach).await;
    assert!(result.is_ok(), "Breach reporting should succeed");

    // Notify authority
    let authority_result = service.notify_authority(&breach_id).await;
    assert!(
        authority_result.is_ok(),
        "Authority notification should succeed"
    );

    // Notify users
    let users_result = service.notify_users(&breach_id).await;
    assert!(users_result.is_ok(), "User notification should succeed");
}

#[tokio::test]
async fn test_data_anonymization() {
    let anonymizer = DataAnonymizer::new();

    // Test user ID anonymization
    let user_id = "user-12345";
    let hashed = anonymizer.anonymize_user_id(user_id, AnonymizationMethod::Hashing);
    assert!(hashed.starts_with("hash-"));
    assert_ne!(hashed, user_id);

    let masked = anonymizer.anonymize_user_id(user_id, AnonymizationMethod::Masking);
    assert_eq!(masked, "u***5");

    let generalized = anonymizer.anonymize_user_id(user_id, AnonymizationMethod::Generalization);
    assert_eq!(generalized, "anonymized-user");

    let suppressed = anonymizer.anonymize_user_id(user_id, AnonymizationMethod::Suppression);
    assert_eq!(suppressed, "***");

    // Test email anonymization
    let email = "john.doe@example.com";
    let anon_email = anonymizer.anonymize_email(email);
    assert_eq!(anon_email, "j***@example.com");

    // Test IP anonymization
    let ip = "192.168.1.100";
    let anon_ip = anonymizer.anonymize_ip(ip);
    assert_eq!(anon_ip, "192.168.***");
}

#[tokio::test]
async fn test_anonymize_user_data() {
    let repo = Arc::new(InMemoryGdprRepository::new());
    let service = GdprService::new(repo);

    let result = service.anonymize_user_data("user-anon-123").await;
    assert!(result.is_ok(), "Data anonymization should succeed");
}

#[tokio::test]
async fn test_partial_data_deletion() {
    let repo = Arc::new(InMemoryGdprRepository::new());
    let service = GdprService::new(repo);

    let request = DeletionRequest {
        user_id: "user-partial-123".to_string(),
        organization_id: "org-partial-123".to_string(),
        categories: vec![
            PersonalDataCategory::UsageRecords,
            PersonalDataCategory::CostRecords,
        ],
        reason: "User requested partial deletion".to_string(),
        requested_at: Utc::now(),
        requested_by: "user-partial-123".to_string(),
        retain_audit_log: true,
    };

    let result = service.delete_user_data(request).await;
    assert!(result.is_ok(), "Partial deletion should succeed");
}

#[tokio::test]
async fn test_export_specific_categories() {
    let repo = Arc::new(InMemoryGdprRepository::new());
    let service = GdprService::new(repo);

    let request = DataExportRequest {
        user_id: "user-specific-123".to_string(),
        organization_id: "org-specific-123".to_string(),
        format: DataExportFormat::Json,
        categories: vec![
            PersonalDataCategory::UsageRecords,
            PersonalDataCategory::ConsentRecords,
        ],
        requested_at: Utc::now(),
        requested_by: "user-specific-123".to_string(),
    };

    let result = service.export_user_data(request).await;
    assert!(result.is_ok(), "Specific category export should succeed");

    let response = result.unwrap();
    assert_eq!(response.metadata.categories_included.len(), 2);
}

#[tokio::test]
async fn test_breach_severity_levels() {
    let repo = Arc::new(InMemoryGdprRepository::new());
    let service = GdprService::new(repo);

    let severities = vec![
        BreachSeverity::Low,
        BreachSeverity::Medium,
        BreachSeverity::High,
        BreachSeverity::Critical,
    ];

    for severity in severities {
        let breach = llm_cost_ops::compliance::BreachNotification::new(
            format!("{:?}_breach", severity),
            severity,
            10,
            100,
            format!("{:?} severity breach", severity),
        );

        let result = service.report_breach(breach).await;
        assert!(result.is_ok(), "Breach reporting should succeed for all severity levels");
    }
}

#[test]
fn test_consent_purposes() {
    // Test all consent purpose variants
    let purposes = vec![
        ConsentPurpose::DataProcessing,
        ConsentPurpose::Marketing,
        ConsentPurpose::Analytics,
        ConsentPurpose::ThirdPartySharing,
        ConsentPurpose::Custom("custom_purpose".to_string()),
    ];

    for purpose in purposes {
        let serialized = serde_json::to_string(&purpose).unwrap();
        let deserialized: ConsentPurpose = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            format!("{:?}", purpose),
            format!("{:?}", deserialized),
            "Consent purpose should serialize/deserialize correctly"
        );
    }
}

#[test]
fn test_data_categories() {
    // Test all data category variants
    let categories = vec![
        PersonalDataCategory::UsageRecords,
        PersonalDataCategory::CostRecords,
        PersonalDataCategory::ApiKeys,
        PersonalDataCategory::AuditLogs,
        PersonalDataCategory::ConsentRecords,
        PersonalDataCategory::All,
    ];

    for category in categories {
        let serialized = serde_json::to_string(&category).unwrap();
        let deserialized: PersonalDataCategory = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            format!("{:?}", category),
            format!("{:?}", deserialized),
            "Data category should serialize/deserialize correctly"
        );
    }
}
