use crate::{extractor::url_extractor, io::http_client};
use dioxus::{html::br, prelude::*};

#[component]
pub fn BrokenLinks() -> Element {
    let mut url = use_signal(|| String::new());
    let mut foundlinks = use_signal(|| Vec::<String>::new());
    let mut brokenlinks = use_signal(|| Vec::<(String, String)>::new());
    
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

                    button {
                        class: "w-full bg-primary text-primary-foreground px-4 py-2 rounded-lg hover:bg-primary/90 transition",
                        onclick: move |_| async move {
                            let url_val = url();
                            // spawn(async move {
                            //     let res = http_client::identify_broken_links(&url_val).await;
                            //     println!("Response: {}", res);
                            //     brokenLink.set(res);
                            // });
                            spawn(async move {
                                if let Some(html) = http_client::fetch_html_from_url(&url_val).await {
                                    let res = url_extractor::extract_url_from_html(&html);
                                    foundlinks.set(res);
                                }
                            
                                for link in foundlinks() {
                                    let res = http_client::identify_broken_links(&link).await;
                                    brokenlinks.push((link, res));
                                }
                            });
                        },
                        "Broken Links"
                }

                div {
                    class: "w-full max-w-2xl space-y-6 bg-card",
                    h1 {
                        class: "text-3xl font-semibold tracking-tight",
                        "Broken Links"
                    },
                    
                    for (link, status) in brokenlinks().iter() {
                        br {}
                        
                        p { "link:","{link}"," status:", "{status}"}
                    }
                }
            }
        }
        
    }
}