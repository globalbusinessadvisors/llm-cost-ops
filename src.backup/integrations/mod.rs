// Integration stubs for LLM DevOps modules

pub mod observatory;
pub mod edge_agent;
pub mod governance;

pub use observatory::ObservatoryClient;
pub use edge_agent::EdgeAgentClient;
pub use governance::GovernanceClient;
