use crate::extractor::heading_extractor;
use crate::io::http_client;
use dioxus::prelude::*;

#[component]
pub fn HeadingExtractor() -> Element {
    let mut url = use_signal(|| String::new());
    let mut headings = use_signal(|| Vec::<(String, String)>::new());

    rsx! {
        div {
            class: "min-h-screen p-8 flex flex-col items-center justify-start",

            div {
                class: "w-full max-w-2xl space-y-6 bg-card",

                h1 {
                    class: "text-3xl font-semibold tracking-tight",
                    "Heading Extractor"
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
                                let result = heading_extractor::extract_headings_from_html(&html);
                                headings.set(result);
                            }
                        });
                    },
                    "Extract Headings"
                }

                if !headings().is_empty() {
                    div {
                        class: "bg-card border border-border rounded-lg p-4 space-y-3",

                        h2 {
                            class: "text-xl font-medium",
                            "Extracted Headings"
                        }

                        for (tag, text) in headings().iter() {
                            div {
                                class: "bg-muted px-4 py-2 rounded flex items-center space-x-2",
                                span {
                                    class: "text-sm font-semibold text-muted-foreground",
                                    "{tag}"
                                }
                                span {
                                    class: "text-sm text-foreground",
                                    "{text}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
