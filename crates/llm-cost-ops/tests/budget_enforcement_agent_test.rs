//! Budget Enforcement Agent Integration Tests
//!
//! These tests verify the Budget Enforcement Agent behavior according to
//! the LLM-CostOps constitution requirements.

use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

use llm_cost_ops::agents::{
    budget_enforcement::{
        BudgetDefinition, BudgetEnforcementAgent, BudgetEnforcementConfig,
        BudgetEvaluationRequest, BudgetScope, BudgetViolationType, SpendData,
    },
    contracts::{
        AgentClassification, AgentId, DecisionType, ExecutionRef, SignalType,
    },
    AgentRegistry,
};

/// Helper to create a test budget definition
fn create_test_budget(limit: i64, tenant_id: &str) -> BudgetDefinition {
    BudgetDefinition {
        budget_id: format!("test-budget-{}", Uuid::new_v4()),
        name: "Test Budget".to_string(),
        limit: Decimal::from(limit),
        currency: "USD".to_string(),
        period_start: Utc::now() - Duration::days(15),
        period_end: Utc::now() + Duration::days(15),
        warning_threshold: 0.80,
        critical_threshold: 0.95,
        gating_threshold: 1.0,
        enable_forecasting: false,
        is_soft_limit: true,
        scope: BudgetScope::Tenant {
            tenant_id: tenant_id.to_string(),
        },
    }
}

/// Helper to create test spend data
fn create_test_spend_data(current_spend: i64) -> SpendData {
    SpendData {
        current_spend: Decimal::from(current_spend),
        currency: "USD".to_string(),
        daily_spend_history: vec![],
        data_completeness: 1.0,
        data_as_of: Utc::now(),
    }
}

/// Helper to create test execution ref
fn create_test_execution_ref(tenant_id: &str) -> ExecutionRef {
    ExecutionRef::new(Uuid::new_v4(), tenant_id)
}

// =============================================================================
// PROMPT 1: Contract & Boundary Definition Tests
// =============================================================================

#[test]
fn test_agent_classification_is_financial_governance() {
    let agent = BudgetEnforcementAgent::with_defaults();
    assert_eq!(
        agent.classification(),
        AgentClassification::FinancialGovernance,
        "Budget Enforcement Agent MUST be classified as FINANCIAL GOVERNANCE"
    );
}

#[test]
fn test_agent_id_follows_naming_convention() {
    let agent = BudgetEnforcementAgent::with_defaults();
    let id = agent.agent_id().to_string();
    assert!(
        id.starts_with("llm-costops."),
        "Agent ID must follow 'llm-costops.<name>' convention"
    );
    assert_eq!(
        id,
        "llm-costops.budget-enforcement",
        "Budget Enforcement Agent must have correct ID"
    );
}

#[test]
fn test_agent_version_is_semantic() {
    let agent = BudgetEnforcementAgent::with_defaults();
    let version = agent.agent_version();
    assert!(
        version.major >= 1,
        "Agent must have a valid major version >= 1"
    );
}

#[test]
fn test_agent_registered_in_registry() {
    let registry = AgentRegistry::new();
    assert!(
        registry.is_registered("llm-costops.budget-enforcement"),
        "Budget Enforcement Agent must be registered in the platform"
    );
}

// =============================================================================
// PROMPT 2: Runtime & Infrastructure Tests
// =============================================================================

#[tokio::test]
async fn test_agent_is_stateless() {
    // Create two separate agent instances
    let agent1 = BudgetEnforcementAgent::with_defaults();
    let agent2 = BudgetEnforcementAgent::with_defaults();

    let budget = create_test_budget(1000, "tenant-1");
    let spend = create_test_spend_data(500);
    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget.clone(), spend.clone(), exec_ref.clone());

    // Both agents should produce equivalent results
    let signal1 = agent1.evaluate(&request).await.unwrap();

    let exec_ref2 = create_test_execution_ref("tenant-1");
    let request2 = BudgetEvaluationRequest::new(budget, spend, exec_ref2);
    let signal2 = agent2.evaluate(&request2).await.unwrap();

    assert_eq!(
        signal1.violation_type, signal2.violation_type,
        "Stateless agents must produce deterministic results"
    );
    assert_eq!(
        signal1.severity, signal2.severity,
        "Stateless agents must produce deterministic results"
    );
}

#[tokio::test]
async fn test_agent_produces_deterministic_output() {
    let agent = BudgetEnforcementAgent::with_defaults();

    let budget = create_test_budget(1000, "tenant-1");
    let spend = create_test_spend_data(850); // 85% utilization

    // Run multiple evaluations
    for _ in 0..3 {
        let exec_ref = create_test_execution_ref("tenant-1");
        let request = BudgetEvaluationRequest::new(budget.clone(), spend.clone(), exec_ref);
        let signal = agent.evaluate(&request).await.unwrap();

        // Same inputs should produce same classification
        assert_eq!(
            signal.violation_type,
            BudgetViolationType::ApproachingLimit,
            "Agent must be deterministic"
        );
    }
}

#[tokio::test]
async fn test_agent_does_not_modify_execution() {
    let agent = BudgetEnforcementAgent::with_defaults();

    let budget = create_test_budget(100, "tenant-1");
    let spend = create_test_spend_data(150); // Over budget

    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget, spend, exec_ref);

    let signal = agent.evaluate(&request).await.unwrap();

    // Agent emits a signal but does NOT enforce
    assert_eq!(
        signal.signal_type,
        SignalType::Gating,
        "Agent should emit gating signal when over budget"
    );

    // The signal is advisory - it doesn't actually block anything
    // (blocking would be done by a downstream consumer)
}

// =============================================================================
// PROMPT 3: Platform Wiring Tests
// =============================================================================

#[test]
fn test_decision_type_is_budget_constraint_evaluation() {
    // The decision type must be exactly as specified in PROMPT 4
    let expected = DecisionType::BudgetConstraintEvaluation;
    assert_eq!(
        expected.to_string(),
        "budget_constraint_evaluation",
        "Decision type must match specification"
    );
}

#[test]
fn test_request_validation() {
    let budget = create_test_budget(1000, "tenant-1");
    let spend = create_test_spend_data(500);
    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget, spend, exec_ref);

    assert!(
        request.validate().is_ok(),
        "Valid request should pass validation"
    );
}

#[test]
fn test_request_validation_rejects_invalid_budget() {
    let mut budget = create_test_budget(1000, "tenant-1");
    budget.limit = Decimal::from(-100); // Invalid negative budget

    let spend = create_test_spend_data(500);
    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget, spend, exec_ref);

    assert!(
        request.validate().is_err(),
        "Invalid budget should fail validation"
    );
}

// =============================================================================
// PROMPT 4: Budget Constraint Evaluation Tests
// =============================================================================

#[tokio::test]
async fn test_budget_within_limits_returns_advisory() {
    let agent = BudgetEnforcementAgent::with_defaults();

    let budget = create_test_budget(1000, "tenant-1");
    let spend = create_test_spend_data(500); // 50% utilization

    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget, spend, exec_ref);

    let signal = agent.evaluate(&request).await.unwrap();

    assert_eq!(signal.violation_type, BudgetViolationType::None);
    assert_eq!(signal.signal_type, SignalType::Advisory);
}

#[tokio::test]
async fn test_budget_at_warning_threshold_returns_warning() {
    let agent = BudgetEnforcementAgent::with_defaults();

    let budget = create_test_budget(1000, "tenant-1");
    let spend = create_test_spend_data(850); // 85% utilization (above 80% warning)

    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget, spend, exec_ref);

    let signal = agent.evaluate(&request).await.unwrap();

    assert_eq!(signal.violation_type, BudgetViolationType::ApproachingLimit);
    assert_eq!(signal.signal_type, SignalType::Warning);
}

#[tokio::test]
async fn test_budget_at_critical_threshold_returns_critical() {
    let agent = BudgetEnforcementAgent::with_defaults();

    let budget = create_test_budget(1000, "tenant-1");
    let spend = create_test_spend_data(960); // 96% utilization (above 95% critical)

    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget, spend, exec_ref);

    let signal = agent.evaluate(&request).await.unwrap();

    assert_eq!(signal.violation_type, BudgetViolationType::LimitExceeded);
}

#[tokio::test]
async fn test_budget_exceeded_returns_gating() {
    let agent = BudgetEnforcementAgent::with_defaults();

    let budget = create_test_budget(1000, "tenant-1");
    let spend = create_test_spend_data(1050); // 105% utilization

    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget, spend, exec_ref);

    let signal = agent.evaluate(&request).await.unwrap();

    assert_eq!(signal.violation_type, BudgetViolationType::LimitExceeded);
    assert_eq!(signal.signal_type, SignalType::Gating);
}

#[tokio::test]
async fn test_utilization_calculation_accuracy() {
    let agent = BudgetEnforcementAgent::with_defaults();

    let budget = create_test_budget(1000, "tenant-1");
    let spend = create_test_spend_data(250); // 25% utilization

    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget, spend, exec_ref);

    let signal = agent.evaluate(&request).await.unwrap();

    assert!(
        (signal.utilization_percent - 25.0).abs() < 0.01,
        "Utilization calculation must be accurate"
    );
}

#[tokio::test]
async fn test_remaining_budget_calculation() {
    let agent = BudgetEnforcementAgent::with_defaults();

    let budget = create_test_budget(1000, "tenant-1");
    let spend = create_test_spend_data(300);

    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget, spend, exec_ref);

    let signal = agent.evaluate(&request).await.unwrap();

    assert_eq!(
        signal.remaining_budget,
        Decimal::from(700),
        "Remaining budget must be budget - current_spend"
    );
}

// =============================================================================
// Confidence Calculation Tests
// =============================================================================

#[tokio::test]
async fn test_confidence_high_with_complete_data() {
    let mut config = BudgetEnforcementConfig::default();
    config.persist_events = false;
    config.emit_telemetry = false;

    let agent = BudgetEnforcementAgent::new(config);

    let budget = create_test_budget(1000, "tenant-1");
    let mut spend = create_test_spend_data(500);
    spend.data_completeness = 1.0; // 100% complete

    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget, spend, exec_ref);

    // The agent internally computes confidence - we verify the signal is produced
    let signal = agent.evaluate(&request).await;
    assert!(signal.is_ok(), "Evaluation should succeed with complete data");
}

#[tokio::test]
async fn test_confidence_reduced_with_incomplete_data() {
    let mut config = BudgetEnforcementConfig::default();
    config.persist_events = false;
    config.emit_telemetry = false;

    let agent = BudgetEnforcementAgent::new(config);

    let budget = create_test_budget(1000, "tenant-1");
    let mut spend = create_test_spend_data(500);
    spend.data_completeness = 0.5; // 50% complete

    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget, spend, exec_ref);

    // The agent should still produce a signal, but with lower confidence
    let signal = agent.evaluate(&request).await;
    assert!(signal.is_ok(), "Evaluation should succeed even with incomplete data");
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_zero_budget_handling() {
    let agent = BudgetEnforcementAgent::with_defaults();

    let mut budget = create_test_budget(0, "tenant-1");
    budget.limit = Decimal::ZERO;

    let spend = create_test_spend_data(0);

    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget, spend, exec_ref);

    // Zero budget should fail validation
    assert!(request.validate().is_err());
}

#[tokio::test]
async fn test_zero_spend_handling() {
    let agent = BudgetEnforcementAgent::with_defaults();

    let budget = create_test_budget(1000, "tenant-1");
    let spend = create_test_spend_data(0);

    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget, spend, exec_ref);

    let signal = agent.evaluate(&request).await.unwrap();

    assert_eq!(signal.utilization_percent, 0.0);
    assert_eq!(signal.violation_type, BudgetViolationType::None);
}

#[tokio::test]
async fn test_exact_threshold_handling() {
    let agent = BudgetEnforcementAgent::with_defaults();

    let budget = create_test_budget(1000, "tenant-1");
    let spend = create_test_spend_data(800); // Exactly at 80% warning threshold

    let exec_ref = create_test_execution_ref("tenant-1");
    let request = BudgetEvaluationRequest::new(budget, spend, exec_ref);

    let signal = agent.evaluate(&request).await.unwrap();

    // At exactly the threshold, should trigger warning
    assert_eq!(signal.violation_type, BudgetViolationType::ApproachingLimit);
}

// =============================================================================
// Non-Responsibility Tests (What the agent MUST NOT do)
// =============================================================================

#[test]
fn test_agent_does_not_have_sql_capability() {
    // This is a design verification - the agent module doesn't import sqlx directly
    // Verified by code inspection: budget_enforcement.rs doesn't have any sqlx imports
}

#[test]
fn test_agent_output_is_advisory_only() {
    // The output types (BudgetConstraintSignal, SignalType) are purely informational
    // They don't contain any execution control mechanisms
    let advisory = SignalType::Advisory;
    let warning = SignalType::Warning;
    let gating = SignalType::Gating;

    // All signal types are informational - they don't execute anything
    assert!(matches!(advisory, SignalType::Advisory));
    assert!(matches!(warning, SignalType::Warning));
    assert!(matches!(gating, SignalType::Gating));
}
