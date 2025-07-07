use crate::Route;
use dioxus::prelude::*;


/// The Navbar component that will be rendered on all pages of our app since every page is under the layout.
///
///
/// This layout component wraps the UI of [Route::Home] and [Route::Blog] in a common navbar. The contents of the Home and Blog
/// routes will be rendered under the outlet inside this component
#[component]
pub fn Navbar() -> Element {
    rsx! {
        nav {
            class: "navbar",
            Link {
                class: "nav-link",
                active_class: "active-nav-link",
                to: Route::Home {},
                "Home"
            }
            Link {
                class: "nav-link",
                active_class: "active-nav-link",
                to: Route::Blog { id: 1 },
                "Blog"
            }
            Link {
                class: "nav-link",
                active_class: "active-nav-link",
                to: Route::HeadingExtractor {},
                "Heading Extractor"
            }
            Link {
                class: "nav-link",
                active_class: "active-nav-link",
                to: Route::BrokenLinks {},
                "Broken Link"
            }
            Link {
                class: "nav-link",
                active_class: "active-nav-link",
                to: Route::TitleDiscription {},
                "Title Discription"
            }
        }

        // The `Outlet` component is used to render the next component inside the layout. In this case, it will render either
        // the [`Home`] or [`Blog`] component depending on the current route.
        Outlet::<Route> {}
    }
}
