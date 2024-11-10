use gtk::{Image, Label, Notebook, Overlay, ScrolledWindow, TextBuffer, TextView};
use gtk::prelude::*;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;
use std::convert::TryInto;

static FILE_PATH_MAP: Lazy<Mutex<HashMap<usize, PathBuf>>> = Lazy::new(|| Mutex::new(HashMap::new()));

use crate::file::{download_icon, get_language_from_extension, read_file_contents};

pub fn create_page(notebook: &Notebook, path: Option<PathBuf>) {
    let scrolled_window = ScrolledWindow::builder().hexpand(true).vexpand(true).build();
    let text_view = TextView::builder().editable(true).wrap_mode(gtk::WrapMode::Char).build();
    let buffer = text_view.buffer();

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let file_path_label = Label::new(Some(""));

    let (tab_title, icon_path, file_path) = if let Some(path) = path {
        if path.is_file() {
            if let Some((contents, filename)) = read_file_contents(&path) {
                buffer.set_text(&contents);
                file_path_label.set_text(&path.to_str().unwrap_or("Invalid UTF-8 path"));

                let language = get_language_from_extension(&filename).unwrap_or("default");
                let icon_url = format!("https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/{}/{}-original.svg", language, language);
                let icon_path = format!("/tmp/{}_icon.png", language);

                if download_icon(&icon_url, &icon_path).is_err() {
                    (filename, "/usr/share/pixmaps/aapelix-qr-logo.png".to_string(), path.clone())
                } else {
                    (filename, icon_path, path.clone())
                }
            } else {
                ("Untitled".to_string(), "/usr/share/pixmaps/aapelix-qr-logo.png".to_string(), path.clone())
            }
        } else {
            ("Untitled".to_string(), "/usr/share/pixmaps/aapelix-qr-logo.png".to_string(), path.clone())
        }
    } else {
        buffer.set_text("");
        ("Untitled".to_string(), "/usr/share/pixmaps/aapelix-qr-logo.png".to_string(), PathBuf::new())
    };

    scrolled_window.set_child(Some(&text_view));

    let label_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .margin_start(6)
        .margin_end(6)
        .margin_top(6)
        .margin_bottom(6)
        .build();

    let icon_image = Image::from_file(icon_path);
    label_box.append(&icon_image);

    let label = Label::new(Some(&tab_title));
    let close_button = gtk::Button::with_label("x");

    label_box.append(&label);
    label_box.append(&close_button);

    let index = notebook.append_page(&hbox, Some(&label_box));
    notebook.set_current_page(Some(index));
    notebook.set_tab_reorderable(&hbox, true);
    notebook.set_tab_detachable(&hbox, false);

    let overlay = Overlay::new();
    overlay.set_child(Some(&scrolled_window));

    overlay.add_overlay(&file_path_label);
    file_path_label.set_halign(gtk::Align::End);
    file_path_label.set_valign(gtk::Align::End);

    hbox.append(&overlay);

    let index_usize: usize = index.try_into().unwrap();
    
    let mut map = FILE_PATH_MAP.lock().unwrap();
    map.insert(index_usize, file_path);

    let notebook_clone = notebook.clone();
    close_button.connect_clicked(move |_| {
        let current_index = index_usize;
        
        // Remove the page from the notebook
        notebook_clone.remove_page(Some(current_index.try_into().unwrap()));

        // Update the file path mappings
        let mut map = FILE_PATH_MAP.lock().unwrap();
        
        // Remove the current index
        map.remove(&current_index);

        // Create a new map with updated indices
        let mut new_map = HashMap::new();
        
        // Redistribute the remaining file paths to their new indices
        for (old_index, path) in map.iter() {
            if *old_index > current_index {
                // If the index is greater than the removed index, decrease it by 1
                new_map.insert(old_index - 1, path.clone());
            } else {
                // If the index is less than the removed index, keep it the same
                new_map.insert(*old_index, path.clone());
            }
        }

        // Replace the old map with the new one
        *map = new_map;
    });
}

pub fn get_page_buffer(notebook: &Notebook) -> Option<TextBuffer> {
    let current_page = notebook.current_page();

    if let Some(widget) = notebook.nth_page(current_page) {
        if let Some(box_widget) = widget.downcast_ref::<gtk::Box>() {
            if let Some(first_child) = box_widget.first_child() {
                if let Some(overlay) = first_child.downcast_ref::<Overlay>() {
                    if let Some(child) = overlay.child() {
                        if let Some(scrolled_window) = child.downcast_ref::<ScrolledWindow>() {
                            if let Some(text_view) = scrolled_window.child() {
                                if let Some(text_view) = text_view.downcast_ref::<TextView>() {
                                    return Some(text_view.buffer());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}


pub fn get_page_file_path(notebook: &Notebook) -> Option<PathBuf> {
    let current_page = notebook.current_page();
    println!("{:?}", current_page);

    if let Some(index) = current_page {
        let map = FILE_PATH_MAP.lock().unwrap();
        map.get(&(index as usize)).cloned()
    } else {
        None
    }
}


