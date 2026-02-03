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

use criterion::{criterion_group, criterion_main, Criterion};
use std::{hint::black_box, time::Duration};
use tokio::runtime::Runtime;


use app::{
    repository::sqlite::{
         JobRepository, PageRepository, ResultsRepository,
    },
    repository::sqlite::{
     IssueRepository, LinkRepository
    },
    test_utils::{connect_test_db_no_migrate, connect_test_db_v1},
};

// ============================================================================
// V1 READ BENCHMARKS - Old schema (requires JOINs through analysis_results)
// ============================================================================
// Uses test_v1.db which has the original schema with:
// - analysis_jobs, analysis_results, page_analysis, seo_issues, page_edge


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



// ============================================================================
// V2 READ BENCHMARKS - New schema with direct FK lookups
// ============================================================================
// These benchmarks test the redesigned schema where pages, issues, and links
// have a direct job_id FK, eliminating the need for JOINs through analysis_results.

/// Benchmark V2 complete result retrieval (should be faster than V1)
fn bench_get_complete_result_v2(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(connect_test_db_no_migrate());
    let repo = ResultsRepository::new(pool);

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
    let repo = JobRepository::new(pool);

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
    let repo = JobRepository::new(pool);

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
    let repo = JobRepository::new(pool);

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
    let repo = IssueRepository::new(pool);

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
    let repo = PageRepository::new(pool);

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
    let repo = LinkRepository::new(pool);

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

// ============================================================================
// CRITERION CONFIGURATION
// ============================================================================

// V1 read benchmarks - old schema (requires JOINs through analysis_results)
// Uses test_v1.db with original schema

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

criterion_main!(read_benchmarks_v2);
