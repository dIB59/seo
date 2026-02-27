//! Tests for Custom Rule functionality
//!
//! This module tests the Create Custom Rule feature including:
//! - Category validation
//! - Rule type validation
//! - Custom issues functionality

use std::sync::Arc;

use chrono::Utc;
use sqlx::SqlitePool;

// Import from the app crate
use app::contexts::{IssueRuleInfo, Page};
use app::repository::{ExtensionRepositoryTrait, sqlite_extension_repo};

// ============================================================================
// Test Helpers
// ============================================================================

/// Creates an in-memory SQLite database with migrations applied for testing.
async fn setup_test_db() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    pool
}

/// Creates a test page with typical SEO data
#[allow(dead_code)]
fn make_test_page() -> Page {
    Page {
        id: "test-page-123".to_string(),
        job_id: "test-job-456".to_string(),
        url: "https://example.com/test-page".to_string(),
        depth: 0,
        status_code: Some(200),
        content_type: Some("text/html".to_string()),
        title: Some("Test Page Title".to_string()),
        meta_description: Some("This is a test meta description for testing purposes.".to_string()),
        canonical_url: Some("https://example.com/test-page".to_string()),
        robots_meta: Some("index,follow".to_string()),
        word_count: Some(500),
        load_time_ms: Some(1500),
        response_size_bytes: Some(1024),
        has_viewport: true,
        has_structured_data: false,
        crawled_at: Utc::now(),
        extracted_data: std::collections::HashMap::new(),
    }
}

// ============================================================================
// Category Validation Tests
// ============================================================================

#[tokio::test]
async fn test_valid_categories() {
    // Valid categories based on the codebase
    let valid_categories = vec!["seo", "performance", "accessibility", "security", "usability"];
    
    for category in valid_categories {
        // Test that categories are stored correctly in the database
        let pool = setup_test_db().await;
        let repo: Arc<dyn ExtensionRepositoryTrait> = sqlite_extension_repo(pool);
        
        let rule_id = format!("test-rule-{}", category);
        let new_rule = IssueRuleInfo {
            id: rule_id.clone(),
            name: "Test Rule".to_string(),
            category: category.to_string(),
            severity: "warning".to_string(),
            rule_type: "presence".to_string(),
            target_field: Some("title".to_string()),
            recommendation: Some("Test recommendation".to_string()),
            is_builtin: false,
            is_enabled: true,
        };
        
        let result = repo.insert_rule(&new_rule).await;
        
        assert!(result.is_ok(), "Category '{}' should be valid", category);
        
        // Verify the rule was inserted with correct category
        let fetched_rule = repo.get_rule_by_id(&rule_id).await;
        assert!(fetched_rule.is_ok());
        let rule = fetched_rule.unwrap();
        assert_eq!(rule.category, category, "Category should match what was inserted");
    }
}

#[tokio::test]
async fn test_invalid_category_handling() {
    // Invalid categories should still be stored (as the system allows custom categories)
    let pool = setup_test_db().await;
    let repo: Arc<dyn ExtensionRepositoryTrait> = sqlite_extension_repo(pool);
    
    let rule_id = "test-rule-invalid-category";
    let new_rule = IssueRuleInfo {
        id: rule_id.to_string(),
        name: "Test Rule".to_string(),
        category: "invalid-category-xyz".to_string(),
        severity: "warning".to_string(),
        rule_type: "presence".to_string(),
        target_field: Some("title".to_string()),
        recommendation: None,
        is_builtin: false,
        is_enabled: true,
    };
    
    // The system allows custom categories, so this should succeed
    let result = repo.insert_rule(&new_rule).await;
    assert!(result.is_ok());
}

// ============================================================================
// Rule Type Validation Tests
// ============================================================================

#[tokio::test]
async fn test_valid_rule_types() {
    // Valid rule types from loader.rs
    let valid_rule_types = vec!["presence", "threshold", "length"];
    
    for rule_type in valid_rule_types {
        let pool = setup_test_db().await;
        let repo: Arc<dyn ExtensionRepositoryTrait> = sqlite_extension_repo(pool);
        
        let rule_id = format!("test-rule-type-{}", rule_type);
        let new_rule = IssueRuleInfo {
            id: rule_id.clone(),
            name: "Test Rule".to_string(),
            category: "seo".to_string(),
            severity: "warning".to_string(),
            rule_type: rule_type.to_string(),
            target_field: Some("title".to_string()),
            recommendation: None,
            is_builtin: false,
            is_enabled: true,
        };
        
        let result = repo.insert_rule(&new_rule).await;
        
        assert!(result.is_ok(), "Rule type '{}' should be valid", rule_type);
        
        // Verify the rule was inserted with correct rule_type
        let fetched_rule = repo.get_rule_by_id(&rule_id).await;
        assert!(fetched_rule.is_ok());
        let rule = fetched_rule.unwrap();
        assert_eq!(rule.rule_type, rule_type, "Rule type should match what was inserted");
    }
}

// ============================================================================
// Custom Issues Tests
// ============================================================================

#[tokio::test]
async fn test_create_custom_rule_with_presence_type() {
    let pool = setup_test_db().await;
    let repo: Arc<dyn ExtensionRepositoryTrait> = sqlite_extension_repo(pool);
    
    let rule_id = "custom-test-presence";
    let new_rule = IssueRuleInfo {
        id: rule_id.to_string(),
        name: "Custom Presence Rule".to_string(),
        category: "seo".to_string(),
        severity: "warning".to_string(),
        rule_type: "presence".to_string(),
        target_field: Some("title".to_string()),
        recommendation: Some("Add a title to the page".to_string()),
        is_builtin: false,
        is_enabled: true,
    };
    
    let result = repo.insert_rule(&new_rule).await;
    
    assert!(result.is_ok());
    
    // Verify the rule was created
    let rule = repo.get_rule_by_id(rule_id).await.unwrap();
    assert_eq!(rule.name, "Custom Presence Rule");
    assert_eq!(rule.category, "seo");
    assert_eq!(rule.severity, "warning");
    assert_eq!(rule.rule_type, "presence");
    assert_eq!(rule.target_field, Some("title".to_string()));
    assert!(!rule.is_builtin);
    assert!(rule.is_enabled);
}

#[tokio::test]
async fn test_create_custom_rule_with_threshold_type() {
    let pool = setup_test_db().await;
    let repo: Arc<dyn ExtensionRepositoryTrait> = sqlite_extension_repo(pool);
    
    let rule_id = "custom-test-threshold";
    let new_rule = IssueRuleInfo {
        id: rule_id.to_string(),
        name: "Custom Threshold Rule".to_string(),
        category: "performance".to_string(),
        severity: "critical".to_string(),
        rule_type: "threshold".to_string(),
        target_field: Some("load_time_ms".to_string()),
        recommendation: Some("Optimize page load time".to_string()),
        is_builtin: false,
        is_enabled: true,
    };
    
    let result = repo.insert_rule(&new_rule).await;
    
    assert!(result.is_ok());
    
    // Verify the rule was created
    let rule = repo.get_rule_by_id(rule_id).await.unwrap();
    assert_eq!(rule.rule_type, "threshold");
    assert_eq!(rule.target_field, Some("load_time_ms".to_string()));
}

#[tokio::test]
async fn test_create_custom_rule_with_length_type() {
    let pool = setup_test_db().await;
    let repo: Arc<dyn ExtensionRepositoryTrait> = sqlite_extension_repo(pool);
    
    let rule_id = "custom-test-length";
    let new_rule = IssueRuleInfo {
        id: rule_id.to_string(),
        name: "Custom Length Rule".to_string(),
        category: "seo".to_string(),
        severity: "info".to_string(),
        rule_type: "length".to_string(),
        target_field: Some("meta_description".to_string()),
        recommendation: Some("Expand meta description".to_string()),
        is_builtin: false,
        is_enabled: true,
    };
    
    let result = repo.insert_rule(&new_rule).await;
    
    assert!(result.is_ok());
    
    // Verify the rule was created
    let rule = repo.get_rule_by_id(rule_id).await.unwrap();
    assert_eq!(rule.rule_type, "length");
    assert_eq!(rule.target_field, Some("meta_description".to_string()));
}

#[tokio::test]
async fn test_custom_rule_not_builtin() {
    let pool = setup_test_db().await;
    let repo: Arc<dyn ExtensionRepositoryTrait> = sqlite_extension_repo(pool);
    
    let rule_id = "custom-not-builtin";
    let new_rule = IssueRuleInfo {
        id: rule_id.to_string(),
        name: "Test Rule".to_string(),
        category: "seo".to_string(),
        severity: "warning".to_string(),
        rule_type: "presence".to_string(),
        target_field: Some("title".to_string()),
        recommendation: None,
        is_builtin: false,
        is_enabled: true,
    };
    
    let _ = repo.insert_rule(&new_rule).await;
    
    // Verify it's not a built-in rule
    let rule = repo.get_rule_by_id(rule_id).await.unwrap();
    assert!(!rule.is_builtin);
}

#[tokio::test]
async fn test_cannot_delete_builtin_rules() {
    let pool = setup_test_db().await;
    let repo: Arc<dyn ExtensionRepositoryTrait> = sqlite_extension_repo(pool.clone());
    
    // Insert a rule and mark it as builtin
    let rule_id = "builtin-rule-test";
    sqlx::query(
        r#"
        INSERT INTO audit_rules (
            id, name, category, severity, description, rule_type,
            target_field, is_enabled, is_builtin, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, 1, 1, datetime('now'), datetime('now'))
        "#,
    )
    .bind(rule_id)
    .bind("Builtin Rule")
    .bind("seo")
    .bind("warning")
    .bind("Test")
    .bind("presence")
    .bind("title")
    .execute(&pool)
    .await
    .expect("Failed to insert builtin rule");
    
    // Verify it's marked as builtin
    let rule = repo.get_rule_by_id(rule_id).await.unwrap();
    assert!(rule.is_builtin);
    
    // Attempting to delete a builtin rule should fail
    let delete_result = repo.delete_rule(rule_id).await;
    assert!(delete_result.is_err());
}

// ============================================================================
// Custom Rule Update Tests
// ============================================================================

#[tokio::test]
async fn test_custom_rule_update_functionality() {
    let pool = setup_test_db().await;
    let repo: Arc<dyn ExtensionRepositoryTrait> = sqlite_extension_repo(pool);
    
    // Create a rule
    let rule_id = "update-test-rule";
    let new_rule = IssueRuleInfo {
        id: rule_id.to_string(),
        name: "Original Name".to_string(),
        category: "seo".to_string(),
        severity: "warning".to_string(),
        rule_type: "presence".to_string(),
        target_field: Some("title".to_string()),
        recommendation: None,
        is_builtin: false,
        is_enabled: true,
    };
    
    let _ = repo.insert_rule(&new_rule).await;
    
    // Update the rule
    let update_result = repo
        .update_rule(
            rule_id,
            Some("Updated Name"),
            Some("critical"),
            Some(true),
            Some("Updated recommendation"),
        )
        .await;
    
    assert!(update_result.is_ok());
    
    // Verify the update
    let updated_rule = repo.get_rule_by_id(rule_id).await.unwrap();
    assert_eq!(updated_rule.name, "Updated Name");
    assert_eq!(updated_rule.severity, "critical");
    assert_eq!(updated_rule.recommendation, Some("Updated recommendation".to_string()));
}

#[tokio::test]
async fn test_custom_rule_enable_disable() {
    let pool = setup_test_db().await;
    let repo: Arc<dyn ExtensionRepositoryTrait> = sqlite_extension_repo(pool);
    
    // Create a rule
    let rule_id = "toggle-test-rule";
    let new_rule = IssueRuleInfo {
        id: rule_id.to_string(),
        name: "Toggle Test".to_string(),
        category: "seo".to_string(),
        severity: "warning".to_string(),
        rule_type: "presence".to_string(),
        target_field: Some("title".to_string()),
        recommendation: None,
        is_builtin: false,
        is_enabled: true,
    };
    
    let _ = repo.insert_rule(&new_rule).await;
    
    // Verify it's initially enabled
    let rule = repo.get_rule_by_id(rule_id).await.unwrap();
    assert!(rule.is_enabled);
    
    // Disable the rule
    let disable_result = repo.set_rule_enabled(rule_id, false).await;
    assert!(disable_result.is_ok());
    
    let disabled_rule = repo.get_rule_by_id(rule_id).await.unwrap();
    assert!(!disabled_rule.is_enabled);
    
    // Re-enable the rule
    let enable_result = repo.set_rule_enabled(rule_id, true).await;
    assert!(enable_result.is_ok());
    
    let enabled_rule = repo.get_rule_by_id(rule_id).await.unwrap();
    assert!(enabled_rule.is_enabled);
}

// ============================================================================
// Count Custom Rules Tests
// ============================================================================

#[tokio::test]
async fn test_count_custom_rules() {
    let pool = setup_test_db().await;
    let repo: Arc<dyn ExtensionRepositoryTrait> = sqlite_extension_repo(pool);
    
    // Initially should have 0 custom rules
    let initial_count = repo.count_custom_rules().await.unwrap();
    assert_eq!(initial_count, 0);
    
    // Add 5 custom rules
    for i in 0..5 {
        let rule_id = format!("count-test-{}", i);
        let new_rule = IssueRuleInfo {
            id: rule_id,
            name: format!("Count Test {}", i),
            category: "seo".to_string(),
            severity: "warning".to_string(),
            rule_type: "presence".to_string(),
            target_field: Some("title".to_string()),
            recommendation: None,
            is_builtin: false,
            is_enabled: true,
        };
        
        let _ = repo.insert_rule(&new_rule).await;
    }
    
    // Should now have 5 custom rules
    let count = repo.count_custom_rules().await.unwrap();
    assert_eq!(count, 5);
    
    // Delete 2 rules
    let _ = repo.delete_rule("count-test-0").await;
    let _ = repo.delete_rule("count-test-1").await;
    
    // Should now have 3 custom rules
    let final_count = repo.count_custom_rules().await.unwrap();
    assert_eq!(final_count, 3);
}

// ============================================================================
// Get All Rules Tests
// ============================================================================

#[tokio::test]
async fn test_get_all_rules() {
    let pool = setup_test_db().await;
    let repo: Arc<dyn ExtensionRepositoryTrait> = sqlite_extension_repo(pool);
    
    // Initially should have some rules (possibly from builtins or previous tests)
    let initial_rules = repo.get_all_rules().await.unwrap();
    let initial_count = initial_rules.len();
    
    // Add some custom rules
    for i in 0..3 {
        let rule_id = format!("getall-test-{}", i);
        let new_rule = IssueRuleInfo {
            id: rule_id,
            name: format!("Test Rule {}", i),
            category: "seo".to_string(),
            severity: "warning".to_string(),
            rule_type: "presence".to_string(),
            target_field: Some("title".to_string()),
            recommendation: None,
            is_builtin: false,
            is_enabled: true,
        };
        
        let _ = repo.insert_rule(&new_rule).await;
    }
    
    // Should now have initial + 3 rules
    let rules = repo.get_all_rules().await.unwrap();
    assert_eq!(rules.len(), initial_count + 3);
}

// ============================================================================
// Get Rule By ID Tests
// ============================================================================

#[tokio::test]
async fn test_get_rule_by_id_not_found() {
    let pool = setup_test_db().await;
    let repo: Arc<dyn ExtensionRepositoryTrait> = sqlite_extension_repo(pool);
    
    // Try to get a non-existent rule
    let result = repo.get_rule_by_id("non-existent-rule").await;
    
    // Should fail
    assert!(result.is_err());
}

// ============================================================================
// Delete Non-Existent Rule Tests
// ============================================================================

#[tokio::test]
async fn test_delete_non_existent_rule() {
    let pool = setup_test_db().await;
    let repo: Arc<dyn ExtensionRepositoryTrait> = sqlite_extension_repo(pool);
    
    // Try to delete a non-existent rule
    let result = repo.delete_rule("non-existent-rule").await;
    
    // Should fail
    assert!(result.is_err());
}
