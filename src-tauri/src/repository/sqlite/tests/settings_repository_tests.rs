use crate::test_utils::fixtures;

#[tokio::test]
async fn test_get_set_setting_key_value() {
    let pool = fixtures::setup_test_db().await;
    let repo = crate::repository::sqlite::SettingsRepository::new(pool.clone());

    // Ensure no key exists initially
    let v = repo.get_setting("gemini_prompt_blocks").await.unwrap();
    assert!(v.is_none());

    // Set and get
    repo.set_setting("gemini_prompt_blocks", "[]").await.unwrap();
    let v = repo.get_setting("gemini_prompt_blocks").await.unwrap();
    assert_eq!(v.unwrap(), "[]");

    // Alias key mapping
    repo.set_setting("google_api_key", "gkey").await.unwrap();
    let v = repo.get_setting("gemini_api_key").await.unwrap();
    assert_eq!(v.unwrap(), "gkey");
}
