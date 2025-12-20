use crate::domain::models::{IssueType, JobStatus};

mod issues_repository;
mod job_repository;
mod page_repository;
mod results_repository;
mod settings_repository;
mod summary_repository;

pub use issues_repository::IssuesRepository;
pub use job_repository::JobRepository;
pub use page_repository::PageRepository;
pub use results_repository::ResultsRepository;
pub use settings_repository::SettingsRepository;
pub use summary_repository::SummaryRepository;

pub fn map_job_status(s: &str) -> JobStatus {
    match s {
        "queued" => JobStatus::Queued,
        "processing" => JobStatus::Processing,
        "completed" => JobStatus::Completed,
        "failed" => JobStatus::Failed,
        _ => JobStatus::Queued,
    }
}

pub fn map_issue_type(s: &str) -> IssueType {
    match s {
        "critical" => IssueType::Critical,
        "warning" => IssueType::Warning,
        "suggestion" => IssueType::Suggestion,
        _ => IssueType::Suggestion,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::models::*,
        repository::sqlite::{
            job_repository::JobRepository, page_repository::PageRepository,
            results_repository::ResultsRepository, settings_repository::SettingsRepository,
            summary_repository::SummaryRepository,
        },
        test_utils::fixtures,
    };

    #[tokio::test]
    async fn test_job_lifecycle() {
        let pool = fixtures::setup_test_db().await;
        let repo = JobRepository::new(pool.clone());

        // Use shared fixture helper
        let settings = fixtures::settings_with_max_pages(5);

        // 1. Create
        let job_id = repo
            .create_with_settings("https://test.com", &settings)
            .await
            .expect("Failed to create job");

        // 2. Verify Pending
        let pending = repo
            .get_pending_jobs()
            .await
            .expect("Failed to get pending");

        assert_eq!(pending.len(), 1, "Should have one pending job");
        assert_eq!(pending[0].id, job_id);
        assert_eq!(pending[0].status, JobStatus::Queued);

        // 3. Update Status
        repo.update_status(job_id, JobStatus::Processing)
            .await
            .expect("Update status failed");

        let pending_processing = repo.get_pending_jobs().await.unwrap();
        assert_eq!(pending_processing[0].status, JobStatus::Processing);

        // 4. Complete
        repo.update_status(job_id, JobStatus::Completed)
            .await
            .expect("Update status failed");

        let pending_final = repo.get_pending_jobs().await.unwrap();
        assert!(
            pending_final.is_empty(),
            "Completed jobs should not appear in pending"
        );
    }

    #[tokio::test]
    async fn test_settings_persistence() {
        let pool = fixtures::setup_test_db().await;
        let job_repo = JobRepository::new(pool.clone());
        let settings_repo = SettingsRepository::new(pool.clone());

        let settings = fixtures::settings_with_max_pages(42);

        // Creating a job also creates settings
        let _job_id = job_repo
            .create_with_settings("https://settings.test", &settings)
            .await
            .unwrap();

        let pending = job_repo.get_pending_jobs().await.unwrap();
        let settings_id = pending[0].settings_id;

        let retrieved = settings_repo.get_by_id(settings_id).await.unwrap();
        assert_eq!(retrieved.max_pages, 42, "Settings should persist correctly");
    }

    #[tokio::test]
    async fn test_results_and_pages() {
        let pool = fixtures::setup_test_db().await;
        let results_repo = ResultsRepository::new(pool.clone());
        let page_repo = PageRepository::new(pool.clone());
        let job_repo = JobRepository::new(pool.clone());

        let settings = fixtures::default_settings();
        let job_id = job_repo
            .create_with_settings("https://result.test", &settings)
            .await
            .unwrap();

        // 1. Create Result
        let result_id = results_repo
            .create(
                "https://result.test",
                true, // sitemap
                true, // robots
                true, // ssl
            )
            .await
            .expect("Failed to create result");

        // Link
        job_repo.link_to_result(job_id, &result_id).await.unwrap();

        // 2. Add a page
        let page_data = PageAnalysisData {
            analysis_id: result_id.clone(),
            url: "https://result.test/page1".into(),
            title: Some("Page 1".into()),
            meta_description: None,
            meta_keywords: None,
            canonical_url: None,
            h1_count: 1,
            h2_count: 0,
            h3_count: 0,
            word_count: 100,
            image_count: 1,
            images_without_alt: 0,
            internal_links: 0,
            external_links: 0,
            load_time: 0.2,
            status_code: Some(200),
            content_size: 500,
            mobile_friendly: true,
            has_structured_data: false,
            lighthouse_performance: None,
            lighthouse_accessibility: None,
            lighthouse_best_practices: None,
            lighthouse_seo: None,
            links: vec![],
            headings: vec![],
            images: vec![],
            detailed_links: vec![],
        };

        page_repo
            .insert(&page_data)
            .await
            .expect("Failed to insert page");

        // 3. Update Progress
        results_repo
            .update_progress(&result_id, 50.0, 1, 2)
            .await
            .unwrap();

        // Generate summary (required for get_result_by_job_id)
        let summary_repo = SummaryRepository::new(pool.clone());
        summary_repo
            .generate_summary(&result_id, &[], &[page_data])
            .await
            .unwrap();

        let complete = results_repo.get_result_by_job_id(job_id).await.unwrap();
        assert_eq!(complete.analysis.id, result_id);
        assert_eq!(complete.analysis.progress, 50.0);
        assert_eq!(complete.pages.len(), 1);
        assert_eq!(complete.pages[0].url, "https://result.test/page1");

        // 4. Finalize
        results_repo
            .finalize(&result_id, AnalysisStatus::Completed)
            .await
            .unwrap();

        let finalized = results_repo.get_result_by_job_id(job_id).await.unwrap();
        assert_eq!(finalized.analysis.status, JobStatus::Completed); // check mapper logic if it matches enum
    }

    #[tokio::test]
    async fn test_detailed_page_persistence() {
        let pool = fixtures::setup_test_db().await;
        let results_repo = ResultsRepository::new(pool.clone());
        let page_repo = PageRepository::new(pool.clone());
        let job_repo = JobRepository::new(pool.clone());
        let settings = fixtures::default_settings();

        // Setup job & result
        let job_id = job_repo
            .create_with_settings("https://detail.test", &settings)
            .await
            .unwrap();
        let result_id = results_repo
            .create("https://detail.test", true, true, true)
            .await
            .unwrap();
        job_repo.link_to_result(job_id, &result_id).await.unwrap();

        // Create page with details
        let mut page_data = PageAnalysisData::default_test_instance();
        page_data.analysis_id = result_id.clone();
        page_data.headings = vec![HeadingElement {
            tag: "h1".into(),
            text: "Header".into(),
        }];
        page_data.images = vec![ImageElement {
            src: "img.jpg".into(),
            alt: Some("Alt".into()),
        }];
        page_data.detailed_links = vec![LinkElement {
            href: "/".into(),
            text: "Home".into(),
            is_internal: true,
            status_code: None,
        }];

        page_repo.insert(&page_data).await.unwrap();

        // Retrieve and Verify
        // Need to ensure summary exists for get_result_by_job_id
        let summary_repo = SummaryRepository::new(pool.clone());
        summary_repo
            .generate_summary(&result_id, &[], &[page_data.clone()])
            .await
            .unwrap();

        let complete = results_repo.get_result_by_job_id(job_id).await.unwrap();
        let retrieved_page = &complete.pages[0];

        assert_eq!(retrieved_page.headings.len(), 1);
        assert_eq!(retrieved_page.headings[0].text, "Header");
        assert_eq!(retrieved_page.images.len(), 1);
        assert_eq!(retrieved_page.images[0].src, "img.jpg");
        assert_eq!(retrieved_page.detailed_links.len(), 1);
        assert_eq!(retrieved_page.detailed_links[0].text, "Home");
    }

    #[tokio::test]
    async fn test_job_queries() {
        let pool = fixtures::setup_test_db().await;
        let job_repo = JobRepository::new(pool.clone());
        let results_repo = ResultsRepository::new(pool.clone());
        let summary_repo = SummaryRepository::new(pool.clone());

        let settings = fixtures::default_settings();
        let job_id_1 = job_repo
            .create_with_settings("https://job1.test", &settings)
            .await
            .unwrap();
        let _job_id_2 = job_repo
            .create_with_settings("https://job2.test", &settings)
            .await
            .unwrap();

        // job 1 has result
        let result_id = results_repo
            .create("https://job1.test", false, false, false)
            .await
            .unwrap();
        results_repo
            .update_progress(&result_id, 33.0, 1, 3)
            .await
            .unwrap();
        // ensure summary exists for join
        summary_repo
            .generate_summary(&result_id, &[], &[])
            .await
            .unwrap();
        job_repo.link_to_result(job_id_1, &result_id).await.unwrap();

        // Test get_progress
        let progress = job_repo.get_progress(job_id_1).await.unwrap();
        assert_eq!(progress.job_id, job_id_1);
        assert_eq!(progress.progress, Some(33.0));
        assert_eq!(progress.analyzed_pages, Some(1));

        // Test get_all
        let all_jobs = job_repo.get_all().await.unwrap();
        assert!(all_jobs.len() >= 2);

        let job_1_progress = all_jobs.iter().find(|j| j.job_id == job_id_1).unwrap();
        assert_eq!(job_1_progress.progress, Some(33.0));
    }

    #[tokio::test]
    async fn test_link_status_backfilling() {
        let pool = fixtures::setup_test_db().await;
        let results_repo = ResultsRepository::new(pool.clone());
        let page_repo = PageRepository::new(pool.clone());
        let job_repo = JobRepository::new(pool.clone());
        let settings = fixtures::default_settings();

        let job_id = job_repo
            .create_with_settings("https://backfill.test", &settings)
            .await
            .unwrap();
        let result_id = results_repo
            .create("https://backfill.test", true, true, true)
            .await
            .unwrap();
        job_repo.link_to_result(job_id, &result_id).await.unwrap();

        // 1. Insert "Target" page (the one being linked TO)
        let mut target_page = PageAnalysisData::default_test_instance();
        target_page.analysis_id = result_id.clone();
        target_page.url = "https://backfill.test/target".into();
        target_page.status_code = Some(404); // Let's say it was found but missing
        page_repo.insert(&target_page).await.unwrap();

        // 2. Insert "Source" page (links TO target)
        let mut source_page = PageAnalysisData::default_test_instance();
        source_page.analysis_id = result_id.clone();
        source_page.url = "https://backfill.test/source".into();
        // The link initially has NO status code
        source_page.detailed_links = vec![LinkElement {
            href: "/target".into(),
            text: "Go to Target".into(),
            is_internal: true,
            status_code: None,
        }];
        page_repo.insert(&source_page).await.unwrap();

        // Generate summary (required)
        let summary_repo = SummaryRepository::new(pool.clone());
        summary_repo
            .generate_summary(&result_id, &[], &[target_page, source_page])
            .await
            .unwrap();

        // 3. Fetch Result and Verify Backfilling
        let complete = results_repo.get_result_by_job_id(job_id).await.unwrap();

        let retrieved_source = complete
            .pages
            .iter()
            .find(|p| p.url == "https://backfill.test/source")
            .unwrap();
        let link = &retrieved_source.detailed_links[0];

        assert_eq!(link.href, "/target");
        assert_eq!(
            link.status_code,
            Some(404),
            "Status code should be backfilled from target page"
        );
    }
}
