//! Integration tests for Governance & FinOps (Phase 4 Layer 1)

use llm_cost_ops::governance::{
    // Constants
    AGENT_PHASE, AGENT_LAYER, MAX_TOKENS, MAX_LATENCY_MS,
    // Configuration
    GovernanceConfig,
    // Signal types
    GovernanceDecisionEvent, GovernanceDecisionType,
    CostRiskSignal, BudgetThresholdSignal, PolicyViolationSignal, ApprovalRequiredSignal,
    GovernanceRiskLevel, ApprovalType, ViolationType,
    // Policy evaluation
    PolicyEvaluator, PolicyRule, PolicySeverity,
    policy::{PolicyCondition, EvaluationContext},
    // Performance budgets
    PerformanceBudget, PerformanceGuard,
};
use llm_cost_ops::agents::{AgentId, AgentVersion, RuVectorClient};
use rust_decimal::Decimal;

/// Test Phase 4 Layer 1 constants are correctly defined
#[test]
fn test_phase4_layer1_constants() {
    assert_eq!(AGENT_PHASE, "phase4");
    assert_eq!(AGENT_LAYER, "layer1");
    assert_eq!(MAX_TOKENS, 1200);
    assert_eq!(MAX_LATENCY_MS, 2500);
}

/// Test governance configuration from defaults
#[test]
fn test_governance_config_defaults() {
    let config = GovernanceConfig::default();

    assert_eq!(config.phase, "phase4");
    assert_eq!(config.layer, "layer1");
    assert_eq!(config.max_tokens, 1200);
    assert_eq!(config.max_latency_ms, 2500);
    assert!(config.enable_policy_signals);
    assert!(config.enable_cost_signals);
    assert!(config.enable_approval_signals);
}

/// Test cost_risk_signal creation and serialization
#[test]
fn test_cost_risk_signal() {
    let signal = CostRiskSignal::new(
        AgentId::new("cost-forecasting-agent"),
        AgentVersion::new("1.0.0"),
        GovernanceRiskLevel::High,
        "cost_spike",
        Decimal::from(150),
        Decimal::from(100),
    )
    .with_resources(vec!["gpt-4".to_string(), "claude-3-opus".to_string()])
    .with_recommendation("Review model usage and consider using cheaper alternatives")
    .with_organization("org-123")
    .with_project("project-456");

    assert_eq!(signal.risk_level, GovernanceRiskLevel::High);
    assert_eq!(signal.deviation_percent, 50.0);
    assert_eq!(signal.affected_resources.len(), 2);
    assert!(signal.organization_id.is_some());
    assert!(signal.project_id.is_some());

    // Test serialization
    let json = serde_json::to_string(&signal).unwrap();
    assert!(json.contains("cost_spike"));
    assert!(json.contains("high"));
}

/// Test budget_threshold_signal creation
#[test]
fn test_budget_threshold_signal() {
    let signal = BudgetThresholdSignal::new(
        AgentId::new("budget-enforcement-agent"),
        AgentVersion::new("1.0.0"),
        "monthly-budget",
        85,
        Decimal::from(8500),
        Decimal::from(10000),
    )
    .with_projected_overage(Decimal::from(500))
    .with_days_until_exhaustion(5)
    .with_recommendation("Reduce usage or request budget increase");

    assert_eq!(signal.threshold_percent, 85);
    assert_eq!(signal.risk_level, GovernanceRiskLevel::High);
    assert_eq!(signal.utilization_percent, 85.0);
    assert!(signal.projected_overage.is_some());
    assert_eq!(signal.days_until_exhaustion, Some(5));
}

/// Test policy_violation_signal (NO ENFORCEMENT)
#[test]
fn test_policy_violation_signal() {
    let signal = PolicyViolationSignal::new(
        AgentId::new("policy-agent"),
        AgentVersion::new("1.0.0"),
        "budget-policy-001",
        "Monthly Budget Cap",
        ViolationType::BudgetPolicy,
        GovernanceRiskLevel::Critical,
        "Monthly budget exceeded by 15%",
    )
    .with_values(
        serde_json::json!(11500),
        serde_json::json!(10000),
    )
    .with_affected_entity("project-456")
    .with_remediation("Review and reduce usage or request budget increase")
    .with_blocking(false); // NOTE: is_blocking is INFORMATIONAL ONLY

    assert_eq!(signal.violation_type, ViolationType::BudgetPolicy);
    assert_eq!(signal.severity, GovernanceRiskLevel::Critical);
    // CRITICAL: is_blocking is for UI purposes only - agents MUST NOT enforce
    assert!(!signal.is_blocking);
}

/// Test approval_required_signal (NO AUTO-APPROVE)
#[test]
fn test_approval_required_signal() {
    let signal = ApprovalRequiredSignal::new(
        AgentId::new("cost-agent"),
        AgentVersion::new("1.0.0"),
        ApprovalType::BudgetOverride,
        "Request to exceed monthly budget",
        "Current spend projected to exceed budget by $500",
        "user-123",
    )
    .with_action(serde_json::json!({
        "type": "budget_override",
        "amount": 500,
        "duration": "month"
    }))
    .with_risk_level(GovernanceRiskLevel::High)
    .with_impact("Additional $500 spend this month")
    .with_approvers(vec!["finance-team".to_string(), "manager-456".to_string()]);

    assert_eq!(signal.approval_type, ApprovalType::BudgetOverride);
    assert_eq!(signal.risk_level, GovernanceRiskLevel::High);
    assert_eq!(signal.suggested_approvers.len(), 2);
    // CRITICAL: suggested_approvers is for ROUTING only - agents MUST NOT auto-approve
}

/// Test GovernanceDecisionEvent wrapper
#[test]
fn test_governance_decision_event() {
    let signal = CostRiskSignal::new(
        AgentId::new("test-agent"),
        AgentVersion::new("1.0.0"),
        GovernanceRiskLevel::Medium,
        "anomaly",
        Decimal::from(120),
        Decimal::from(100),
    );

    let event = GovernanceDecisionEvent::cost_risk(signal);

    assert_eq!(event.decision_type, GovernanceDecisionType::CostRiskSignal);
    assert_eq!(event.phase, "phase4");
    assert_eq!(event.layer, "layer1");
    assert!(!event.id.is_nil());
}

/// Test all GovernanceDecisionType variants
#[test]
fn test_governance_decision_types() {
    assert_eq!(GovernanceDecisionType::CostRiskSignal.to_string(), "cost_risk_signal");
    assert_eq!(GovernanceDecisionType::BudgetThresholdSignal.to_string(), "budget_threshold_signal");
    assert_eq!(GovernanceDecisionType::PolicyViolationSignal.to_string(), "policy_violation_signal");
    assert_eq!(GovernanceDecisionType::ApprovalRequiredSignal.to_string(), "approval_required_signal");
}

/// Test policy evaluator (analysis only - NO ENFORCEMENT)
#[test]
fn test_policy_evaluator_analysis_only() {
    let rule = PolicyRule {
        id: "budget-80".to_string(),
        name: "Budget 80% Warning".to_string(),
        description: "Warn when budget reaches 80%".to_string(),
        policy_type: ViolationType::BudgetPolicy,
        severity: PolicySeverity::Warning,
        condition: PolicyCondition::BudgetThreshold {
            budget_id: "monthly".to_string(),
            threshold_percent: 80,
        },
        active: true,
        organization_id: None,
        project_id: None,
    };

    let evaluator = PolicyEvaluator::with_rules(vec![rule]);

    // Test policy violation detection
    let mut context = EvaluationContext::new();
    context.set("current_spend", serde_json::json!("9000"));
    context.set("budget_limit", serde_json::json!("10000"));

    let results = evaluator.evaluate_all(&context);

    assert_eq!(results.len(), 1);
    assert!(!results[0].satisfied); // Policy violated (90% > 80%)
    assert!(results[0].violation.is_some());

    // CRITICAL: The evaluator only REPORTS violations
    // It does NOT block or enforce anything
    // The consuming system decides what to do
}

/// Test performance budget enforcement
#[test]
fn test_performance_budget() {
    let budget = PerformanceBudget::default();

    assert_eq!(budget.max_tokens, 1200);
    assert_eq!(budget.max_latency_ms, 2500);

    // Test token check within budget
    assert!(budget.check_tokens(1000).is_ok());

    // Test token check over budget (advisory mode - should not fail)
    assert!(budget.check_tokens(2000).is_ok()); // Advisory only

    // Test strict mode
    let strict_budget = PerformanceBudget::default().with_strict(true);
    assert!(strict_budget.check_tokens(2000).is_err());
}

/// Test performance guard
#[test]
fn test_performance_guard() {
    let budget = PerformanceBudget::new(1000, 5000);
    let mut guard = budget.guard();

    guard.record_tokens(500);

    let metrics = guard.finish();

    assert_eq!(metrics.tokens_used, 500);
    assert_eq!(metrics.max_tokens, 1000);
    assert!(metrics.within_budget);
    assert_eq!(metrics.token_utilization(), 50.0);
}

/// Test RuVector client governance event persistence
#[tokio::test]
async fn test_ruvector_governance_persistence() {
    let client = RuVectorClient::with_defaults().unwrap();

    let signal = CostRiskSignal::new(
        AgentId::new("test-agent"),
        AgentVersion::new("1.0.0"),
        GovernanceRiskLevel::High,
        "test_anomaly",
        Decimal::from(150),
        Decimal::from(100),
    );

    let event = GovernanceDecisionEvent::cost_risk(signal);
    let result = client.persist_governance_event(&event).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, "persisted");
}

/// Test governance rules compliance: Agents MUST emit signals
#[test]
fn test_governance_rules_emit_requirement() {
    // This test verifies the design supports the requirement:
    // "Agents MUST emit cost signals, policy evaluation signals, approval requirements"

    // Cost signal can be created
    let cost_signal = CostRiskSignal::new(
        AgentId::new("agent"),
        AgentVersion::new("1.0.0"),
        GovernanceRiskLevel::Medium,
        "test",
        Decimal::from(100),
        Decimal::from(100),
    );
    assert!(!cost_signal.signal_id.is_nil());

    // Budget threshold signal can be created
    let budget_signal = BudgetThresholdSignal::new(
        AgentId::new("agent"),
        AgentVersion::new("1.0.0"),
        "budget",
        50,
        Decimal::from(50),
        Decimal::from(100),
    );
    assert!(!budget_signal.signal_id.is_nil());

    // Policy violation signal can be created (NO ENFORCEMENT)
    let policy_signal = PolicyViolationSignal::new(
        AgentId::new("agent"),
        AgentVersion::new("1.0.0"),
        "policy",
        "Test Policy",
        ViolationType::BudgetPolicy,
        GovernanceRiskLevel::Low,
        "Test description",
    );
    assert!(!policy_signal.signal_id.is_nil());

    // Approval required signal can be created (NO AUTO-APPROVE)
    let approval_signal = ApprovalRequiredSignal::new(
        AgentId::new("agent"),
        AgentVersion::new("1.0.0"),
        ApprovalType::BudgetOverride,
        "Test approval",
        "Test reason",
        "user",
    );
    assert!(!approval_signal.signal_id.is_nil());
}

/// Test governance rules compliance: Agents MUST NOT auto-enforce or auto-approve
#[test]
fn test_governance_rules_no_enforcement() {
    // This test verifies the design prevents:
    // "Agents MUST NOT auto-enforce policy"
    // "Agents MUST NOT auto-approve actions"

    // PolicyViolationSignal has is_blocking but it's INFORMATIONAL ONLY
    let signal = PolicyViolationSignal::new(
        AgentId::new("agent"),
        AgentVersion::new("1.0.0"),
        "policy",
        "Test",
        ViolationType::BudgetPolicy,
        GovernanceRiskLevel::Critical,
        "Critical violation",
    );

    // Even with is_blocking=true, the signal is ADVISORY
    // The consuming system decides whether to block
    // The agent MUST NOT enforce anything
    let signal_with_blocking = signal.with_blocking(true);
    assert!(signal_with_blocking.is_blocking); // Just a flag for the UI

    // ApprovalRequiredSignal has suggested_approvers but does NOT auto-approve
    let approval = ApprovalRequiredSignal::new(
        AgentId::new("agent"),
        AgentVersion::new("1.0.0"),
        ApprovalType::HighCostOperation,
        "High cost request",
        "Exceeds threshold",
        "user",
    )
    .with_approvers(vec!["approver1".to_string()]);

    // suggested_approvers is for ROUTING only
    // The agent MUST NOT auto-approve anything
    assert_eq!(approval.suggested_approvers.len(), 1);
}

/// Test risk level ordering
#[test]
fn test_risk_level_ordering() {
    assert!(GovernanceRiskLevel::Low < GovernanceRiskLevel::Medium);
    assert!(GovernanceRiskLevel::Medium < GovernanceRiskLevel::High);
    assert!(GovernanceRiskLevel::High < GovernanceRiskLevel::Critical);
}
