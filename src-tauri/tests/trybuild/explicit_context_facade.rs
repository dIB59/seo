use app::contexts::analysis::{JobSettings, JobStatus};

fn main() {
    let settings = JobSettings::default();
    let status = JobStatus::Pending;

    let _ = (settings.max_pages, status);
}