// Template types ARE accessible via the proper facade path.
use app::contexts::report::{ReportTemplate, TemplateSection};
use app::contexts::report::template::{
    Condition, PatternFilter, RenderContext, RenderError, RenderedFragment,
};

fn main() {
    let _ = (
        std::mem::size_of::<ReportTemplate>(),
        std::mem::size_of::<TemplateSection>(),
        std::mem::size_of::<Condition>(),
        std::mem::size_of::<PatternFilter>(),
        std::mem::size_of::<RenderError>(),
        std::mem::size_of::<RenderedFragment>(),
    );
}
