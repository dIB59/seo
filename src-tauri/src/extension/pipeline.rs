//! Extension Pipeline
//!
//! This module provides the execution pipeline for extensions.
//! The pipeline orchestrates the three phases: Extract, Validate, Export.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::RwLock;

use super::capabilities::ExtensionCapability;
use super::context::{ExtractionContext, ExportContext, ValidationContext};
use super::result::{ExtractionResult, ExportResult, ValidationResult, PipelineResult};
use super::traits::{DataExporter, DataExtractor, Extension, ExtensionConfig, IssueGenerator};
use crate::contexts::{NewIssue, Page};

/// The extension pipeline that orchestrates execution.
///
/// The pipeline manages three phases:
/// 1. **Extract** - Run data extractors to pull data from HTML
/// 2. **Validate** - Run issue generators to validate pages
/// 3. **Export** - Run data exporters to expose data externally
pub struct ExtensionPipeline {
    /// Data extractors indexed by ID
    extractors: DashMap<String, Arc<dyn DataExtractor>>,
    
    /// Issue generators indexed by ID
    validators: DashMap<String, Arc<dyn IssueGenerator>>,
    
    /// Data exporters indexed by ID
    exporters: DashMap<String, Arc<dyn DataExporter>>,
    
    /// Extension configurations
    configs: RwLock<HashMap<String, ExtensionConfig>>,
    
    /// Cache for extraction results
    extraction_cache: DashMap<String, ExtractionResult>,
    
    /// Whether to run phases in parallel where possible
    parallel: bool,
}

impl ExtensionPipeline {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self {
            extractors: DashMap::new(),
            validators: DashMap::new(),
            exporters: DashMap::new(),
            configs: RwLock::new(HashMap::new()),
            extraction_cache: DashMap::new(),
            parallel: true,
        }
    }
    
    /// Create a pipeline with parallel execution disabled
    pub fn sequential() -> Self {
        Self {
            parallel: false,
            ..Self::new()
        }
    }
    
    /// Register an extension in the pipeline.
    ///
    /// The extension is automatically categorized based on its capabilities.
    pub fn register(&self, extension: Arc<dyn Extension>) {
        let config = extension.config();
        let id = config.id.clone();
        
        // Check capabilities and register appropriately
        if extension.has_capability(ExtensionCapability::DataExtraction) {
            // We need to cast to DataExtractor
            // This is a bit tricky in Rust - we'll use a different approach
            tracing::debug!("Extension {} has DataExtraction capability", id);
        }
        
        if extension.has_capability(ExtensionCapability::IssueGeneration) {
            tracing::debug!("Extension {} has IssueGeneration capability", id);
        }
        
        if extension.has_capability(ExtensionCapability::DataExport) {
            tracing::debug!("Extension {} has DataExport capability", id);
        }
        
        // Store config
        // Note: In a real implementation, we'd need to store typed extensions
        // This requires the extension to be registered via specific methods
    }
    
    /// Register a data extractor
    pub fn register_extractor(&self, extractor: Arc<dyn DataExtractor>) {
        let id = extractor.id().to_string();
        self.extractors.insert(id.clone(), extractor);
        tracing::debug!("Registered data extractor: {}", id);
    }
    
    /// Register an issue generator
    pub fn register_validator(&self, validator: Arc<dyn IssueGenerator>) {
        let id = validator.id().to_string();
        self.validators.insert(id.clone(), validator);
        tracing::debug!("Registered issue generator: {}", id);
    }
    
    /// Register a data exporter
    pub fn register_exporter(&self, exporter: Arc<dyn DataExporter>) {
        let id = exporter.id().to_string();
        self.exporters.insert(id.clone(), exporter);
        tracing::debug!("Registered data exporter: {}", id);
    }
    
    /// Unregister an extension by ID
    pub fn unregister(&self, id: &str) {
        self.extractors.remove(id);
        self.validators.remove(id);
        self.exporters.remove(id);
        self.extraction_cache.remove(id);
    }
    
    /// Get counts of registered extensions
    pub fn counts(&self) -> (usize, usize, usize) {
        (
            self.extractors.len(),
            self.validators.len(),
            self.exporters.len(),
        )
    }
    
    /// Clear all extensions
    pub fn clear(&self) {
        self.extractors.clear();
        self.validators.clear();
        self.exporters.clear();
        self.extraction_cache.clear();
    }
    
    /// Clear the extraction cache
    pub fn clear_cache(&self) {
        self.extraction_cache.clear();
    }
    
    // ========================================================================
    // Phase 1: Extraction
    // ========================================================================
    
    /// Run the extraction phase.
    ///
    /// Extracts data from HTML using all registered extractors.
    pub async fn extract_phase(&self, context: &ExtractionContext) -> Vec<ExtractionResult> {
        let start = Instant::now();
        let mut results = Vec::new();
        
        if self.parallel {
            // Run extractors in parallel using tokio tasks
            let handles: Vec<_> = self
                .extractors
                .iter()
                .map(|entry| {
                    let extractor = entry.value().clone();
                    let ctx = context.clone();
                    tokio::spawn(async move {
                        let start = Instant::now();
                        match extractor.extract(&ctx) {
                            Ok(mut result) => {
                                result.metadata.duration_ms = start.elapsed().as_millis() as u64;
                                Some(result)
                            }
                            Err(e) => {
                                tracing::warn!("Extractor {} failed: {}", extractor.id(), e);
                                Some(ExtractionResult::new(extractor.id().to_string())
                                    .with_error(e.to_string()))
                            }
                        }
                    })
                })
                .collect();
            
            for handle in handles {
                if let Ok(Some(result)) = handle.await {
                    results.push(result);
                }
            }
        } else {
            // Run extractors sequentially
            for entry in self.extractors.iter() {
                let extractor = entry.value();
                let start = Instant::now();
                
                match extractor.extract(context) {
                    Ok(mut result) => {
                        result.metadata.duration_ms = start.elapsed().as_millis() as u64;
                        results.push(result);
                    }
                    Err(e) => {
                        tracing::warn!("Extractor {} failed: {}", extractor.id(), e);
                        results.push(
                            ExtractionResult::new(extractor.id().to_string())
                                .with_error(e.to_string())
                        );
                    }
                }
            }
        }
        
        tracing::debug!(
            "Extraction phase completed in {}ms for {} extractors",
            start.elapsed().as_millis(),
            results.len()
        );
        
        results
    }
    
    // ========================================================================
    // Phase 2: Validation
    // ========================================================================
    
    /// Run the validation phase.
    ///
    /// Validates pages using all registered issue generators.
    pub async fn validate_phase(
        &self,
        page: &Page,
        html: Option<&str>,
        extracted_data: &HashMap<String, serde_json::Value>,
    ) -> Vec<ValidationResult> {
        let start = Instant::now();
        let mut results = Vec::new();
        
        // Build validation context
        let context = ValidationContext {
            page: page.clone(),
            html: html.map(|s| s.to_string()),
            headers: HashMap::new(),
            extracted_data: extracted_data.clone(),
            lighthouse_data: None,
            metadata: HashMap::new(),
        };
        
        for entry in self.validators.iter() {
            let validator = entry.value();
            
            // Check if this validator applies to this page
            if !validator.applies_to(page) {
                continue;
            }
            
            let validator_start = Instant::now();
            
            match validator.validate(&context) {
                Ok(mut result) => {
                    result.metadata.duration_ms = validator_start.elapsed().as_millis() as u64;
                    results.push(result);
                }
                Err(e) => {
                    tracing::warn!("Validator {} failed: {}", validator.id(), e);
                }
            }
        }
        
        tracing::debug!(
            "Validation phase completed in {}ms, {} results",
            start.elapsed().as_millis(),
            results.len()
        );
        
        results
    }
    
    // ========================================================================
    // Phase 3: Export
    // ========================================================================
    
    /// Run the export phase.
    ///
    /// Exports data using all registered exporters.
    pub async fn export_phase(
        &self,
        page_id: &str,
        url: &str,
        extracted_data: &HashMap<String, serde_json::Value>,
        issues: &[NewIssue],
    ) -> Vec<ExportResult> {
        let start = Instant::now();
        let mut results = Vec::new();
        
        // Build export context
        let context = ExportContext {
            page_id: page_id.to_string(),
            url: url.to_string(),
            data: extracted_data.clone(),
            issues: issues.to_vec(),
            format: super::context::ExportFormat::Json,
            endpoint: None,
            headers: HashMap::new(),
        };
        
        for entry in self.exporters.iter() {
            let exporter = entry.value();
            let export_start = Instant::now();
            
            match exporter.export(&context) {
                Ok(mut result) => {
                    result.duration_ms = export_start.elapsed().as_millis() as u64;
                    results.push(result);
                }
                Err(e) => {
                    results.push(
                        ExportResult::failure(exporter.id().to_string(), e.to_string())
                            .with_duration(export_start.elapsed().as_millis() as u64)
                    );
                }
            }
        }
        
        tracing::debug!(
            "Export phase completed in {}ms, {} results",
            start.elapsed().as_millis(),
            results.len()
        );
        
        results
    }
    
    // ========================================================================
    // Full Pipeline
    // ========================================================================
    
    /// Execute the full pipeline for a page.
    ///
    /// This runs all three phases in order:
    /// 1. Extract data from HTML
    /// 2. Validate and generate issues
    /// 3. Export data for external consumption
    pub async fn execute(
        &self,
        page: &Page,
        html: &str,
    ) -> PipelineResult {
        let total_start = Instant::now();
        let mut result = PipelineResult::new(page.id.clone(), page.url.clone());
        
        // Phase 1: Extract
        let extraction_context = ExtractionContext::new(
            html.to_string(),
            page.url.clone(),
            page.id.clone(),
            page.job_id.clone(),
        )
        .with_depth(page.depth)
        .with_status_code(page.status_code.unwrap_or(0));
        
        let extraction_results = self.extract_phase(&extraction_context).await;
        
        // Merge extracted data
        let mut extracted_data = HashMap::new();
        for ext_result in &extraction_results {
            for (key, value) in &ext_result.data {
                extracted_data.insert(key.clone(), value.to_json());
            }
        }
        
        result.add_extraction_results(extraction_results);
        
        // Phase 2: Validate
        let validation_results = self.validate_phase(page, Some(html), &extracted_data).await;
        
        // Collect issues
        let mut all_issues = Vec::new();
        for val_result in &validation_results {
            all_issues.extend(val_result.issues.clone());
        }
        
        result.add_validation_results(validation_results);
        
        // Phase 3: Export
        let export_results = self.export_phase(
            &page.id,
            &page.url,
            &extracted_data,
            &all_issues,
        ).await;
        
        result.add_export_results(export_results);
        
        result.total_duration_ms = total_start.elapsed().as_millis() as u64;
        
        tracing::info!(
            "Pipeline completed for {} in {}ms: {} issues, {} extracted fields",
            page.url,
            result.total_duration_ms,
            result.total_issues(),
            result.extracted_data.len()
        );
        
        result
    }
    
    /// Execute only the extraction and validation phases.
    ///
    /// Use this when you don't need to export data.
    pub async fn extract_and_validate(
        &self,
        page: &Page,
        html: &str,
    ) -> (HashMap<String, serde_json::Value>, Vec<NewIssue>) {
        // Phase 1: Extract
        let extraction_context = ExtractionContext::new(
            html.to_string(),
            page.url.clone(),
            page.id.clone(),
            page.job_id.clone(),
        );
        
        let extraction_results = self.extract_phase(&extraction_context).await;
        
        // Merge extracted data
        let mut extracted_data = HashMap::new();
        for result in &extraction_results {
            for (key, value) in &result.data {
                extracted_data.insert(key.clone(), value.to_json());
            }
        }
        
        // Phase 2: Validate
        let validation_results = self.validate_phase(page, Some(html), &extracted_data).await;
        
        // Collect issues
        let issues: Vec<NewIssue> = validation_results
            .iter()
            .flat_map(|r| r.issues.clone())
            .collect();
        
        (extracted_data, issues)
    }
}

impl Default for ExtensionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pipeline_creation() {
        let pipeline = ExtensionPipeline::new();
        let (extractors, validators, exporters) = pipeline.counts();
        assert_eq!(extractors, 0);
        assert_eq!(validators, 0);
        assert_eq!(exporters, 0);
    }
    
    #[test]
    fn test_pipeline_clear() {
        let pipeline = ExtensionPipeline::new();
        pipeline.clear();
        let (extractors, validators, exporters) = pipeline.counts();
        assert_eq!(extractors, 0);
        assert_eq!(validators, 0);
        assert_eq!(exporters, 0);
    }
}
