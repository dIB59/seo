// Template engine internals (engine module, condition module) must not
// be accessible directly — only the re-exports from `report::template::`.
use app::contexts::report::template::engine::context_variables;

fn main() {
    let _ = context_variables;
}
