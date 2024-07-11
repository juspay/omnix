//! Various widgets

use std::path::PathBuf;

use dioxus::prelude::*;

/// A refresh button with a busy indicator
///
/// You want to use [crate::state::datum] for this.
pub fn RefreshButton<F: 'static + FnMut(Event<MouseData>)>(busy: bool, mut handler: F) -> Element {
    rsx! {
        button {
            disabled: busy,
            onclick: move |evt| {
                if !busy {
                    handler(evt)
                }
            },
            title: "Refresh current data being viewed",
            LoaderIcon { loading: busy }
        }
    }
}

/// A button that opens a file explorer dialog.
///
/// Note: You can only select a single folder.
///
/// NOTE(for future): When migrating to Dioxus using Tauri 2.0, switch to using
/// https://github.com/tauri-apps/tauri-plugin-dialog
// #[component]
pub fn FolderDialogButton<F: 'static + FnMut(PathBuf)>(mut handler: F) -> Element {
    // FIXME: The id should be unique if this widget is used multiple times on
    // the same page.
    let id = "folder-dialog-input";
    rsx! {
        input {
            r#type: "file",
            multiple: false,
            directory: true,
            accept: "",
            oninput: move |evt: Event<FormData>| {
                if let Some(path) = get_selected_path(evt) {
                    handler(path)
                }
            },
            id: id,
            class: "hidden"
        }
        label {
            class: "py-1 px-1 cursor-pointer hover:scale-125 active:scale-100",
            r#for: id,
            title: "Choose a local folder",
            "📁"
        }
    }
}

/// Get the user selected path from a file dialog event
///
/// If the user has not selected any (eg: cancels the dialog), this returns
/// None. Otherwise, it returns the first entry in the selected list.
fn get_selected_path(evt: Event<FormData>) -> Option<PathBuf> {
    match evt.files().as_ref() {
        None => {
            tracing::error!("unable to get files from event");
            None
        }
        Some(file_engine) => {
            let path = file_engine.files().first().cloned()?;
            Some(PathBuf::from(path))
        }
    }
}

#[component]
pub fn Loader() -> Element {
    rsx! {
        div { class: "flex justify-center items-center",
            div { class: "animate-spin rounded-full h-16 w-16 border-t-2 border-b-2 border-purple-500" }
        }
    }
}

#[component]
pub fn LoaderIcon(loading: bool) -> Element {
    let cls = if loading {
        "animate-spin text-base-800"
    } else {
        "text-primary-700 hover:text-primary-500"
    };
    rsx! {
        div { class: cls,
            svg {
                class: "h-6 w-6 scale-x-[-1]",
                xmlns: "http://www.w3.org/2000/svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                path {
                    d: "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "2"
                }
            }
        }
    }
}

/// A div that can get scrollbar for long content
///
/// Since our body container is `overflow-hidden`, we need to wrap content that
/// can get long in this component.
#[component]
#[allow(dead_code)] // https://github.com/juspay/nix-browser/issues/132
pub fn Scrollable(children: Element) -> Element {
    rsx! {
        div { class: "overflow-auto", { children } }
    }
}
