//! The views module contains the components for all Layouts and Routes for our app. Each layout and route in our [`Route`]
//! enum will render one of these components.
//!
//!
//! The [`Home`] and [`Blog`] components will be rendered when the current route is [`Route::Home`] or [`Route::Blog`] respectively.
//!
//!
//! The [`Navbar`] component will be rendered on all pages of our app since every page is under the layout. The layout defines
//! a common wrapper around all child routes.

mod home;
pub use home::Home;

mod blog;
pub use blog::Blog;

mod navbar;
pub use navbar::Navbar;

mod seo_analysis;
pub use seo_analysis::HeadingExtractor;

mod brokenlinks;
pub use brokenlinks::BrokenLinks;

mod title_description;
pub use title_description::TitleDiscription;

mod keywords;
pub use keywords::Keywords;
