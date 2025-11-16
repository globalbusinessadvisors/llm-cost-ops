// Observatory integration stub

use crate::domain::{Result, UsageRecord};

pub struct ObservatoryClient;

impl ObservatoryClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn emit_cost_metric(&self, _record: &UsageRecord) -> Result<()> {
        // TODO: Implement Observatory integration
        Ok(())
    }
}

impl Default for ObservatoryClient {
    fn default() -> Self {
        Self::new()
    }
}
