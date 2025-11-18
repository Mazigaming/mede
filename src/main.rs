mod downloader;
use downloader::Downloader;
use dioxus::prelude::*;
use std::path::PathBuf;

const LIBS_DIR: &str = "libs";
const OUTPUT_DIR: &str = "output";
const PLACEHOLDER_TEXT: &str = "link here....";
const EMPTY_URL_ERROR: &str = "❌ Please enter a URL";
const DOWNLOADING_STATUS: &str = "⏳ Downloading...";
const SUCCESS_PREFIX: &str = "✅ Downloaded to ";
const ERROR_PREFIX: &str = "❌ Download failed: ";
const AUDIO_OUTPUT: &str = "audio.mp3";
const VIDEO_OUTPUT: &str = "video.mp4";

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    AddressBar {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
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

    let downloader = Downloader::new(LIBS_DIR.into(), OUTPUT_DIR.into());

    rsx! {
        div {
            id: "userInterface",

            div {
                input {
                    id: "addressBar",
                    r#type: "text",
                    placeholder: PLACEHOLDER_TEXT,
                    value: "{url}",
                    oninput: move |e| url.set(e.value().clone()),
                },

                button {
                    id: "downloadBtn",
                    onclick: move |_| {
                        let url_value = url();
                        let only_sound_val = only_sound();
                        let downloader_clone = downloader.clone();
                        let mut status = status; 
                        
                        if url_value.trim().is_empty() {
                            status.set(EMPTY_URL_ERROR.to_string());
                            return;
                        }

                        if !is_valid_url(&url_value) {
                            status.set("❌ Please enter a valid URL (http:// or https://)".to_string());
                            return;
                        }

                        spawn(async move {
                            status.set(DOWNLOADING_STATUS.to_string());

                            let output_file = if only_sound_val {
                                AUDIO_OUTPUT.to_string()
                            } else {
                                VIDEO_OUTPUT.to_string()
                            };

                            match downloader_clone.download(url_value, output_file).await {
                                Ok(path) => {
                                    let msg = format!("{}{}", SUCCESS_PREFIX, path.display());
                                    status.set(msg);
                                }
                                Err(e) => {
                                    status.set(format!("{}{}", ERROR_PREFIX, e));
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
            }

            div {
                p { "Status: {status}" }
            }
        }
    }
}
