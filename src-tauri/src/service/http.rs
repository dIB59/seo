use anyhow::{Context, Result};
use rquest::Client;
use rquest_util::Emulation;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum ClientType {
    Standard,
    HeavyEmulation,
}

/// Factory for creating an HTTP client based on the desired level of stealth/performance.
pub fn create_client(client_type: ClientType) -> Result<Client> {
    let builder = Client::builder().timeout(Duration::from_secs(30));

    match client_type {
        ClientType::HeavyEmulation => {
            // Use rquest_util for heavy browser impersonation
            builder
                .emulation(Emulation::Firefox136)
                .build()
                .context("Failed to build heavy impersonated rquest client")
        }
        ClientType::Standard => {
            // Standard rquest client
            builder
                .build()
                .context("Failed to build standard rquest client")
        }
    }
}
