//! Extension Management Commands
//!
//! This module provides Tauri commands for managing SEO extensions,
//! including issue rules, data extractors, and audit checks.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::contexts::extension::domain::issue_rule::IssueRuleInfo;
use crate::error::CommandError;
use crate::lifecycle::app_state::AppState;

// ============================================================================
// Response Types
// ============================================================================


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

/// Information about an audit check for the frontend
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AuditCheckInfo {
    pub key: String,
    pub label: String,
    pub category: String,
    pub weight: f64,
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

/// Summary of extension system status
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExtensionSummary {
    pub total_rules: usize,
    pub total_extractors: usize,
    pub total_checks: usize,
    pub builtin_rules: usize,
    pub custom_rules: usize,
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
    
    Ok(ExtensionSummary {
        total_rules: registry.rule_count(),
        total_extractors: registry.extractor_count(),
        total_checks: registry.check_count(),
        builtin_rules: registry.rule_count().saturating_sub(custom_rules),
        custom_rules,
    })
}

/// Get all registered issue rules
#[tauri::command]
#[specta::specta]
pub async fn get_all_issue_rules(
    app_state: State<'_, AppState>,
) -> Result<Vec<IssueRuleInfo>, CommandError> {
    let repo = &app_state.extension_repository;
    let rules = repo.get_all_rules().await
        .map_err(|e| CommandError::from(anyhow::anyhow!("{}", e)))?;
    Ok(rules)
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
            DataExtractorInfo {
                id: id.clone(),
                name: id.clone(),
                description: None,
                extractor_type: "css_selector".to_string(),
                is_builtin: true,
                is_enabled: true,
            }
        })
        .collect();
    
    Ok(extractors)
}

/// Get all registered audit checks
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

/// Create a new custom issue rule
#[tauri::command]
#[specta::specta]
pub async fn create_custom_rule(
    request: CreateRuleRequest,
    app_state: State<'_, AppState>,
) -> Result<IssueRuleInfo, CommandError> {
    let repo = &app_state.extension_repository;
    
    let id = format!("custom-{}", uuid::Uuid::new_v4());
    
    let new_rule = IssueRuleInfo {
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
    
    Ok(new_rule)
}

/// Update an existing custom rule
#[tauri::command]
#[specta::specta]
pub async fn update_custom_rule(
    request: UpdateRuleRequest,
    app_state: State<'_, AppState>,
) -> Result<IssueRuleInfo, CommandError> {
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
    Ok(rule)
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
) -> Result<IssueRuleInfo, CommandError> {
    let repo = &app_state.extension_repository;
    
    repo.set_rule_enabled(&rule_id, enabled).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to toggle rule: {}", e)))?;
    
    let rule = repo.get_rule_by_id(&rule_id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to fetch rule: {}", e)))?;
    Ok(rule)
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
