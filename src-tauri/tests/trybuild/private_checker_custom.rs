// CustomCheckAdapter is an internal implementation detail of the
// checker module — commands should never construct one directly.
use app::checker::custom::CustomCheckAdapter;

fn main() {
    let _ = std::mem::size_of::<CustomCheckAdapter>();
}
