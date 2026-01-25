//! Policy Evaluation Module
//!
//! Phase 4 Layer 1 - Policy evaluation (analysis only, NO ENFORCEMENT).
//!
//! # CRITICAL: Analysis-Only Architecture
//!
//! This module evaluates policies and emits signals. It does NOT:
//! - Block operations
//! - Enforce rules
//! - Modify behavior
//!
//! The consuming system decides what action to take based on evaluation results.

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::collections::HashMap;

use super::types::{RiskLevel, ViolationType};

/// Policy rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Unique policy identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Policy description
    pub description: String,

    /// Policy type/category
    pub policy_type: ViolationType,

    /// Severity if violated
    pub severity: PolicySeverity,

    /// Rule condition (for evaluation)
    pub condition: PolicyCondition,

    /// Whether policy is active
    pub active: bool,

    /// Organization scope (None = global)
    pub organization_id: Option<String>,

    /// Project scope (None = all projects)
    pub project_id: Option<String>,
}

/// Policy severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicySeverity {
    /// Informational - log only
    Info,
    /// Warning - should be reviewed
    Warning,
    /// Error - requires attention
    Error,
    /// Critical - immediate action needed
    Critical,
}

impl From<PolicySeverity> for RiskLevel {
    fn from(severity: PolicySeverity) -> Self {
        match severity {
            PolicySeverity::Info => RiskLevel::Low,
            PolicySeverity::Warning => RiskLevel::Medium,
            PolicySeverity::Error => RiskLevel::High,
            PolicySeverity::Critical => RiskLevel::Critical,
        }
    }
}

/// Policy condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum PolicyCondition {
    /// Budget threshold check
    BudgetThreshold {
        budget_id: String,
        threshold_percent: u8,
    },

    /// Cost limit check
    CostLimit {
        limit: Decimal,
        period: String,
    },

    /// Growth rate check
    GrowthRateLimit {
        max_rate_percent: f64,
        period: String,
    },

    /// Model usage restriction
    ModelRestriction {
        allowed_models: Vec<String>,
    },

    /// Resource quota check
    ResourceQuota {
        resource_type: String,
        max_usage: u64,
    },

    /// Custom expression (for extensibility)
    Custom {
        expression: String,
        params: HashMap<String, serde_json::Value>,
    },
}

/// Policy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    /// Policy that was evaluated
    pub policy_id: String,

    /// Policy name
    pub policy_name: String,

    /// Whether policy is satisfied
    pub satisfied: bool,

    /// Violation details (if not satisfied)
    pub violation: Option<PolicyViolation>,

    /// Evaluation context
    pub context: HashMap<String, serde_json::Value>,
}

/// Policy violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    /// Violation type
    pub violation_type: ViolationType,

    /// Severity
    pub severity: PolicySeverity,

    /// Description of violation
    pub description: String,

    /// Actual value that violated
    pub actual_value: serde_json::Value,

    /// Expected/allowed value
    pub expected_value: serde_json::Value,

    /// Recommended remediation (advisory only)
    pub recommended_remediation: String,
}

/// Policy evaluator
///
/// # IMPORTANT: Analysis-Only
/// This evaluator ONLY analyzes and reports. It does NOT enforce policies.
pub struct PolicyEvaluator {
    rules: Vec<PolicyRule>,
}

impl PolicyEvaluator {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn with_rules(rules: Vec<PolicyRule>) -> Self {
        Self { rules }
    }

    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
    }

    /// Evaluate all policies against the given context
    ///
    /// # Returns
    /// Vector of evaluation results. The caller decides what to do with violations.
    /// This function does NOT enforce anything.
    pub fn evaluate_all(&self, context: &EvaluationContext) -> Vec<PolicyResult> {
        self.rules
            .iter()
            .filter(|r| r.active)
            .filter(|r| self.scope_matches(r, context))
            .map(|r| self.evaluate_rule(r, context))
            .collect()
    }

    /// Evaluate a single policy
    pub fn evaluate(&self, policy_id: &str, context: &EvaluationContext) -> Option<PolicyResult> {
        self.rules
            .iter()
            .find(|r| r.id == policy_id && r.active)
            .map(|r| self.evaluate_rule(r, context))
    }

    /// Get all violations from evaluation results
    pub fn get_violations(results: &[PolicyResult]) -> Vec<&PolicyResult> {
        results.iter().filter(|r| !r.satisfied).collect()
    }

    fn scope_matches(&self, rule: &PolicyRule, context: &EvaluationContext) -> bool {
        // Check organization scope
        if let Some(ref rule_org) = rule.organization_id {
            if let Some(ref ctx_org) = context.organization_id {
                if rule_org != ctx_org {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check project scope
        if let Some(ref rule_project) = rule.project_id {
            if let Some(ref ctx_project) = context.project_id {
                if rule_project != ctx_project {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    fn evaluate_rule(&self, rule: &PolicyRule, context: &EvaluationContext) -> PolicyResult {
        let (satisfied, violation) = match &rule.condition {
            PolicyCondition::BudgetThreshold { budget_id, threshold_percent } => {
                self.evaluate_budget_threshold(budget_id, *threshold_percent, context, rule)
            }

            PolicyCondition::CostLimit { limit, period } => {
                self.evaluate_cost_limit(*limit, period, context, rule)
            }

            PolicyCondition::GrowthRateLimit { max_rate_percent, period } => {
                self.evaluate_growth_rate(*max_rate_percent, period, context, rule)
            }

            PolicyCondition::ModelRestriction { allowed_models } => {
                self.evaluate_model_restriction(allowed_models, context, rule)
            }

            PolicyCondition::ResourceQuota { resource_type, max_usage } => {
                self.evaluate_resource_quota(resource_type, *max_usage, context, rule)
            }

            PolicyCondition::Custom { expression, params } => {
                self.evaluate_custom(expression, params, context, rule)
            }
        };

        PolicyResult {
            policy_id: rule.id.clone(),
            policy_name: rule.name.clone(),
            satisfied,
            violation,
            context: context.data.clone(),
        }
    }

    fn evaluate_budget_threshold(
        &self,
        budget_id: &str,
        threshold_percent: u8,
        context: &EvaluationContext,
        rule: &PolicyRule,
    ) -> (bool, Option<PolicyViolation>) {
        let current = context.get_decimal("current_spend").unwrap_or(Decimal::ZERO);
        let limit = context.get_decimal("budget_limit").unwrap_or(Decimal::ONE);

        if limit.is_zero() {
            return (true, None);
        }

        let utilization = ((current / limit) * Decimal::from(100))
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0);

        let satisfied = utilization <= threshold_percent as f64;

        let violation = if !satisfied {
            Some(PolicyViolation {
                violation_type: rule.policy_type.clone(),
                severity: rule.severity,
                description: format!(
                    "Budget '{}' at {:.1}% utilization (threshold: {}%)",
                    budget_id, utilization, threshold_percent
                ),
                actual_value: serde_json::json!(utilization),
                expected_value: serde_json::json!(threshold_percent),
                recommended_remediation: format!(
                    "Review spending for budget '{}' and consider adjusting limits or reducing usage",
                    budget_id
                ),
            })
        } else {
            None
        };

        (satisfied, violation)
    }

    fn evaluate_cost_limit(
        &self,
        limit: Decimal,
        period: &str,
        context: &EvaluationContext,
        rule: &PolicyRule,
    ) -> (bool, Option<PolicyViolation>) {
        let current = context.get_decimal("current_cost").unwrap_or(Decimal::ZERO);
        let satisfied = current <= limit;

        let violation = if !satisfied {
            Some(PolicyViolation {
                violation_type: rule.policy_type.clone(),
                severity: rule.severity,
                description: format!(
                    "Cost ${} exceeds {} limit of ${}",
                    current, period, limit
                ),
                actual_value: serde_json::json!(current.to_string()),
                expected_value: serde_json::json!(limit.to_string()),
                recommended_remediation: format!(
                    "Reduce {} spending to stay within ${} limit",
                    period, limit
                ),
            })
        } else {
            None
        };

        (satisfied, violation)
    }

    fn evaluate_growth_rate(
        &self,
        max_rate: f64,
        period: &str,
        context: &EvaluationContext,
        rule: &PolicyRule,
    ) -> (bool, Option<PolicyViolation>) {
        let current_rate = context.get_f64("growth_rate").unwrap_or(0.0);
        let satisfied = current_rate <= max_rate;

        let violation = if !satisfied {
            Some(PolicyViolation {
                violation_type: rule.policy_type.clone(),
                severity: rule.severity,
                description: format!(
                    "Growth rate {:.1}% exceeds {} max of {:.1}%",
                    current_rate, period, max_rate
                ),
                actual_value: serde_json::json!(current_rate),
                expected_value: serde_json::json!(max_rate),
                recommended_remediation: format!(
                    "Review {} cost drivers and implement optimization measures",
                    period
                ),
            })
        } else {
            None
        };

        (satisfied, violation)
    }

    fn evaluate_model_restriction(
        &self,
        allowed_models: &[String],
        context: &EvaluationContext,
        rule: &PolicyRule,
    ) -> (bool, Option<PolicyViolation>) {
        let used_model = context.get_string("model_id").unwrap_or_default();
        let satisfied = allowed_models.iter().any(|m| m == &used_model);

        let violation = if !satisfied {
            Some(PolicyViolation {
                violation_type: rule.policy_type.clone(),
                severity: rule.severity,
                description: format!(
                    "Model '{}' is not in allowed list: {:?}",
                    used_model, allowed_models
                ),
                actual_value: serde_json::json!(used_model),
                expected_value: serde_json::json!(allowed_models),
                recommended_remediation: format!(
                    "Use one of the allowed models: {}",
                    allowed_models.join(", ")
                ),
            })
        } else {
            None
        };

        (satisfied, violation)
    }

    fn evaluate_resource_quota(
        &self,
        resource_type: &str,
        max_usage: u64,
        context: &EvaluationContext,
        rule: &PolicyRule,
    ) -> (bool, Option<PolicyViolation>) {
        let current_usage = context.get_u64(&format!("{}_usage", resource_type)).unwrap_or(0);
        let satisfied = current_usage <= max_usage;

        let violation = if !satisfied {
            Some(PolicyViolation {
                violation_type: rule.policy_type.clone(),
                severity: rule.severity,
                description: format!(
                    "Resource '{}' usage {} exceeds quota {}",
                    resource_type, current_usage, max_usage
                ),
                actual_value: serde_json::json!(current_usage),
                expected_value: serde_json::json!(max_usage),
                recommended_remediation: format!(
                    "Reduce {} usage or request quota increase",
                    resource_type
                ),
            })
        } else {
            None
        };

        (satisfied, violation)
    }

    fn evaluate_custom(
        &self,
        _expression: &str,
        _params: &HashMap<String, serde_json::Value>,
        _context: &EvaluationContext,
        _rule: &PolicyRule,
    ) -> (bool, Option<PolicyViolation>) {
        // Custom expressions would need a proper expression evaluator
        // For now, always pass (can be extended later)
        (true, None)
    }
}

impl Default for PolicyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for policy evaluation
#[derive(Debug, Clone, Default)]
pub struct EvaluationContext {
    pub organization_id: Option<String>,
    pub project_id: Option<String>,
    pub user_id: Option<String>,
    pub data: HashMap<String, serde_json::Value>,
}

impl EvaluationContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_organization(mut self, org_id: impl Into<String>) -> Self {
        self.organization_id = Some(org_id.into());
        self
    }

    pub fn with_project(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.data.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    pub fn get_decimal(&self, key: &str) -> Option<Decimal> {
        self.data.get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| {
                self.data.get(key)
                    .and_then(|v| v.as_f64())
                    .map(|f| Decimal::try_from(f).unwrap_or_default())
            })
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.data.get(key).and_then(|v| v.as_f64())
    }

    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.data.get(key).and_then(|v| v.as_u64())
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.data.get(key).and_then(|v| v.as_str()).map(String::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_budget_rule() -> PolicyRule {
        PolicyRule {
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
        }
    }

    #[test]
    fn test_budget_policy_satisfied() {
        let evaluator = PolicyEvaluator::with_rules(vec![create_budget_rule()]);

        let mut context = EvaluationContext::new();
        context.set("current_spend", serde_json::json!("700"));
        context.set("budget_limit", serde_json::json!("1000"));

        let results = evaluator.evaluate_all(&context);
        assert_eq!(results.len(), 1);
        assert!(results[0].satisfied);
    }

    #[test]
    fn test_budget_policy_violated() {
        let evaluator = PolicyEvaluator::with_rules(vec![create_budget_rule()]);

        let mut context = EvaluationContext::new();
        context.set("current_spend", serde_json::json!("900"));
        context.set("budget_limit", serde_json::json!("1000"));

        let results = evaluator.evaluate_all(&context);
        assert_eq!(results.len(), 1);
        assert!(!results[0].satisfied);
        assert!(results[0].violation.is_some());
    }

    #[test]
    fn test_get_violations() {
        let evaluator = PolicyEvaluator::with_rules(vec![
            create_budget_rule(),
            PolicyRule {
                id: "cost-limit".to_string(),
                name: "Daily Cost Limit".to_string(),
                description: "Max daily cost".to_string(),
                policy_type: ViolationType::CostOptimizationPolicy,
                severity: PolicySeverity::Error,
                condition: PolicyCondition::CostLimit {
                    limit: Decimal::from(100),
                    period: "daily".to_string(),
                },
                active: true,
                organization_id: None,
                project_id: None,
            },
        ]);

        let mut context = EvaluationContext::new();
        context.set("current_spend", serde_json::json!("700"));
        context.set("budget_limit", serde_json::json!("1000"));
        context.set("current_cost", serde_json::json!(50.0));

        let results = evaluator.evaluate_all(&context);
        let violations = PolicyEvaluator::get_violations(&results);

        assert_eq!(violations.len(), 0); // Both policies satisfied
    }
}
