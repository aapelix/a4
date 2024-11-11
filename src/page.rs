use gtk::pango::TabArray;
use gtk::{Image, Label, Notebook, Overlay, ScrolledWindow, TextBuffer, TextView};
use gtk::prelude::*;
use std::cell::RefCell;
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

    text_view.set_left_margin(10);
    text_view.set_bottom_margin(10);
    text_view.set_right_margin(10);
    text_view.set_top_margin(5);
    text_view.set_tabs(&TabArray::new(4, true));

    let buffer = text_view.buffer();

    let text_view_style = text_view.style_context();

    text_view_style.add_class("textview");

    let provider = gtk::CssProvider::new();

    provider
        .load_from_data(
            "
            .textview {
                font-size: 16pt; 
                letter-spacing: 0px;
                line-height: 1;
            }
            ",
        );

    text_view_style.add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    setup_syntax_highlighting(&buffer);

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
    file_path_label.set_margin_bottom(5);
    file_path_label.set_margin_end(5);

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

use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Style};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| SyntaxSet::load_defaults_newlines());
static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| ThemeSet::load_defaults());

pub fn setup_syntax_highlighting(buffer: &gtk::TextBuffer) {

    println!("Setting up syntax highlighting");

    let theme = &THEME_SET.themes["base16-ocean.dark"];
    let syntax = SYNTAX_SET.find_syntax_by_extension("rs")
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
    
    let highlighter = RefCell::new(HighlightLines::new(syntax, theme));

    // Connect to the buffer's changed signal
    buffer.connect_changed(move |buffer| {
        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false)
            .to_string();
        
        buffer.remove_all_tags(&buffer.start_iter(), &buffer.end_iter());
        
        let mut offset = 0;
        for line in LinesWithEndings::from(&text) {
            let ranges = highlighter.borrow_mut()
                .highlight_line(line, &SYNTAX_SET)
                .unwrap_or_default();
            
            for (style, text) in ranges {
                apply_style(buffer, style, offset, text.len());
                offset += text.len();
            }
        }
    });
}

fn apply_style(buffer: &gtk::TextBuffer, style: Style, offset: usize, length: usize) {
    let start_iter = buffer.iter_at_offset(offset as i32);
    let end_iter = buffer.iter_at_offset((offset + length) as i32);
    
    // Create or reuse tag for this style
    let tag_name = format!("style_{:x}_{:x}_{:x}", style.foreground.r, style.foreground.g, style.foreground.b);
    let tag = if let Some(tag) = buffer.tag_table().lookup(&tag_name) {
        tag
    } else {
        let tag = gtk::TextTag::new(Some(&tag_name));
        tag.set_property("foreground", &format!("#{:02x}{:02x}{:02x}", 
            style.foreground.r, style.foreground.g, style.foreground.b));
        if style.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
            tag.set_property("weight", 700i32);
        }
        if style.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
            tag.set_property("style", gtk::pango::Style::Italic);
        }
        buffer.tag_table().add(&tag);
        tag
    };
    
    buffer.apply_tag(&tag, &start_iter, &end_iter);
}

