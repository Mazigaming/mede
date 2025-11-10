mod downloader;
use downloader::Downloader;
use dioxus::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    AddressBar {},
    #[route("/editor")]
    Editor {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[component]
pub fn AddressBar() -> Element {
    let mut url = use_signal(|| "".to_string());
    let mut status = use_signal(|| "Idle".to_string());
    let mut only_sound = use_signal(|| false);
    let mut attach_metadata = use_signal(|| false);

    let downloader = Downloader::new("libs".into(), "output".into());

    rsx! {
        div {
            id: "userInterface",

            div {
                input {
                    id: "addressBar",
                    r#type: "text",
                    placeholder: "link here....",
                    value: "{url}",
                    oninput: move |e| url.set(e.value().clone()),
                },

                button {
                    id: "downloadBtn",
                    onclick: move |_| {
                        let url_value = url();
                        let only_sound_val = only_sound();
                        let attach_metadata_val = attach_metadata();
                        let downloader_clone = downloader.clone();
                        let mut status = status; 
                        
                        if url_value.trim().is_empty() {
                            status.set("❌ Please enter a URL".to_string());
                            return;
                        }

                        spawn(async move {
                            status.set("⏳ Downloading...".to_string());

                            let output_file = if only_sound_val {
                                "audio.mp3".to_string()
                            } else {
                                "video.mp4".to_string()
                            };

                            match downloader_clone.download(url_value, output_file).await {
                                Ok(path) => {
                                    let msg = format!("✅ Downloaded to {}", path.display());
                                    status.set(msg);
                                }
                                Err(e) => {
                                    status.set(format!("❌ Download failed: {}", e));
                                }
                            }
                        });
                    },
                    "Download"
                }
            }

            div {
                id: "togglers",

                div {
                    h1 { "Editor Mode:" }
                    label {
                        class: "switch",
                        input {
                            r#type: "checkbox",
                        },
                        span { class: "slider round" }
                    }
                }
                div {
                    h1 { "Sound Only:" }
                    label {
                        class: "switch",
                        input {
                            r#type: "checkbox",
                            checked: only_sound(),
                            onchange: move |e| {
                                let mut only_sound = only_sound;
                                only_sound.set(e.checked());
                            },
                        },
                        span { class: "slider round" }
                    }
                }

                div {
                    h1 { "Attach Metadata:" }
                    label {
                        class: "switch",
                        input {
                            r#type: "checkbox",
                            checked: attach_metadata(),
                            onchange: move |e| {
                                let mut attach_metadata = attach_metadata;
                                attach_metadata.set(e.checked());
                            },
                        },
                        span { class: "slider round" }
                    }
                }
            }

            div {
                p { "Status: {status}" }
            }
        }
    }
}

#[component]
pub fn Editor() -> Element {
    rsx! {
        div {
        
        }
    }
}
