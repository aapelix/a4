use gtk::Notebook;
use gtk::prelude::*;

use crate::page::create_page;

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
                    let path = file_path.into_os_string().into_string().unwrap();
                    create_page(&notebook_clone, &path);
                }
            }

            dialog.close();
        });

        file_chooser.show();
}
