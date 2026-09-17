//! Integration and unit tests for Atena Studio Associative Memory Engine
//! Structured across dedicated modules:
//! - `basic_learning`: Dialogue parsing, Hebbian learning, and queries
//! - `temporal_events`: Temporal resolution, relative dates, and timestamps
//! - `rules_and_forget`: Negative behavioral rules and explicit forgetting tags
//! - `episodic_chain`: Markdown episodic journals and session scopes
//! - `persistence_and_v1`: Binary serialization, migration, and V1 compatibility
//! - `massive_brain`: Large-scale graph population and multi-hop associative traversal

mod basic_learning;
mod temporal_events;
mod rules_and_forget;
mod episodic_chain;
mod persistence_and_v1;
mod massive_brain;
