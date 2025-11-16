// GDPR Compliance Module
//
// Implements GDPR compliance features including:
// - Right to Access (Article 15)
// - Right to Rectification (Article 16)
// - Right to Erasure (Article 17)
// - Right to Restrict Processing (Article 18)
// - Data Breach Notification (Articles 33-34)
// - Consent Management
// - Privacy by Design

pub mod types;
pub mod export;
pub mod deletion;
pub mod consent;
pub mod breach;
pub mod anonymization;
pub mod repository;
pub mod service;
pub mod handlers;
pub mod routes;

pub use types::{
    ConsentRecord, ConsentPurpose, ConsentStatus, DataExportFormat,
    DataExportRequest, DataExportResponse, DeletionRequest, DeletionResponse,
    DeletionStatus, BreachNotification, BreachSeverity, BreachStatus,
    ProcessingRestriction, RestrictionReason, PersonalDataCategory,
    RetentionPolicy as GdprRetentionPolicy, AnonymizationMethod,
};

pub use export::{DataExporter, DefaultDataExporter};
pub use deletion::{DataDeleter, DefaultDataDeleter, DeletionResult};
pub use consent::{ConsentManager, DefaultConsentManager, ConsentValidator};
pub use breach::{BreachNotifier, DefaultBreachNotifier, BreachDetector};
pub use anonymization::{DataAnonymizer, anonymize_record};
pub use repository::{GdprRepository, InMemoryGdprRepository};
pub use service::GdprService;
pub use routes::create_gdpr_router;
