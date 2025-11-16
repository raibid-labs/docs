//! BRP configuration

use bevy::prelude::*;

/// BRP server configuration
#[derive(Resource, Debug, Clone)]
pub struct BrpConfig {
    /// BRP server host
    pub host: String,
    /// BRP server port
    pub port: u16,
}

impl Default for BrpConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 15702,
        }
    }
}
