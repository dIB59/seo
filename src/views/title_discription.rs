use crate::{extractor::{general_extractor, url_extractor}, io::http_client};
use dioxus::{html::u, prelude::*};

#[component]
pub fn TitleDiscription() -> Element {
    let mut url = use_signal(|| String::new());
    let mut title = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());

    rsx!{
        div {
            class: "min-h-screen p-8 flex flex-col items-center justify-start",

            div {
                class: "w-full max-w-2xl space-y-6 bg-card",

                h1 {
                    class: "text-3xl font-semibold tracking-tight",
                    "Meta Title and Description"
                }

                input {
                    class: "w-full px-4 py-2 bg-input text-foreground border border-border rounded-lg focus:outline-none focus:ring-2 focus:ring-ring transition",
                    r#type: "url",
                    placeholder: "Enter a URL (e.g. https://example.com)",
                    value: "{url()}",
                    oninput: move |e| url.set(e.value().clone()),
                }

                button {
                    class: "w-full bg-primary text-primary-foreground px-4 py-2 rounded-lg hover:bg-primary/90 transition",
                    onclick: move |_| {
                        let url_val = url();
                        // let mut headings = headings;
                        spawn(async move {
                            if let Some(html) = http_client::fetch_html_from_url(&url_val).await {
                                let m_title = general_extractor::extract_meta_titles_from_html(&html);
                                let m_discription = general_extractor::extract_meta_description_from_html(&html);
                                title.set(m_title);
                                description.set(m_discription);
                            }
                        });
                    },
                    "Extract Both"
                }

                div {
                    class: "w-full max-w-2xl space-y-6 bg-card",
                    h1 {
                        class: "text-3xl font-semibold tracking-tight",
                        "Title and Description"
                    },
                    li {
                        class: "text-sm font-semibold text-muted-foreground",
                        "title: " "{title()}"
                    }
                    li {
                        class: "text-sm font-semibold text-muted-foreground",
                        "discription: " "{description()}"
                    }

                }
            }
        }
    }
}