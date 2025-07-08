use crate::{extractor::general_extractor, io::http_client, views::keywords};
use dioxus::prelude::*;

#[component]
pub fn Keywords() -> Element {
    let mut url = use_signal(|| String::new());
    let mut keywords = use_signal(|| String::new());

    rsx!{
        div {
            class: "min-h-screen p-8 flex flex-col items-center justify-start",

            div {
                class: "w-full max-w-2xl space-y-6 bg-card",

                h1 {
                    class: "text-3xl font-semibold tracking-tight",
                    "Meta Keywords"
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
                        spawn(async move {
                            if let Some(html) = http_client::fetch_html_from_url(&url_val).await {
                                let m_keywords = general_extractor::extract_meta_keywords_from_html(&html);
                                keywords.set(m_keywords);
                            }
                        });
                    },
                    "Extract Keywords"
                }


                div {
                    class: "w-full max-w-2xl space-y-6 bg-card",

                    h2 {
                        class: "text-xl font-medium",
                        "Extracted Keywords"
                    }

                    {keywords()}
                }
                
            }
        }
    }
}