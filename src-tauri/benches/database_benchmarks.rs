// benches/database_benchmarks.rs
//
// Comprehensive database benchmarks for measuring read/write performance
// Run with: cargo bench --bench database_benchmarks
//
// These benchmarks establish baseline metrics before optimization and
// measure improvements after implementing SQLite pragmas, connection pooling,
// and query optimizations.
//
// V2 BENCHMARKS: Compare old schema (with JOINs) vs new schema (direct FK lookups)
// The V2 repositories use the redesigned schema with direct job_id on pages/issues/links.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};
use tokio::runtime::Runtime;
use uuid::Uuid;

use app::{
    repository::sqlite::{
        IssuesRepository, JobRepository, PageRepository, ResultsRepository, SummaryRepository,
    },
    repository::sqlite_v2::{
        JobRepositoryV2, PageRepositoryV2, IssueRepositoryV2, LinkRepositoryV2, ResultsRepositoryV2,
    },
    test_utils::{generators, set_up_benchmark_db, set_up_test_db_with_prod_data, connect_test_db_no_migrate, connect_test_db_v1},
};

// ============================================================================
// V1 READ BENCHMARKS - Old schema (requires JOINs through analysis_results)
// ============================================================================
// Uses test_v1.db which has the original schema with:
// - analysis_jobs, analysis_results, page_analysis, seo_issues, page_edge

/// Benchmark V1 complete result retrieval (uses JOINs)
fn bench_get_result_by_job_id_v1(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(connect_test_db_v1());
    let repo = ResultsRepository::new(pool);

    c.bench_function("read_v1/get_result_by_job_id", |b| {
        b.to_async(&rt).iter(|| async {
            let result = repo
                .get_result_by_job_id(black_box(12))
                .await
                .expect("Failed to get result");
            black_box(result)
        });
    });
}

/// Benchmark V1 pending jobs
fn bench_get_pending_jobs_v1(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(connect_test_db_v1());
    let repo = JobRepository::new(pool);

    c.bench_function("read_v1/get_pending_jobs", |b| {
        b.to_async(&rt).iter(|| async {
            let result = repo.get_pending_jobs().await.expect("Failed to get jobs");
            black_box(result)
        });
    });
}

/// Benchmark V1 all jobs retrieval
fn bench_get_all_jobs_v1(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(connect_test_db_v1());
    let repo = JobRepository::new(pool);

    c.bench_function("read_v1/get_all_jobs", |b| {
        b.to_async(&rt).iter(|| async {
            let result = repo.get_all().await.expect("Failed to get all jobs");
            black_box(result)
        });
    });
}

/// Benchmark V1 progress polling
fn bench_get_progress_v1(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(connect_test_db_v1());
    let repo = JobRepository::new(pool);

    c.bench_function("read_v1/get_progress", |b| {
        b.to_async(&rt).iter(|| async {
            let result = repo.get_progress(black_box(13)).await.expect("Failed");
            black_box(result)
        });
    });
}

// ============================================================================
// V2 READ BENCHMARKS - New schema with direct FK lookups
// ============================================================================
// These benchmarks test the redesigned schema where pages, issues, and links
// have a direct job_id FK, eliminating the need for JOINs through analysis_results.

/// Benchmark V2 complete result retrieval (should be faster than V1)
fn bench_get_complete_result_v2(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(connect_test_db_no_migrate());
    let repo = ResultsRepositoryV2::new(pool);

    // Use existing job ID from migrated data
    c.bench_function("read_v2/get_complete_result", |b| {
        let job_id = "12";  // Using string ID
        b.to_async(&rt).iter(|| async {
            let result = repo
                .get_complete_result(black_box(job_id))
                .await
                .expect("Failed to get result");
            black_box(result)
        });
    });
}

/// Benchmark V2 pending jobs (direct status lookup)
fn bench_get_pending_jobs_v2(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(connect_test_db_no_migrate());
    let repo = JobRepositoryV2::new(pool);

    c.bench_function("read_v2/get_pending_jobs", |b| {
        b.to_async(&rt).iter(|| async {
            let result = repo.get_pending().await.expect("Failed to get jobs");
            black_box(result)
        });
    });
}

/// Benchmark V2 all jobs retrieval
fn bench_get_all_jobs_v2(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(connect_test_db_no_migrate());
    let repo = JobRepositoryV2::new(pool);

    c.bench_function("read_v2/get_all_jobs", |b| {
        b.to_async(&rt).iter(|| async {
            let result = repo.get_all().await.expect("Failed to get all jobs");
            black_box(result)
        });
    });
}

/// Benchmark V2 progress polling (direct column on jobs table)
fn bench_get_progress_v2(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(connect_test_db_no_migrate());
    let repo = JobRepositoryV2::new(pool);

    c.bench_function("read_v2/get_progress", |b| {
        let job_id = "13";  // Using string ID  
        b.to_async(&rt).iter(|| async {
            let result = repo.get_by_id(black_box(job_id)).await.expect("Failed");
            black_box(result.progress)
        });
    });
}

/// Benchmark V2 issues by job (direct FK lookup - key optimization)
fn bench_get_issues_by_job_v2(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(connect_test_db_no_migrate());
    let repo = IssueRepositoryV2::new(pool);

    c.bench_function("read_v2/get_issues_by_job", |b| {
        let job_id = "12";  // Using string ID
        b.to_async(&rt).iter(|| async {
            let result = repo.get_by_job_id(black_box(job_id)).await.expect("Failed");
            black_box(result)
        });
    });
}

/// Benchmark V2 pages by job (direct FK lookup - key optimization)
fn bench_get_pages_by_job_v2(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(connect_test_db_no_migrate());
    let repo = PageRepositoryV2::new(pool);

    c.bench_function("read_v2/get_pages_by_job", |b| {
        let job_id = "12";  // Using string ID
        b.to_async(&rt).iter(|| async {
            let result = repo.get_by_job_id(black_box(job_id)).await.expect("Failed");
            black_box(result)
        });
    });
}

/// Benchmark V2 links by job (direct FK lookup - key optimization)
fn bench_get_links_by_job_v2(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(connect_test_db_no_migrate());
    let repo = LinkRepositoryV2::new(pool);

    c.bench_function("read_v2/get_links_by_job", |b| {
        let job_id = "12";  // Using string ID
        b.to_async(&rt).iter(|| async {
            let result = repo.get_by_job_id(black_box(job_id)).await.expect("Failed");
            black_box(result)
        });
    });
}

// ============================================================================
// WRITE BENCHMARKS - Test insertion performance with synthetic data
// ============================================================================

/// Benchmark single page insertion
fn bench_page_insert_single(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("write/page_insert_single", |b| {
        // Setup outside the benchmark loop
        let pool = rt.block_on(set_up_benchmark_db());
        let analysis_id = Uuid::new_v4().to_string();

        // Create required analysis_results record for FK
        rt.block_on(async {
            sqlx::query(
                "INSERT INTO analysis_results (id, url, status, progress, analyzed_pages, total_pages, sitemap_found, robots_txt_found, ssl_certificate) 
                 VALUES (?, 'https://test.com', 'analyzing', 0, 0, 0, 0, 0, 1)"
            )
            .bind(&analysis_id)
            .execute(&pool)
            .await
            .unwrap();
        });

        let repo = PageRepository::new(pool);
        let mut counter = 0;

        b.to_async(&rt).iter(|| {
            counter += 1;
            let page = generators::generate_mock_pages(1, &analysis_id)
                .into_iter()
                .next()
                .map(|mut p| {
                    p.url = format!("https://example.com/page-{}", counter);
                    p
                })
                .unwrap();
            let repo = repo.clone();
            async move {
                let result = repo.insert(&page).await.expect("Failed to insert page");
                black_box(result)
            }
        });
    });
}

/// Benchmark batch page insertion with varying sizes
fn bench_page_insert_batch(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("write/page_insert_batch");

    for size in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            // Setup outside the benchmark loop
            let pool = rt.block_on(set_up_benchmark_db());
            let analysis_id = Uuid::new_v4().to_string();

            rt.block_on(async {
                sqlx::query(
                    "INSERT INTO analysis_results (id, url, status, progress, analyzed_pages, total_pages, sitemap_found, robots_txt_found, ssl_certificate) 
                     VALUES (?, 'https://test.com', 'analyzing', 0, 0, 0, 0, 0, 1)"
                )
                .bind(&analysis_id)
                .execute(&pool)
                .await
                .unwrap();
            });

            let repo = PageRepository::new(pool.clone());
            let mut batch_counter = 0;

            b.to_async(&rt).iter(|| {
                batch_counter += 1;
                let pages: Vec<_> = generators::generate_mock_pages(size, &analysis_id)
                    .into_iter()
                    .enumerate()
                    .map(|(i, mut p)| {
                        p.url = format!("https://example.com/batch-{}/page-{}", batch_counter, i);
                        p
                    })
                    .collect();
                let repo = repo.clone();
                async move {
                    repo.insert_batch(&pages).await.expect("Failed to insert batch");
                }
            });
        });
    }
    group.finish();
}

/// Benchmark batch issue insertion
fn bench_issues_insert_batch(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("write/issues_insert_batch");

    for size in [50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let pool = rt.block_on(set_up_benchmark_db());
            let analysis_id = Uuid::new_v4().to_string();
            let page_id = Uuid::new_v4().to_string();

            rt.block_on(async {
                sqlx::query(
                    "INSERT INTO analysis_results (id, url, status, progress, analyzed_pages, total_pages, sitemap_found, robots_txt_found, ssl_certificate) 
                     VALUES (?, 'https://test.com', 'analyzing', 0, 0, 0, 0, 0, 1)"
                )
                .bind(&analysis_id)
                .execute(&pool)
                .await
                .unwrap();

                sqlx::query(
                    "INSERT INTO page_analysis (id, analysis_id, url, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, internal_links, external_links, load_time, content_size, mobile_friendly, has_structured_data, created_at) 
                     VALUES (?, ?, 'https://test.com/page', 1, 2, 3, 500, 5, 1, 10, 2, 1.5, 10000, 1, 0, datetime('now'))"
                )
                .bind(&page_id)
                .bind(&analysis_id)
                .execute(&pool)
                .await
                .unwrap();
            });

            let repo = IssuesRepository::new(pool);
            let issues = generators::generate_mock_issues(size, &page_id, "https://test.com/page");

            b.to_async(&rt).iter(|| {
                let repo_clone = repo.clone();
                let issues_clone = issues.clone();
                async move {
                    repo_clone.insert_batch(&issues_clone).await.expect("Failed to insert issues");
                }
            });
        });
    }
    group.finish();
}

/// Benchmark edge insertion (link graph)
fn bench_edges_insert_batch(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("write/edges_insert_batch");

    for size in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let pool = rt.block_on(set_up_benchmark_db());
            let analysis_id = Uuid::new_v4().to_string();

            // Create analysis and pages for FK constraints
            let page_ids: Vec<String> = rt.block_on(async {
                sqlx::query(
                    "INSERT INTO analysis_results (id, url, status, progress, analyzed_pages, total_pages, sitemap_found, robots_txt_found, ssl_certificate) 
                     VALUES (?, 'https://test.com', 'analyzing', 0, 0, 0, 0, 0, 1)"
                )
                .bind(&analysis_id)
                .execute(&pool)
                .await
                .unwrap();

                // Create 10 pages to distribute edges across
                let mut ids = Vec::new();
                for i in 0..10 {
                    let page_id = Uuid::new_v4().to_string();
                    sqlx::query(
                        "INSERT INTO page_analysis (id, analysis_id, url, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, internal_links, external_links, load_time, content_size, mobile_friendly, has_structured_data, created_at) 
                         VALUES (?, ?, ?, 1, 2, 3, 500, 5, 1, 10, 2, 1.5, 10000, 1, 0, datetime('now'))"
                    )
                    .bind(&page_id)
                    .bind(&analysis_id)
                    .bind(format!("https://test.com/page-{}", i))
                    .execute(&pool)
                    .await
                    .unwrap();
                    ids.push(page_id);
                }
                ids
            });

            let repo = PageRepository::new(pool);
            let edges = generators::generate_mock_edges(size, &page_ids);

            b.to_async(&rt).iter(|| {
                let repo_clone = repo.clone();
                let edges_clone = edges.clone();
                async move {
                    repo_clone.insert_edges_batch(&edges_clone).await.expect("Failed to insert edges");
                }
            });
        });
    }
    group.finish();
}

/// Benchmark summary generation
fn bench_generate_summary(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("write/generate_summary", |b| {
        let pool = rt.block_on(set_up_benchmark_db());
        let analysis_id = Uuid::new_v4().to_string();
        let page_id = Uuid::new_v4().to_string();

        rt.block_on(async {
            sqlx::query(
                "INSERT INTO analysis_results (id, url, status, progress, analyzed_pages, total_pages, sitemap_found, robots_txt_found, ssl_certificate) 
                 VALUES (?, 'https://test.com', 'analyzing', 0, 0, 0, 0, 0, 1)"
            )
            .bind(&analysis_id)
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query(
                "INSERT INTO page_analysis (id, analysis_id, url, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, internal_links, external_links, load_time, content_size, mobile_friendly, has_structured_data, created_at) 
                 VALUES (?, ?, 'https://test.com/page', 1, 2, 3, 500, 5, 1, 10, 2, 1.5, 10000, 1, 0, datetime('now'))"
            )
            .bind(&page_id)
            .bind(&analysis_id)
            .execute(&pool)
            .await
            .unwrap();
        });

        let repo = SummaryRepository::new(pool);
        let issues = generators::generate_mock_issues(100, &page_id, "https://test.com/page");
        let pages = generators::generate_mock_pages(10, &analysis_id);

        b.to_async(&rt).iter(|| {
            let repo_clone = repo.clone();
            let analysis_id_clone = analysis_id.clone();
            let issues_clone = issues.clone();
            let pages_clone = pages.clone();
            async move {
                repo_clone.generate_summary(&analysis_id_clone, &issues_clone, &pages_clone)
                    .await
                    .expect("Failed to generate summary");
            }
        });
    });
}

// ============================================================================
// CRITERION CONFIGURATION
// ============================================================================

// V1 read benchmarks - old schema (requires JOINs through analysis_results)
// Uses test_v1.db with original schema
criterion_group! {
    name = read_benchmarks_v1;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(2));
    targets = 
        bench_get_result_by_job_id_v1,
        bench_get_pending_jobs_v1,
        bench_get_all_jobs_v1,
        bench_get_progress_v1
}

// V2 read benchmarks - new schema with direct FK lookups
// Uses test.db with redesigned schema
criterion_group! {
    name = read_benchmarks_v2;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(2));
    targets = 
        bench_get_complete_result_v2,
        bench_get_pending_jobs_v2,
        bench_get_all_jobs_v2,
        bench_get_progress_v2,
        bench_get_issues_by_job_v2,
        bench_get_pages_by_job_v2,
        bench_get_links_by_job_v2
}

// Note: Write benchmarks require specific schema setup
// criterion_group! {
//     name = write_benchmarks;
//     config = Criterion::default()
//         .sample_size(20)
//         .measurement_time(Duration::from_secs(15))
//         .warm_up_time(Duration::from_secs(2));
//     targets = 
//         bench_page_insert_single,
//         bench_page_insert_batch,
//         bench_issues_insert_batch,
//         bench_edges_insert_batch,
//         bench_generate_summary
// }

criterion_main!(read_benchmarks_v1, read_benchmarks_v2);
