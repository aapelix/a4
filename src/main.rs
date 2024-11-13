extern crate gtk4 as gtk;
use file::open_file_dialog;
use gtk::glib::Propagation;
use gtk::{prelude::*, EventControllerKey, Notebook, Paned};
use gtk::{Application, ApplicationWindow};
use page::create_page;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

mod file;
mod page;

#[derive(Debug, Clone)]
enum Column {
    Filename = 0,
    Path = 1,
    IsDir = 2,
}

fn create_ui(app: &Application) {
    let window = ApplicationWindow::new(app);
    window.set_title(Some("a4"));
    window.set_default_size(800, 600);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let hbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .margin_start(6)
        .margin_end(6)
        .margin_top(6)
        .margin_bottom(6)
        .build();

    let command_palette_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let command_palette= gtk::Entry::builder().placeholder_text("Type a command...").hexpand(true).width_request(600).build();

    command_palette_box.set_halign(gtk::Align::Center);
    command_palette_box.append(&command_palette);

    command_palette_box.set_margin_top(5);

    let mut command_map: HashMap<String, Box<dyn Fn()>> = HashMap::new();

    fn add_command(command_map: &mut HashMap<String, Box<dyn Fn()>>, name: &str, action: Box<dyn Fn()>) {
        command_map.insert(name.to_string(), action);
    }

    vbox.append(&command_palette_box);

    let paned = Paned::new(gtk::Orientation::Horizontal);

    let notebook = Notebook::new();
    notebook.set_hexpand(true);

    let side_bar = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let view_selector = gtk::ComboBoxText::new();
    view_selector.append_text("Explorer");
    view_selector.append_text("Extensions");
    view_selector.set_active(Some(0));
    side_bar.append(&view_selector);

    let view_stack = gtk::Stack::new();
    view_stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
    side_bar.append(&view_stack);

    let store = gtk::TreeStore::new(&[
        String::static_type(),    // Filename
        String::static_type(),    // Path
        bool::static_type(),      // IsDir
    ]);

    let tree_view = gtk::TreeView::with_model(&store);
    tree_view.set_headers_visible(false);

    let filename_column = gtk::TreeViewColumn::new();
    let cell_icon = gtk::CellRendererPixbuf::new();
    let cell_text = gtk::CellRendererText::new();
    
    filename_column.pack_start(&cell_icon, false);
    filename_column.pack_start(&cell_text, true);
    
    filename_column.set_cell_data_func(&cell_icon, |_column, cell, model, iter| {
        let cell = cell.downcast_ref::<gtk::CellRendererPixbuf>().unwrap();
        let is_dir = model.get::<bool>(iter, Column::IsDir as i32);

        cell.set_property(
            "icon-name",
            if is_dir { "folder-symbolic" } else { "text-x-generic-symbolic" },
        );
    });

    filename_column.set_cell_data_func(&cell_text, |_column, cell, model, iter| {
        let cell = cell.downcast_ref::<gtk::CellRendererText>().unwrap();
        let filename = model.get::<String>(iter, Column::Filename as i32);
        cell.set_property("text", &filename);
    });

    tree_view.append_column(&filename_column);

    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_vexpand(true);
    scrolled_window.set_child(Some(&tree_view));

    let store_clone = store.clone();
    
    add_command(&mut command_map, "open folder", Box::new(move || {
        let folder_chooser = gtk::FileChooserDialog::builder()
            .title("Open folder")
            .action(gtk::FileChooserAction::SelectFolder)
            .build();

        folder_chooser.add_button("Cancel", gtk::ResponseType::Cancel);
        folder_chooser.add_button("Open", gtk::ResponseType::Accept);

        let store = store_clone.clone();

        folder_chooser.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file_path) = dialog.file().and_then(|file| file.path()) {
                    store.clear();
                    load_folder_contents(&store, None, &file_path);
                }
            }
            dialog.close();
        });

        folder_chooser.show();
    }));

    let notebook_clone = notebook.clone();

    add_command(&mut command_map, "open file", Box::new(move || {
        open_file_dialog(&notebook_clone);
    }));

    let notebook_clone = notebook.clone();
    tree_view.connect_row_activated(move |tree_view, path, _column| {
        let model = tree_view.model().unwrap();
        let iter = model.iter(path).unwrap();
        
        let is_dir = model.get::<bool>(&iter, Column::IsDir as i32);
        let file_path = model.get::<String>(&iter, Column::Path as i32);
        
        if !is_dir {
            let path = PathBuf::from(file_path);
            let path = path.into_os_string().into_string().unwrap();
            println!("{}", path);
            create_page(&notebook_clone, &path);
        }
    });

    create_page(&notebook, "./src/file.rs");

    view_stack.add_named(&scrolled_window, Some("Explorer"));

    // Create and add a placeholder for the Extensions view
    let extensions_view = gtk::Label::new(Some("Extensions view coming soon..."));
    view_stack.add_named(&extensions_view, Some("Extensions"));

    view_selector.connect_changed(move |combo| {
        if let Some(active_text) = combo.active_text() {
            view_stack.set_visible_child_name(active_text.as_str());
        }
    });

    paned.set_start_child(Some(&side_bar));
    paned.set_end_child(Some(&notebook));

    paned.set_position(200);

    hbox.append(&paned);
    vbox.append(&hbox);

    command_palette.connect_activate(move |entry| {
        let text = entry.text().to_string().to_lowercase();

        if let Some(action) = command_map.get(&text) {
            action();
            entry.set_text("");
        } else {
            println!("Command not found: {}", text);
        }
    });

    window.set_child(Some(&vbox));

    let key_controller = EventControllerKey::new();
    let notebook_clone = notebook.clone();
    key_controller.connect_key_pressed(move |_, key, _, modifier| {
        if let Some(key) = key.name() {
            if key == "s" && modifier == gtk::gdk::ModifierType::CONTROL_MASK {
                println!("Save file")
            }
        }
        Propagation::Stop
    });

    window.add_controller(key_controller);
    window.show();
}

fn load_folder_contents(store: &gtk::TreeStore, parent: Option<&gtk::TreeIter>, path: &Path) {
    if let Ok(entries) = path.read_dir() {
        let mut entries: Vec<_> = entries
            .filter_map(Result::ok)
            .collect();

        entries.sort_by(|a, b| {
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();
            
            if a_is_dir && !b_is_dir {
                std::cmp::Ordering::Less
            } else if !a_is_dir && b_is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.file_name().cmp(&b.file_name())
            }
        });

        for entry in entries {
            let path = entry.path();
            let is_dir = path.is_dir();
            let filename = entry.file_name().to_string_lossy().to_string();
            let path_str = path.to_string_lossy().to_string();

            let iter = store.append(parent);
            store.set(&iter, &[
                (Column::Filename as u32, &filename),
                (Column::Path as u32, &path_str),
                (Column::IsDir as u32, &is_dir),
            ]);

            if is_dir {
                load_folder_contents(store, Some(&iter), &path);
            }
        }
    }
}

fn main() {
    env::set_var("GTK_THEME", "Adwaita:dark");

    let app = Application::new(Some("com.aapelix.a4"), Default::default());
    app.connect_activate(|app| create_ui(app));
    app.run();
}