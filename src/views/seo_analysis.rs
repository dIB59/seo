use dioxus::prelude::*;
use crate::io::http_client;
use crate::extractor::heading_extractor;

#[component]
pub fn HeadingExtractor() -> Element {
    let mut url = use_signal(|| String::new());
    let headings = use_signal(|| Vec::<(String, String)>::new());
    println!("App component rendered");

    rsx! {
        div {
            id: "seo-analysis",
            class: "p-6 max-w-xl mx-auto",

            h1 { class: "text-2xl font-bold mb-4", "Heading Extractor" }

            input {
                class: "border p-2 w-full mb-4 rounded text-gray-700",
                r#type: "text",
                placeholder: "Enter URL...",
                value: "{url()}",
                oninput: move |e| url.set(e.value().clone()),
            }

            button {
                class: "bg-blue-500 text-black px-4 py-2 rounded hover:bg-blue-600",
                onclick: move |_| {
                    println!("Fetching headings from URL: {}", url());
                    let url_val = url();
                    let mut headings = headings;
                    spawn(async move {
                        if let Some(html) = http_client::fetch_html_from_url(&url_val).await {
                            let result = heading_extractor::extract_headings_from_html(&html);
                            headings.set(result);
                        }
                    });
                },
                "Extract Headings"
            }

            div {
                class: "mt-6 space-y-2",
                for (tag, text) in headings().iter() {
                    div {
                        class: "p-2 bg-gray-100 rounded",
                        strong { "{tag}: " }
                        span { "{text}" }
                    }
                }
            }
        }
    }
}
