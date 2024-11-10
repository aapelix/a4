extern crate gtk4 as gtk;
use file::{open_file_dialog, save_file};
use gtk::glib::Propagation;
use gtk::{prelude::*, EventControllerKey, Label, ListBox, ListBoxRow, Notebook, Paned};
use gtk::{Application, ApplicationWindow};
use page::{create_page, get_page_buffer, get_page_file_path};
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::cell::RefCell;
use std::rc::Rc;

mod file;
mod page;

fn create_ui(app: &Application) {
    let window = ApplicationWindow::new(app);
    window.set_title(Some("a4"));
    window.set_default_size(800, 600);

    let hbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .margin_start(6)
        .margin_end(6)
        .margin_top(6)
        .margin_bottom(6)
        .build();

    let paned = Paned::new(gtk::Orientation::Horizontal);

    let notebook = Notebook::new();
    notebook.set_hexpand(true);

    let side_bar = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let list_box = ListBox::new();
    let open_folder_button = gtk::Button::with_label("Open Folder");

    let list_box_clone = list_box.clone();

    open_folder_button.connect_clicked(move |_| {
        let folder_chooser = gtk::FileChooserDialog::builder()
            .title("Open folder")
            .action(gtk::FileChooserAction::SelectFolder)
            .build();

        folder_chooser.add_button("Cancel", gtk::ResponseType::Cancel);
        folder_chooser.add_button("Open", gtk::ResponseType::Accept);

        let list_box = list_box_clone.clone();

        folder_chooser.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file_path) = dialog.file().and_then(|file| file.path()) {
                    println!("{:?}", file_path);
                    
                    if file_path.is_dir() {
                        load_folder_contents(&list_box, &file_path);
                    }
                }
            }
        
            dialog.close();
        });

        folder_chooser.show();
    });

    let open_button = gtk::Button::with_label("Open file");
    let notebook_clone = notebook.clone();
    open_button.connect_clicked(move |_| {
        open_file_dialog(&notebook_clone);
    });

    create_page(&notebook, None);

    side_bar.append(&open_button);
    side_bar.append(&open_folder_button);
    side_bar.append(&list_box);

    paned.set_start_child(Some(&side_bar));
    paned.set_end_child(Some(&notebook));

    hbox.append(&paned);

    window.set_child(Some(&hbox));

    let key_controller = EventControllerKey::new();
    let notebook_clone = notebook.clone();
    key_controller.connect_key_pressed(move |_, key, _, modifier| {
        println!("Key pressed: {:?}, {:?}", key.name(), modifier);
        if let Some(key) = key.name() {
            if key == "s" && modifier == gtk::gdk::ModifierType::CONTROL_MASK {
                println!("Save file");
                println!("{:?}", get_page_file_path(&notebook_clone));
                let buffer = get_page_buffer(&notebook_clone);
                if let Some(buffer) = buffer {
                    println!("{:?}", buffer.text(&buffer.start_iter(), &buffer.end_iter(), true));
                    if let Some(path) = get_page_file_path(&notebook_clone) {
                        println!("{:?}", path);
                        save_file(&buffer, &path);
                    }
                }
            }
        }
        Propagation::Stop
    });

    window.add_controller(key_controller);
    window.show();
}

fn load_folder_contents(list_box: &ListBox, path: &Path) {

    let entries: Vec<PathBuf> = path
        .read_dir()
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();

    // Sort entries to group folders and files
    let mut sorted_entries = entries.clone();
    sorted_entries.sort_by(|a, b| {
        let a_is_dir = a.is_dir();
        let b_is_dir = b.is_dir();
        
        if a_is_dir && !b_is_dir {
            std::cmp::Ordering::Less
        } else if !a_is_dir && b_is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.file_name().cmp(&b.file_name())
        }
    });

    for entry_path in sorted_entries {
        let entry_name = entry_path.file_name().unwrap_or_default();
        let entry_name_str = entry_name.to_string_lossy();

        let row = ListBoxRow::new();
        row.set_margin_top(if list_box.first_child().is_none() { 10 } else { 0 });

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        hbox.set_halign(gtk::Align::Start);

        let label = Label::new(Some(&entry_name_str));
        label.set_halign(gtk::Align::Start);
        label.set_margin_start(10);

        // Create an expander for directories
        let expander = if entry_path.is_dir() {
            let exp = gtk::Image::from_icon_name("pan-end-symbolic");
            exp.set_margin_end(5);
            Some(exp)
        } else {
            None
        };

        // Add an icon for folders/files
        let icon = if entry_path.is_dir() {
            gtk::Image::from_icon_name("folder-symbolic")
        } else {
            gtk::Image::from_icon_name("text-x-generic-symbolic")
        };
        icon.set_margin_end(5);

        // State to track expansion
        let is_expanded = Rc::new(RefCell::new(false));

        if let Some(exp) = expander.clone() {
            hbox.append(&exp);
        }
        hbox.append(&icon);
        hbox.append(&label);

        row.set_child(Some(&hbox));
        list_box.append(&row);
        
        if entry_path.is_dir() {
            row.set_activatable(true);
            let row_clone = row.clone();
            let list_box_clone = list_box.clone();
            let entry_path_clone = entry_path.clone();
            let is_expanded_clone = is_expanded.clone();
            let expander_clone = expander.clone();
            row.connect_activate(move |_| {

                let mut expanded = is_expanded_clone.borrow_mut();
                
                // Toggle expansion state
                *expanded = !*expanded;

                // Rotate expander icon
                if let Some(exp) = &expander_clone {
                    exp.set_icon_name(if *expanded {
                        Some("pan-down-symbolic")
                    } else {
                        Some("pan-end-symbolic")
                    });
                }

                // Find the position of the current row
                let mut position = 0;
                let mut current_child = list_box_clone.first_child();
                while let Some(child) = current_child {
                    if child == row_clone {
                        break;
                    }
                    position += 1;
                    current_child = child.next_sibling();
                }

                // If expanding, add subdirectories
                if *expanded {
                    if let Ok(entries) = entry_path_clone.read_dir() {
                        for sub_entry in entries.filter_map(Result::ok) {
                            let sub_path = sub_entry.path();
                            let sub_name = sub_path.file_name().unwrap_or_default();
                            let sub_name_str = sub_name.to_string_lossy();

                            let sub_row = ListBoxRow::new();
                            
                            let sub_hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);
                            sub_hbox.set_halign(gtk::Align::Start);

                            let sub_label = Label::new(Some(&format!("  {}", sub_name_str)));
                            sub_label.set_halign(gtk::Align::Start);
                            sub_label.set_margin_start(20);

                            // Subdirectory expander
                            let sub_expander = if sub_path.is_dir() {
                                let exp = gtk::Image::from_icon_name("pan-end-symbolic");
                                exp.set_margin_end(5);
                                Some(exp)
                            } else {
                                None
                            };

                            let sub_icon = if sub_path.is_dir() {
                                gtk::Image::from_icon_name("folder-symbolic")
                            } else {
                                gtk::Image::from_icon_name("text-x-generic-symbolic")
                            };
                            sub_icon.set_margin_end(5);

                            if let Some(exp) = sub_expander {
                                sub_hbox.append(&exp);
                            }
                            sub_hbox.append(&sub_icon);
                            sub_hbox.append(&sub_label);
                            sub_row.set_child(Some(&sub_hbox));

                            // Insert the new row after the parent row
                            list_box_clone.insert(&sub_row, position + 1);
                            position += 1;
                        }
                    }
                } else {
                    // Collapse: remove all rows after the current row until a row at the same or higher level
                    let mut next_child = row_clone.next_sibling();
                    while let Some(child) = next_child {
                        let next = child.next_sibling();
                        if let Some(row) = child.downcast_ref::<ListBoxRow>() {
                            list_box_clone.remove(row);
                        }
                        next_child = next;
                    }
                }
            });
        }
    }
}

fn main() {
    env::set_var("GTK_THEME", "Adwaita:dark");

    let app = Application::new(Some("com.aapelix.a4"), Default::default());
    app.connect_activate(|app| create_ui(app));
    app.run();
}
