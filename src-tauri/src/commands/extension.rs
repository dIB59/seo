//! Extension Management Commands
//!
//! This module provides Tauri commands for managing SEO extensions,
//! including issue generators, data extractors, and data exporters.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;
use std::collections::HashMap;

use crate::extension::{ExtensionCapability, ExtensionConfig};
use crate::error::CommandError;
use crate::lifecycle::app_state::AppState;

// ============================================================================
// Response Types
// ============================================================================

/// Information about an extension for the frontend
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub capabilities: Vec<String>,
    pub is_builtin: bool,
    pub is_enabled: bool,
}

/// Information about an issue generator for the frontend
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct IssueGeneratorInfo {
    pub id: String,
    pub name: String,
    pub category: String,
    pub severity: String,
    pub recommendation: Option<String>,
    pub is_builtin: bool,
    pub is_enabled: bool,
}

/// Information about a data extractor for the frontend
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DataExtractorInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub extractor_type: String,
    pub is_builtin: bool,
    pub is_enabled: bool,
}

/// Information about a data exporter for the frontend
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DataExporterInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub export_format: String,
    pub endpoint: Option<String>,
    pub is_builtin: bool,
    pub is_enabled: bool,
}

/// Request to create a new custom rule
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateRuleRequest {
    pub name: String,
    pub category: String,
    pub severity: String,
    pub rule_type: String,
    pub target_field: String,
    pub threshold_min: Option<f64>,
    pub threshold_max: Option<f64>,
    pub regex_pattern: Option<String>,
    pub recommendation: Option<String>,
}

/// Request to update an existing rule
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateRuleRequest {
    pub id: String,
    pub name: Option<String>,
    pub severity: Option<String>,
    pub threshold_min: Option<f64>,
    pub threshold_max: Option<f64>,
    pub regex_pattern: Option<String>,
    pub recommendation: Option<String>,
    pub is_enabled: Option<bool>,
}

/// Request to create a custom data extractor
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateExtractorRequest {
    pub name: String,
    pub description: Option<String>,
    pub selector: String,
    pub attribute: Option<String>,
    pub multiple: bool,
}

/// Summary of extension system status
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExtensionSummary {
    pub total_extensions: usize,
    pub total_validators: usize,
    pub total_extractors: usize,
    pub total_exporters: usize,
    pub builtin_count: usize,
    pub custom_count: usize,
}

/// Extracted data for a page
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExtractedDataResponse {
    pub page_id: String,
    pub url: String,
    pub data: HashMap<String, serde_json::Value>,
    pub extracted_at: String,
}

// ============================================================================
// Commands
// ============================================================================

/// Get a summary of the extension system
#[tauri::command]
#[specta::specta]
pub async fn get_extension_summary(
    app_state: State<'_, AppState>,
) -> Result<ExtensionSummary, CommandError> {
    let registry = &app_state.extension_registry;
    let repo = &app_state.extension_repository;
    
    let custom_rules = repo.count_custom_rules().await
        .map_err(|e| CommandError::from(anyhow::anyhow!("{}", e)))?;
    
    let validators = registry.rule_count();
    let extractors = registry.extractor_count();
    let exporters = registry.exporter_count();
    
    Ok(ExtensionSummary {
        total_extensions: validators + extractors + exporters,
        total_validators: validators,
        total_extractors: extractors,
        total_exporters: exporters,
        builtin_count: (validators + extractors + exporters).saturating_sub(custom_rules),
        custom_count: custom_rules,
    })
}

/// Get all registered extensions
#[tauri::command]
#[specta::specta]
pub async fn get_all_extensions(
    app_state: State<'_, AppState>,
) -> Result<Vec<ExtensionInfo>, CommandError> {
    let registry = &app_state.extension_registry;
    
    let extensions: Vec<ExtensionInfo> = registry
        .get_all_configs()
        .into_iter()
        .map(|config| ExtensionInfo {
            id: config.id,
            name: config.name,
            description: config.description,
            version: config.version,
            capabilities: config.capabilities.capabilities.into_iter()
                .map(|c| format!("{:?}", c).to_lowercase())
                .collect(),
            is_builtin: config.is_builtin,
            is_enabled: config.enabled,
        })
        .collect();
    
    Ok(extensions)
}

/// Get all registered issue generators (validators)
#[tauri::command]
#[specta::specta]
pub async fn get_all_issue_generators(
    app_state: State<'_, AppState>,
) -> Result<Vec<IssueGeneratorInfo>, CommandError> {
    let repo = &app_state.extension_repository;
    let rules = repo.get_all_rules().await
        .map_err(|e| CommandError::from(anyhow::anyhow!("{}", e)))?;
    
    let generators: Vec<IssueGeneratorInfo> = rules
        .into_iter()
        .map(|rule| IssueGeneratorInfo {
            id: rule.id,
            name: rule.name,
            category: rule.category,
            severity: rule.severity,
            recommendation: rule.recommendation,
            is_builtin: rule.is_builtin,
            is_enabled: rule.is_enabled,
        })
        .collect();
    
    Ok(generators)
}

/// Get all registered data extractors
#[tauri::command]
#[specta::specta]
pub async fn get_all_extractors(
    app_state: State<'_, AppState>,
) -> Result<Vec<DataExtractorInfo>, CommandError> {
    let registry = &app_state.extension_registry;
    let extractor_ids = registry.get_data_extractor_ids();
    
    let extractors: Vec<DataExtractorInfo> = extractor_ids
        .into_iter()
        .map(|id| {
            let config = registry.get_config(&id);
            DataExtractorInfo {
                id: id.clone(),
                name: config.as_ref().map(|c| c.name.clone()).unwrap_or_else(|| id.clone()),
                description: config.as_ref().and_then(|c| c.description.clone()),
                extractor_type: "css_selector".to_string(),
                is_builtin: config.as_ref().map(|c| c.is_builtin).unwrap_or(true),
                is_enabled: config.as_ref().map(|c| c.enabled).unwrap_or(true),
            }
        })
        .collect();
    
    Ok(extractors)
}

/// Get all registered data exporters
#[tauri::command]
#[specta::specta]
pub async fn get_all_exporters(
    app_state: State<'_, AppState>,
) -> Result<Vec<DataExporterInfo>, CommandError> {
    let registry = &app_state.extension_registry;
    let exporter_ids = registry.get_data_exporter_ids();
    
    let exporters: Vec<DataExporterInfo> = exporter_ids
        .into_iter()
        .map(|id| {
            let config = registry.get_config(&id);
            DataExporterInfo {
                id: id.clone(),
                name: config.as_ref().map(|c| c.name.clone()).unwrap_or_else(|| id.clone()),
                description: config.as_ref().and_then(|c| c.description.clone()),
                export_format: "json".to_string(),
                endpoint: None,
                is_builtin: config.as_ref().map(|c| c.is_builtin).unwrap_or(true),
                is_enabled: config.as_ref().map(|c| c.enabled).unwrap_or(true),
            }
        })
        .collect();
    
    Ok(exporters)
}

/// Create a new custom issue generator
#[tauri::command]
#[specta::specta]
pub async fn create_custom_rule(
    request: CreateRuleRequest,
    app_state: State<'_, AppState>,
) -> Result<IssueGeneratorInfo, CommandError> {
    let repo = &app_state.extension_repository;
    
    let id = format!("custom-{}", uuid::Uuid::new_v4());
    
    let new_rule = crate::contexts::IssueRuleInfo {
        id: id.clone(),
        name: request.name.clone(),
        category: request.category.clone(),
        severity: request.severity.clone(),
        rule_type: request.rule_type.clone(),
        target_field: Some(request.target_field.clone()),
        recommendation: request.recommendation.clone(),
        is_builtin: false,
        is_enabled: true,
    };
    
    repo.insert_rule(&new_rule).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to create custom rule: {}", e)))?;
    
    Ok(IssueGeneratorInfo {
        id,
        name: request.name,
        category: request.category,
        severity: request.severity,
        recommendation: request.recommendation,
        is_builtin: false,
        is_enabled: true,
    })
}

/// Update an existing custom rule
#[tauri::command]
#[specta::specta]
pub async fn update_custom_rule(
    request: UpdateRuleRequest,
    app_state: State<'_, AppState>,
) -> Result<IssueGeneratorInfo, CommandError> {
    let repo = &app_state.extension_repository;
    
    // First check if this is a custom rule (not built-in)
    let existing_rule = repo.get_rule_by_id(&request.id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Rule not found: {}", e)))?;
    
    if existing_rule.is_builtin {
        return Err(CommandError::from(anyhow::anyhow!("Cannot modify built-in rules")));
    }
    
    // Use the repository to update
    repo.update_rule(
        &request.id,
        request.name.as_deref(),
        request.severity.as_deref(),
        request.is_enabled,
        request.recommendation.as_deref(),
    ).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to update rule: {}", e)))?;
    
    // Fetch the updated rule
    let rule = repo.get_rule_by_id(&request.id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to fetch updated rule: {}", e)))?;
    
    Ok(IssueGeneratorInfo {
        id: rule.id,
        name: rule.name,
        category: rule.category,
        severity: rule.severity,
        recommendation: rule.recommendation,
        is_builtin: rule.is_builtin,
        is_enabled: rule.is_enabled,
    })
}

/// Delete a custom rule
#[tauri::command]
#[specta::specta]
pub async fn delete_custom_rule(
    rule_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let repo = &app_state.extension_repository;
    
    // Check if this is a custom rule
    let existing_rule = repo.get_rule_by_id(&rule_id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Rule not found: {}", e)))?;
    
    if existing_rule.is_builtin {
        return Err(CommandError::from(anyhow::anyhow!("Cannot delete built-in rules")));
    }
    
    repo.delete_rule(&rule_id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to delete rule: {}", e)))?;
    
    Ok(())
}

/// Toggle a rule's enabled status
#[tauri::command]
#[specta::specta]
pub async fn toggle_rule_enabled(
    rule_id: String,
    enabled: bool,
    app_state: State<'_, AppState>,
) -> Result<IssueGeneratorInfo, CommandError> {
    let repo = &app_state.extension_repository;
    
    repo.set_rule_enabled(&rule_id, enabled).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to toggle rule: {}", e)))?;
    
    let rule = repo.get_rule_by_id(&rule_id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to fetch rule: {}", e)))?;
    
    Ok(IssueGeneratorInfo {
        id: rule.id,
        name: rule.name,
        category: rule.category,
        severity: rule.severity,
        recommendation: rule.recommendation,
        is_builtin: rule.is_builtin,
        is_enabled: rule.is_enabled,
    })
}

/// Reload extensions from database
#[tauri::command]
#[specta::specta]
pub async fn reload_extensions(
    app_state: State<'_, AppState>,
) -> Result<ExtensionSummary, CommandError> {
    // This would require mutable access to the registry
    // For now, just return the current summary
    get_extension_summary(app_state).await
}

/// Get extracted data for a specific page
#[tauri::command]
#[specta::specta]
pub async fn get_extracted_data(
    page_id: String,
    app_state: State<'_, AppState>,
) -> Result<ExtractedDataResponse, CommandError> {
    // This would query the database for stored extracted data
    // For now, return an empty response
    Ok(ExtractedDataResponse {
        page_id,
        url: String::new(),
        data: HashMap::new(),
        extracted_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Export data for external consumption
#[tauri::command]
#[specta::specta]
pub async fn export_page_data(
    page_id: String,
    format: String,
    app_state: State<'_, AppState>,
) -> Result<serde_json::Value, CommandError> {
    let registry = &app_state.extension_registry;
    
    // Get the exporters
    let exporter_ids = registry.get_data_exporter_ids();
    
    if exporter_ids.is_empty() {
        return Ok(serde_json::json!({
            "error": "No exporters registered",
            "page_id": page_id,
        }));
    }
    
    // For now, return a placeholder response
    Ok(serde_json::json!({
        "page_id": page_id,
        "format": format,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "status": "success",
    }))
}

/// Get extension capabilities information
#[tauri::command]
#[specta::specta]
pub async fn get_extension_capabilities() -> Result<Vec<CapabilityInfo>, CommandError> {
    Ok(vec![
        CapabilityInfo {
            name: "issue_generation".to_string(),
            display_name: "Issue Generation".to_string(),
            description: "Validate pages and generate SEO issues based on custom rules".to_string(),
        },
        CapabilityInfo {
            name: "data_extraction".to_string(),
            display_name: "Data Extraction".to_string(),
            description: "Extract structured data from HTML content".to_string(),
        },
        CapabilityInfo {
            name: "data_export".to_string(),
            display_name: "Data Export".to_string(),
            description: "Export extracted data for external consumption via API or webhooks".to_string(),
        },
    ])
}

/// Information about a capability
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CapabilityInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
}

// Legacy compatibility - these types and functions are kept for backward compatibility
// with existing frontend code

/// Legacy type alias for backward compatibility
pub type IssueRuleInfo = IssueGeneratorInfo;

/// Get all registered issue rules (legacy compatibility)
#[tauri::command]
#[specta::specta]
pub async fn get_all_issue_rules(
    app_state: State<'_, AppState>,
) -> Result<Vec<IssueRuleInfo>, CommandError> {
    get_all_issue_generators(app_state).await
}

/// Get all registered audit checks (legacy compatibility)
#[tauri::command]
#[specta::specta]
pub async fn get_all_audit_checks(
    app_state: State<'_, AppState>,
) -> Result<Vec<AuditCheckInfo>, CommandError> {
    let registry = &app_state.extension_registry;
    let check_keys = registry.get_audit_check_keys();
    
    let checks: Vec<AuditCheckInfo> = check_keys
        .into_iter()
        .map(|key| {
            AuditCheckInfo {
                key: key.clone(),
                label: key.clone(),
                category: "seo".to_string(),
                weight: 1.0,
                is_builtin: true,
                is_enabled: true,
            }
        })
        .collect();
    
    Ok(checks)
}

/// Information about an audit check for the frontend (legacy)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AuditCheckInfo {
    pub key: String,
    pub label: String,
    pub category: String,
    pub weight: f64,
    pub is_builtin: bool,
    pub is_enabled: bool,
}
