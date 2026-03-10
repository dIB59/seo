//! Extension Management Commands
//!
//! This module provides Tauri commands for managing SEO extensions,
//! including issue generators, data extractors, and data exporters.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use tauri::State;
use std::collections::HashMap;

use crate::error::CommandError;
use crate::lifecycle::app_state::AppState;
use crate::repository::ExtractorConfigInfo;

fn map_rule_info(rule: crate::contexts::IssueRuleInfo) -> IssueGeneratorInfo {
    IssueGeneratorInfo {
        id: rule.id,
        name: rule.name,
        category: rule.category,
        severity: rule.severity,
        rule_type: Some(rule.rule_type),
        target_field: rule.target_field,
        threshold_min: rule.threshold_min,
        threshold_max: rule.threshold_max,
        regex_pattern: rule.regex_pattern,
        recommendation: rule.recommendation,
        is_builtin: rule.is_builtin,
        is_enabled: rule.is_enabled,
    }
}

async fn reload_runtime_extensions(app_state: &AppState) -> Result<(), CommandError> {
    app_state
        .extension_registry
        .reload_from_repository(app_state.extension_repository.as_ref())
        .await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to reload extensions: {}", e)))
}

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
    pub rule_type: Option<String>,
    pub target_field: Option<String>,
    pub threshold_min: Option<f64>,
    pub threshold_max: Option<f64>,
    pub regex_pattern: Option<String>,
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

/// Registry entry describing a rule-targetable field independent of extractor internals
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RuleFieldInfo {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub target_field: String,
    pub kind: String,
    pub category_id: Option<String>,
    pub category_label: Option<String>,
    pub default_rule_severity: Option<String>,
    pub default_rule_recommendation: Option<String>,
    pub default_rule_threshold_min: Option<f64>,
    pub default_rule_threshold_max: Option<f64>,
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
    // HTML extraction fields
    pub selector: Option<String>,
    pub attribute: Option<String>,
    pub multiple: Option<bool>,
    pub min_count: Option<i32>,
    pub max_count: Option<i32>,
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
    pub expected_value: Option<String>,
    pub negate: Option<bool>,
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
    pub display_name: String,
    pub description: Option<String>,
    pub extractor_type: Option<String>,
    pub selector: String,
    pub attribute: Option<String>,
    pub category_id: Option<String>,
    pub category_label: Option<String>,
    pub default_rule_severity: Option<String>,
    pub default_rule_recommendation: Option<String>,
    pub default_rule_threshold_min: Option<f64>,
    pub default_rule_threshold_max: Option<f64>,
}

/// Request to update an existing extractor
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateExtractorRequest {
    pub id: String,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub extractor_type: Option<String>,
    pub selector: Option<String>,
    pub attribute: Option<String>,
    pub category_id: Option<String>,
    pub category_label: Option<String>,
    pub default_rule_severity: Option<String>,
    pub default_rule_recommendation: Option<String>,
    pub default_rule_threshold_min: Option<f64>,
    pub default_rule_threshold_max: Option<f64>,
}

fn normalize_extractor_type(extractor_type: Option<&str>) -> &'static str {
    match extractor_type.unwrap_or("css_selector") {
        "css" | "css_selector" => "css_selector",
        "xpath" => "xpath",
        "json" | "json_path" => "json_path",
        _ => "css_selector",
    }
}

fn parse_extractor_post_process(post_process: Option<&str>) -> Option<Value> {
    post_process.and_then(|value| serde_json::from_str::<Value>(value).ok())
}

fn get_post_process_string(post_process: &Value, key: &str) -> Option<String> {
    post_process
        .get(key)
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn get_post_process_number(post_process: &Value, key: &str) -> Option<f64> {
    post_process.get(key).and_then(|value| value.as_f64())
}

fn build_extractor_post_process(
    category_id: Option<&str>,
    category_label: Option<&str>,
    default_rule_severity: Option<&str>,
    default_rule_recommendation: Option<&str>,
    default_rule_threshold_min: Option<f64>,
    default_rule_threshold_max: Option<f64>,
) -> Option<String> {
    let category_id = category_id
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

    let category_label = category_label
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

    let default_rule_severity = default_rule_severity
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

    let default_rule_recommendation = default_rule_recommendation
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

    if category_id.is_none()
        && category_label.is_none()
        && default_rule_severity.is_none()
        && default_rule_recommendation.is_none()
        && default_rule_threshold_min.is_none()
        && default_rule_threshold_max.is_none()
    {
        return None;
    }

    Some(
        serde_json::json!({
            "category_id": category_id,
            "category_label": category_label,
            "default_rule_severity": default_rule_severity,
            "default_rule_recommendation": default_rule_recommendation,
            "default_rule_threshold_min": default_rule_threshold_min,
            "default_rule_threshold_max": default_rule_threshold_max,
        })
        .to_string(),
    )
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
        .map(map_rule_info)
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

/// Get all extractor configs from database (including custom ones)
#[tauri::command]
#[specta::specta]
pub async fn get_extractor_configs(
    app_state: State<'_, AppState>,
) -> Result<Vec<ExtractorConfigInfo>, CommandError> {
    let repo = &app_state.extension_repository;
    
    repo.get_all_extractors().await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to get extractors: {}", e)))
}

/// Get rule-targetable field registry entries
#[tauri::command]
#[specta::specta]
pub async fn get_rule_field_registry(
    app_state: State<'_, AppState>,
) -> Result<Vec<RuleFieldInfo>, CommandError> {
    let repo = &app_state.extension_repository;

    let extractors = repo
        .get_all_extractors()
        .await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to get extractors: {}", e)))?;

    let mut fields: Vec<RuleFieldInfo> = Vec::new();
    let mut category_index: HashMap<String, RuleFieldInfo> = HashMap::new();

    for extractor in extractors.into_iter().filter(|value| value.is_enabled) {
        let post_process = parse_extractor_post_process(extractor.post_process.as_deref());

        let category_id = post_process
            .as_ref()
            .and_then(|value| get_post_process_string(value, "category_id"));
        let category_label = post_process
            .as_ref()
            .and_then(|value| get_post_process_string(value, "category_label"));

        let default_rule_severity = post_process
            .as_ref()
            .and_then(|value| get_post_process_string(value, "default_rule_severity"));
        let default_rule_recommendation = post_process
            .as_ref()
            .and_then(|value| get_post_process_string(value, "default_rule_recommendation"));
        let default_rule_threshold_min = post_process
            .as_ref()
            .and_then(|value| get_post_process_number(value, "default_rule_threshold_min"));
        let default_rule_threshold_max = post_process
            .as_ref()
            .and_then(|value| get_post_process_number(value, "default_rule_threshold_max"));

        fields.push(RuleFieldInfo {
            id: format!("extractor:{}", extractor.name),
            label: extractor.display_name.clone(),
            description: extractor.description.clone(),
            target_field: format!("field:extractor:{}", extractor.name),
            kind: "extractor".to_string(),
            category_id: category_id.clone(),
            category_label: category_label.clone(),
            default_rule_severity: default_rule_severity.clone(),
            default_rule_recommendation: default_rule_recommendation.clone(),
            default_rule_threshold_min,
            default_rule_threshold_max,
        });

        if let Some(category_id) = category_id {
            category_index.entry(category_id.clone()).or_insert(RuleFieldInfo {
                id: format!("category:{}", category_id),
                label: category_label.clone().unwrap_or_else(|| category_id.replace('_', " ")),
                description: Some("Any extracted values in this category".to_string()),
                target_field: format!("field:category:{}", category_id),
                kind: "category".to_string(),
                category_id: Some(category_id),
                category_label,
                default_rule_severity: None,
                default_rule_recommendation: None,
                default_rule_threshold_min: None,
                default_rule_threshold_max: None,
            });
        }
    }

    fields.extend(category_index.into_values());
    fields.sort_by(|left, right| left.label.to_lowercase().cmp(&right.label.to_lowercase()));

    Ok(fields)
}

/// Create a new custom extractor
#[tauri::command]
#[specta::specta]
pub async fn create_custom_extractor(
    request: CreateExtractorRequest,
    app_state: State<'_, AppState>,
) -> Result<ExtractorConfigInfo, CommandError> {
    let repo = &app_state.extension_repository;
    
    let id = format!("custom-{}", uuid::Uuid::new_v4());
    let name = request.name.to_lowercase().replace(' ', "_");
    let extractor_type = normalize_extractor_type(request.extractor_type.as_deref());
    let post_process = build_extractor_post_process(
        request.category_id.as_deref(),
        request.category_label.as_deref(),
        request.default_rule_severity.as_deref(),
        request.default_rule_recommendation.as_deref(),
        request.default_rule_threshold_min,
        request.default_rule_threshold_max,
    );
    
    repo.insert_extractor(
        &id,
        &name,
        &request.display_name,
        request.description.as_deref(),
        extractor_type,
        &request.selector,
        request.attribute.as_deref(),
        post_process.as_deref(),
    ).await
    .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to create extractor: {}", e)))?;
    
    // Return the created extractor
    repo.get_extractor_by_id(&id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to get created extractor: {}", e)))
}

/// Update an existing custom extractor
#[tauri::command]
#[specta::specta]
pub async fn update_custom_extractor(
    request: UpdateExtractorRequest,
    app_state: State<'_, AppState>,
) -> Result<ExtractorConfigInfo, CommandError> {
    let repo = &app_state.extension_repository;
    
    // Check if this is a custom extractor
    let existing = repo.get_extractor_by_id(&request.id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Extractor not found: {}", e)))?;
    
    if existing.is_builtin {
        return Err(CommandError::from(anyhow::anyhow!("Cannot modify built-in extractors")));
    }

    let post_process = build_extractor_post_process(
        request.category_id.as_deref(),
        request.category_label.as_deref(),
        request.default_rule_severity.as_deref(),
        request.default_rule_recommendation.as_deref(),
        request.default_rule_threshold_min,
        request.default_rule_threshold_max,
    );
    
    repo.update_extractor(
        &request.id,
        request.name.as_deref(),
        request.display_name.as_deref(),
        request.description.as_deref(),
        request.extractor_type.as_deref().map(|value| normalize_extractor_type(Some(value))),
        request.selector.as_deref(),
        request.attribute.as_deref(),
        post_process.as_deref(),
    ).await
    .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to update extractor: {}", e)))?;
    
    // Return the updated extractor
    repo.get_extractor_by_id(&request.id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to get updated extractor: {}", e)))
}

/// Delete a custom extractor
#[tauri::command]
#[specta::specta]
pub async fn delete_custom_extractor(
    extractor_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let repo = &app_state.extension_repository;
    
    // Check if this is a custom extractor
    let existing = repo.get_extractor_by_id(&extractor_id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Extractor not found: {}", e)))?;
    
    if existing.is_builtin {
        return Err(CommandError::from(anyhow::anyhow!("Cannot delete built-in extractors")));
    }
    
    repo.delete_extractor(&extractor_id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to delete extractor: {}", e)))
}

/// Toggle an extractor's enabled status
#[tauri::command]
#[specta::specta]
pub async fn toggle_extractor_enabled(
    extractor_id: String,
    enabled: bool,
    app_state: State<'_, AppState>,
) -> Result<ExtractorConfigInfo, CommandError> {
    let repo = &app_state.extension_repository;
    
    repo.set_extractor_enabled(&extractor_id, enabled).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to toggle extractor: {}", e)))?;
    
    repo.get_extractor_by_id(&extractor_id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to get extractor: {}", e)))
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
        threshold_min: request.threshold_min,
        threshold_max: request.threshold_max,
        regex_pattern: request.regex_pattern.clone(),
        recommendation: request.recommendation.clone(),
        is_builtin: false,
        is_enabled: true,
    };
    
    repo.insert_rule(&new_rule).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to create custom rule: {}", e)))?;

    reload_runtime_extensions(&app_state).await?;

    let rule = repo.get_rule_by_id(&id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to fetch created rule: {}", e)))?;

    Ok(map_rule_info(rule))
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
        request.threshold_min,
        request.threshold_max,
        request.regex_pattern.as_deref(),
        request.is_enabled,
        request.recommendation.as_deref(),
    ).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to update rule: {}", e)))?;

    reload_runtime_extensions(&app_state).await?;
    
    // Fetch the updated rule
    let rule = repo.get_rule_by_id(&request.id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to fetch updated rule: {}", e)))?;
    
    Ok(map_rule_info(rule))
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

    reload_runtime_extensions(&app_state).await?;
    
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

    reload_runtime_extensions(&app_state).await?;
    
    let rule = repo.get_rule_by_id(&rule_id).await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to fetch rule: {}", e)))?;

    Ok(map_rule_info(rule))
}

/// Reload extensions from database
#[tauri::command]
#[specta::specta]
pub async fn reload_extensions(
    app_state: State<'_, AppState>,
) -> Result<ExtensionSummary, CommandError> {
    reload_runtime_extensions(&app_state).await?;
    get_extension_summary(app_state).await
}

/// Get extracted data for a specific page
#[tauri::command]
#[specta::specta]
pub async fn get_extracted_data(
    page_id: String,
    _app_state: State<'_, AppState>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RuleTargetMigrationResult {
    pub migrated_count: usize,
}

/// Normalize legacy rule target syntax to field:* format
#[tauri::command]
#[specta::specta]
pub async fn normalize_rule_target_fields(
    app_state: State<'_, AppState>,
) -> Result<RuleTargetMigrationResult, CommandError> {
    let repo = &app_state.extension_repository;
    let migrated_count = repo
        .migrate_rule_targets_to_field_format()
        .await
        .map_err(|e| CommandError::from(anyhow::anyhow!("Failed to normalize rule targets: {}", e)))?;

    Ok(RuleTargetMigrationResult { migrated_count })
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
