//! Various widgets

use std::path::PathBuf;

use dioxus::prelude::*;

/// A refresh button with a busy indicator
///
/// You want to use [crate::state::datum] for this.
#[component]
pub fn RefreshButton<F>(cx: Scope, busy: bool, handler: F) -> Element
where
    F: Fn(Event<MouseData>),
{
    let button_cls = if *busy {
        "bg-gray-400 text-white"
    } else {
        "bg-blue-700 text-white hover:bg-blue-800"
    };
    render! {
        div { class: "flex-col items-center justify-center space-y-2 mb-4",
            button {
                class: "py-1 px-2 shadow-lg border-1 {button_cls} rounded-md",
                disabled: *busy,
                onclick: handler,
                "Refresh "
                if *busy {
                    render! { "⏳" }
                } else {
                    render! { "🔄" }
                }
            }
            if *busy {
                render! { Loader {} }
            }
        }
    }
}

/// A button that opens a file explorer dialog.
///
/// Note: You can only select a single folder.
///
/// NOTE(for future): When migrating to Dioxus using Tauri 2.0, switch to using
/// https://github.com/tauri-apps/tauri-plugin-dialog
#[component]
pub fn FolderDialogButton<F>(cx: Scope, handler: F) -> Element
where
    F: Fn(PathBuf),
{
    // FIXME: The id should be unique if this widget is used multiple times on
    // the same page.
    let id = "folder-dialog-input";
    render! {
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
    match evt.files.as_ref() {
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
pub fn Loader(cx: Scope) -> Element {
    render! {
        div { class: "flex justify-center items-center",
            div { class: "animate-spin rounded-full h-16 w-16 border-t-2 border-b-2 border-purple-500" }
        }
    }
}

/// A div that can get scrollbar for long content
///
/// Since our body container is `overflow-hidden`, we need to wrap content that
/// can get long in this component.
#[component]
pub fn Scrollable<'a>(cx: Scope, children: Element<'a>) -> Element {
    render! {
        div { class: "overflow-auto", children }
    }
}
