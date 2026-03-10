//! Extension bounded context.
//!
//! This context owns the full extension runtime: capabilities, execution
//! pipeline, registry, built-ins, and domain types.

pub mod capabilities;
pub mod context;
pub mod result;
pub mod traits;
pub mod pipeline;
pub mod domain;
pub mod builtins;

mod loader;

pub use builtins::register_builtins;
pub use capabilities::{CapabilityConfig, CapabilityMetadata, ExtensionCapability};
pub use context::{ExportContext, ExportFormat, ExtractionContext, ValidationContext};
pub use domain::{
	AuditCheck, AuditContext, CheckResult, CssSelectorExtractor, EvaluationContext,
	ExtractedData, HrefTagExtractor, HtmlExtractionConfig, HtmlExtractionRule, IssueRule,
	PageDataExtractor, RuleCondition, RuleType,
};
pub use domain::{
	CanonicalCheck, CrawlableAnchorsCheck, HreflangCheck, HttpStatusCodeCheck, ImageAltCheck,
	KeywordExtractor, LengthRule, LinkTextCheck, MetaDescriptionCheck, OpenGraphExtractor,
	PresenceRule, RegexRule, RobotsMetaCheck, StatusCodeRule, StructuredDataExtractor,
	ThresholdRule, TitleCheck, TwitterCardExtractor, ValidationReason,
	ValidationResult as DomainValidationResult, ViewportCheck,
};
pub use loader::ExtensionLoader;
pub use pipeline::ExtensionPipeline;
pub use result::{
	ExportResult, ExtractedValue, ExtractionMetadata, ExtractionResult, PipelineResult,
	ValidationResult,
};
pub use traits::{
	DataExporter, DataExtractor, Extension, ExtensionConfig, ExtractionSchema,
	IssueGenerator, SchemaField, SchemaFieldType,
};

use anyhow::Result;
use dashmap::DashMap;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::contexts::analysis::{NewIssue, Page};

/// Central registry for all extensions.
pub struct ExtensionRegistry {
	pipeline: ExtensionPipeline,
	configs: DashMap<String, ExtensionConfig>,
}

impl ExtensionRegistry {
	fn ids_for_capability(&self, capability: ExtensionCapability) -> Vec<String> {
		self.configs
			.iter()
			.filter(|entry| entry.capabilities.has_capability(capability))
			.map(|entry| entry.id.clone())
			.collect()
	}

	pub fn new() -> Self {
		Self {
			pipeline: ExtensionPipeline::new(),
			configs: DashMap::new(),
		}
	}

	pub async fn load_from_database(pool: &SqlitePool) -> Result<Self> {
		let registry = Self::new();

		registry.register_builtin_extensions();

		let loader = ExtensionLoader::new(pool);

		match loader.load_issue_rules().await {
			Ok(rules) => {
				for rule in rules {
					registry.pipeline.register_validator(rule.clone());
					tracing::debug!("Loaded custom rule: {}", rule.id());
				}
			}
			Err(error) => {
				tracing::warn!("Failed to load custom rules: {}", error);
			}
		}

		match loader.load_custom_extractors().await {
			Ok(extractors) => {
				for extractor in extractors {
					registry.pipeline.register_extractor(extractor.clone());
					tracing::debug!("Loaded custom extractor: {}", extractor.id());
				}
			}
			Err(error) => {
				tracing::warn!("Failed to load custom extractors: {}", error);
			}
		}

		let (extractors, validators, exporters) = registry.pipeline.counts();
		tracing::info!(
			"Extension registry loaded: {} extractors, {} validators, {} exporters",
			extractors,
			validators,
			exporters
		);

		Ok(registry)
	}

	pub fn register_builtin_extensions(&self) {
		builtins::register_builtins(&self.pipeline);
	}

	pub fn register_validator(&self, validator: Arc<dyn IssueGenerator>) {
		let config = validator.config();
		self.configs.insert(config.id.clone(), config);
		self.pipeline.register_validator(validator);
	}

	pub fn register_extractor(&self, extractor: Arc<dyn DataExtractor>) {
		let config = extractor.config();
		self.configs.insert(config.id.clone(), config);
		self.pipeline.register_extractor(extractor);
	}

	pub fn register_exporter(&self, exporter: Arc<dyn DataExporter>) {
		let config = exporter.config();
		self.configs.insert(config.id.clone(), config);
		self.pipeline.register_exporter(exporter);
	}

	pub fn unregister(&self, id: &str) {
		self.pipeline.unregister(id);
		self.configs.remove(id);
	}

	pub fn get_config(&self, id: &str) -> Option<ExtensionConfig> {
		self.configs.get(id).map(|entry| entry.value().clone())
	}

	pub fn get_all_configs(&self) -> Vec<ExtensionConfig> {
		self.configs
			.iter()
			.map(|entry| entry.value().clone())
			.collect()
	}

	pub async fn execute(&self, page: &Page, html: &str) -> Result<PipelineResult> {
		Ok(self.pipeline.execute(page, html).await)
	}

	pub async fn extract_and_validate(
		&self,
		page: &Page,
		html: &str,
	) -> (std::collections::HashMap<String, serde_json::Value>, Vec<NewIssue>) {
		self.pipeline.extract_and_validate(page, html).await
	}

	pub fn evaluate_rules(&self, _page: &Page, _context: &ValidationContext) -> Vec<NewIssue> {
		Vec::new()
	}

	pub fn get_issue_rule_ids(&self) -> Vec<String> {
		self.ids_for_capability(ExtensionCapability::IssueGeneration)
	}

	pub fn get_data_extractor_ids(&self) -> Vec<String> {
		self.ids_for_capability(ExtensionCapability::DataExtraction)
	}

	pub fn get_data_exporter_ids(&self) -> Vec<String> {
		self.ids_for_capability(ExtensionCapability::DataExport)
	}

	pub fn get_audit_check_keys(&self) -> Vec<String> {
		self.get_issue_rule_ids()
	}

	pub fn run_extractor(
		&self,
		_id: &str,
		html: &str,
		url: &str,
	) -> Option<std::collections::HashMap<String, serde_json::Value>> {
		let _context = ExtractionContext::new(
			html.to_string(),
			url.to_string(),
			String::new(),
			String::new(),
		);

		None
	}

	pub fn rule_count(&self) -> usize {
		self.get_issue_rule_ids().len()
	}

	pub fn extractor_count(&self) -> usize {
		self.get_data_extractor_ids().len()
	}

	pub fn exporter_count(&self) -> usize {
		self.get_data_exporter_ids().len()
	}

	pub fn counts(&self) -> (usize, usize, usize) {
		self.pipeline.counts()
	}
}

impl Default for ExtensionRegistry {
	fn default() -> Self {
		Self::new()
	}
}