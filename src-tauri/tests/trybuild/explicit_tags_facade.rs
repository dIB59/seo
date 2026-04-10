// Tags types ARE accessible via the proper facade path.
use app::contexts::tags::{Tag, TagRegistry, TagScope, TagSource, TagDataType};

fn main() {
    let _ = (
        std::mem::size_of::<Tag>(),
        std::mem::size_of::<TagScope>(),
        std::mem::size_of::<TagSource>(),
        std::mem::size_of::<TagDataType>(),
    );
    // TagRegistry is not a specta::Type, just verify it exists
    let _ = std::mem::size_of::<TagRegistry>();
}
