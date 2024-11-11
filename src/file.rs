use std::path;
use std::{fs, path::PathBuf};
use gtk::{ApplicationWindow, MessageDialog, Notebook, TextBuffer};
use gtk::prelude::*;

use crate::page::create_page;

pub fn save_file(buffer: &TextBuffer, path: &PathBuf) {
    let contents = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
    fs::write(path, contents).unwrap();
}

pub fn open_file(buffer: &TextBuffer, window: &ApplicationWindow) {}

pub fn new_file(buffer: &TextBuffer, window: &ApplicationWindow) {}

pub fn save_as_file(buffer: &TextBuffer, window: &ApplicationWindow) {}

pub fn read_file_contents(path: &PathBuf) -> Option<(String, String)> {
	match fs::read_to_string(&path) {
        Ok(contents) => {
            let filename = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unknown file")
                .to_string();

            Some((contents, filename))
        }
        Err(e) => {
            let error_dialog = MessageDialog::builder()
                .message_type(gtk::MessageType::Error)
                .buttons(gtk::ButtonsType::Ok)
                .text("Error reading file")
                .secondary_text(&format!("Error: {}", e))
                .modal(true)
                .build();

            error_dialog.connect_response(|dialog, _| {
                dialog.close();
            });

            error_dialog.show();

            None
        }
    }
}

pub fn get_language_from_extension(filename: &str) -> Option<&str> {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(std::ffi::OsStr::to_str)?;

    match ext {
        "rs" => Some("rust"),
        "c" => Some("c"),
        "cpp" | "cc" | "cxx" | "hpp" => Some("cplusplus"),
        "py" => Some("python"),
        "js" => Some("javascript"),
        "ts" => Some("typescript"),
        "java" => Some("java"),
        "go" => Some("go"),
        "cs" => Some("csharp"),

        // add other extensions and languages as needed

        _ => None,
    }
}

pub fn download_icon(url: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    let bytes = response.bytes()?;
    fs::write(dest, &bytes)?;
    Ok(())
}

pub fn open_file_dialog(notebook: &Notebook) {
        let file_chooser = gtk::FileChooserDialog::builder()
            .title("Open file")
            .action(gtk::FileChooserAction::Open)
            .build();

        file_chooser.add_button("Cancel", gtk::ResponseType::Cancel);
        file_chooser.add_button("Open", gtk::ResponseType::Accept);

        let notebook_clone = notebook.clone();

        file_chooser.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file_path) = dialog.file().and_then(|file| file.path()) {
                    create_page(&notebook_clone, Some(file_path));
                }
            }

            dialog.close();
        });

        file_chooser.show();
}