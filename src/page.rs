use gtk::{Image, Label, Notebook, gio, glib};
use gtk::prelude::*;
use sourceview5::prelude::BufferExt;
use std::convert::TryInto;
use sourceview5::{prelude::*, LanguageManager};

pub fn create_page(notebook: &Notebook, path: &str) {

    let buffer = sourceview5::Buffer::new(None);
    buffer.set_highlight_syntax(true);

    let language_manager = LanguageManager::new();

    let lang = language_manager.guess_language(path.rsplit("/").next(), None)
    .map_or_else(|| "txt".to_string(), |language| language.id().to_string());

    if let Some(ref language) = sourceview5::LanguageManager::new().language(&lang) {
        buffer.set_language(Some(language));
    }   

    if let Some(ref scheme) = sourceview5::StyleSchemeManager::new().scheme("Adwaita-dark") {
        buffer.set_style_scheme(Some(scheme));
    }

    let file = gio::File::for_path(path);
    let file = sourceview5::File::builder().location(&file).build();
    let loader = sourceview5::FileLoader::new(&buffer, &file);

    loader.load_async_with_callback(
        glib::Priority::default(),
        gio::Cancellable::NONE,
        move |current_num_bytes, total_num_bytes| {
            println!(
                "loading: {:?}",
                (current_num_bytes as f32 / total_num_bytes as f32) * 100f32
            );
        },
        |res| {
            println!("loaded: {:?}", res);
        },
    );

    let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    let scroll = gtk4::ScrolledWindow::builder()
        .vscrollbar_policy(gtk4::PolicyType::External)
        .build();

    let view = sourceview5::View::with_buffer(&buffer);
    view.set_monospace(true);
    view.set_background_pattern(sourceview5::BackgroundPatternType::Grid);
    view.set_show_line_numbers(true);
    view.set_highlight_current_line(true);
    view.set_tab_width(4);
    view.set_hexpand(true);
    scroll.set_child(Some(&view));
    container.append(&scroll);
    let map = sourceview5::Map::new();
    map.set_view(&view);
    container.append(&map);

    let label_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .margin_start(6)
        .margin_end(6)
        .margin_top(6)
        .margin_bottom(6)
        .build();

    label_box.set_homogeneous(true);
    label_box.set_hexpand(true);

    let left_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    left_label_box.set_halign(gtk::Align::Start);

    let icon_image = Image::from_file("");
    left_label_box.append(&icon_image);

    let label = Label::new(Some(&loader.location().expect("No location").uri().rsplit("/").next().expect("Error getting file name")));
    let close_button = gtk::Button::with_label("x");
    
    close_button.set_halign(gtk::Align::End);

    left_label_box.append(&label);

    label_box.append(&left_label_box);
    label_box.append(&close_button);

    let index = notebook.append_page(&container, Some(&label_box));
    notebook.set_current_page(Some(index));
    notebook.set_tab_reorderable(&container, true);
    notebook.set_tab_detachable(&container, false);

    let index_usize: usize = index.try_into().unwrap();

    let notebook_clone = notebook.clone();
    close_button.connect_clicked(move |_| {
        let current_index = index_usize;
        
        notebook_clone.remove_page(Some(current_index.try_into().unwrap()));
    });
}
