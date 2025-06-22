use std::sync::{Arc, Mutex};

use crate::{schema::MetadataTable, youtube::YouTube};

/// Vault
#[derive(Clone, Debug)]
pub struct Vault {
    pub state: Arc<Mutex<State>>,
}

/// State
#[derive(Debug)]
pub struct State {
    pub metadata_table: MetadataTable,
    pub youtube: YouTube,
}

impl Vault {
    pub fn new(metadata_table: MetadataTable, youtube: YouTube) -> Self {
        let state = Arc::new(Mutex::new(State {
            metadata_table,
            youtube,
        }));

        Self { state }
    }
}
