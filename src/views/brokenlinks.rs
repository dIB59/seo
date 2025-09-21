use crate::{extractor::url_extractor, io::http_client};
use dioxus::prelude::*;

#[component]
pub fn BrokenLinks() -> Element {
    let mut url = use_signal(|| String::new());
    let mut foundlinks = use_signal(|| Vec::<String>::new());
    let mut brokenlinks = use_signal(|| Vec::<(String, String)>::new());
    let mut is_checking = use_signal(|| true);

    rsx! {
        div {
            class: "min-h-screen p-8 flex flex-col items-center justify-start",
            div {
                    class: "w-full max-w-2xl space-y-6 bg-card",

                    h1 {
                        class: "text-3xl font-semibold tracking-tight",
                        "Broken Links"
                    }

                    input {
                        class: "w-full px-4 py-2 bg-input text-foreground border border-border rounded-lg focus:outline-none focus:ring-2 focus:ring-ring transition",
                        r#type: "url",
                        placeholder: "Enter a URL (e.g. https://example.com)",
                        value: "{url()}",
                        oninput: move |e| url.set(e.value().clone()),
                    }

                    div {
                        class: "flex space-x-4",
                    button {
                        class: "w-full bg-primary text-primary-foreground px-4 py-2 rounded-lg hover:bg-primary/90 transition",
                        onclick: move |_| async move {
                            let url_val = url();
                            spawn(async move {
                                if let Some(html) = http_client::fetch_html_from_url(&url_val).await {
                                    let res = url_extractor::extract_urls_from_html(&html);
                                    foundlinks.set(res);
                                }

                                for mut link in foundlinks() {
                                    if !is_checking() {
                                        break;
                                    }
                                    let new_link: String;
                                    if link.starts_with("/"){
                                        new_link = format!("{url_val}{link}");
                                        link = new_link;
                                    }
                                    let res = http_client::identify_broken_links(&link).await;
                                    brokenlinks.push((link, res));
                                }
                            });
                        },
                        "Broken Links"
                    }
                    button {
                        class: "w-full bg-primary text-primary-foreground px-4 py-2 rounded-lg hover:bg-primary/90 transition",
                        onclick: move |_| {
                            is_checking.set(false);
                        },
                        "Stop"
                    }
                }

                div {
                    class: "w-full max-w-2xl space-y-6 bg-card",
                    h1 {
                        class: "text-3xl font-semibold tracking-tight",
                        "Broken Links"
                    },
                    ul {
                        for (link, status) in brokenlinks().iter() {

                            li { "link:","{link}"," status:", "{status}"}
                        }
                    }

                }
            }
        }

    }
}
