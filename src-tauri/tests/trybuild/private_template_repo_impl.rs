// The SQLite template repository implementation must not be accessible
// outside the repository module — callers use the trait via Arc<dyn>.
use app::repository::sqlite::template_repository::ReportTemplateRepository;

fn main() {
    let _ = std::mem::size_of::<ReportTemplateRepository>();
}
