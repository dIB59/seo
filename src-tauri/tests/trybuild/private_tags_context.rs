// Tags context types must be accessed via `contexts::tags::Tag`,
// not leaked into the flat `contexts::` barrel.
use app::contexts::Tag;

fn main() {
    let _ = std::mem::size_of::<Tag>();
}
