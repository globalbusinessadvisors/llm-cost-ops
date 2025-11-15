// Edge-Agent integration stub

use crate::domain::{Result, UsageRecord};

pub struct EdgeAgentClient;

impl EdgeAgentClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn collect_edge_metrics(&self) -> Result<Vec<UsageRecord>> {
        // TODO: Implement Edge-Agent integration
        Ok(vec![])
    }
}

impl Default for EdgeAgentClient {
    fn default() -> Self {
        Self::new()
    }
}
