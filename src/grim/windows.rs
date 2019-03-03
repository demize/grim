use cursive::traits::Identifiable;
use cursive::views::{Dialog, EditView, LinearLayout, SelectView};
use cursive::Cursive;
use pretty_bytes::converter::convert as format_bytes;
use std::cell::RefCell;
use std::thread;

extern crate grim_rust;
use grim_rust::ewfargs::ArgsList;
use grim_rust::sysinfo;
use grim_rust::LoggingInfo;

thread_local! {
    static ARGS: RefCell<ArgsList> = RefCell::new(ArgsList::new());
    static INFO: RefCell<LoggingInfo> = RefCell::new(LoggingInfo::new());
}

enum ExtractionError {
    Blank,
}

/// Return a Dialog containing an Edit view, with the ID and title both set to `name`. Please use this to generate all text inputs, as it makes the code much cleaner.
fn new_entry_box(name: &str, max_size: usize, default: &Option<String>) -> Dialog {
    match default {
        None => Dialog::around(EditView::new().max_content_width(max_size).with_id(name)),
        Some(content) => Dialog::around(
            EditView::new()
                .max_content_width(max_size)
                .content(content.clone())
                .with_id(name),
        ),
    }
    .title(name)
}

/// Extract the information from the field with the given ID into the Option specified. Displays an infobox when it encounters an empty input.
///
/// # Arguments
///
/// * `s` - A mutable reference to the `Cursive` instance to display on.
/// * `from` - The ID of the field to extract from.
/// * `to` - A mutable reference to the Option to extract the field into.
///
/// # Return values
///
/// Returns an empty result if the field was extracted successfully, or `ExtractionError::Blank` if the field was blank.
///
/// # Panics
///
/// Panics when the field cannot be found.
fn extract_field_required(
    s: &mut Cursive,
    from: &str,
    to: &mut Option<String>,
) -> Result<(), ExtractionError> {
    match s.call_on_id(from, |view: &mut EditView| view.get_content()) {
        Some(ref value) if !(*value).is_empty() => {
            to.replace((**value).clone());
            Ok(())
        }
        Some(_) => {
            s.add_layer(Dialog::info(format!("{} is required.", from)));
            Err(ExtractionError::Blank)
        }
        None => panic!(format!("Can't find element with ID {}", from)),
    }
}

/// Extract the information from the field with the given ID into the Option specified.
///
/// # Arguments
///
/// * `s` - A mutable reference to the `Cursive` instance to display on.
/// * `from` - The ID of the field to extract from.
/// * `to` - A mutable reference to the Option to extract the field into.
///
/// # Panics
///
/// Panics when the field cannot be found.
fn extract_field_optional(s: &mut Cursive, from: &str, to: &mut Option<String>) {
    match s.call_on_id(from, |view: &mut EditView| view.get_content()) {
        Some(ref value) if !(*value).is_empty() => {
            to.replace((**value).clone());
        }
        Some(_) => {
            to.replace("".to_string());
        }
        None => panic!(format!("Can't find element with ID {}", from)),
    }
}

/// Display the main menu, and bring the user to the option they choose. Displays options for imaging a hard drive, editing settings, and exiting the program.
///
/// # Arguments
///
/// * `s` - A mutable reference to the `Cursive` instance to display on.
///
/// # Buttons
///
/// * "Image" - Start the flow to image a hard drive by running `examiner_info`.
/// * "Options" - Start the flow to edit the application options; currently unimplemented.
/// * "Exit" - Exit the application.
pub fn main_menu(s: &mut Cursive) {
    /// Describes possible options for the user to choose in the menu.
    enum MenuOptions {
        Image,
        Settings,
        Exit,
    }
    /// Callback for when the user selects an option.
    ///
    /// # Arguments
    ///
    /// * `s` - A mutable reference to the `Cursive` instance to display on.
    /// * `selection` - A reference to a `MenuOptions` value describing the user's selection.
    fn on_submit(s: &mut Cursive, selection: &MenuOptions) {
        match selection {
            MenuOptions::Image => {
                INFO.with(|info| {
                    info.replace(LoggingInfo::new());
                });
                ARGS.with(|args| {
                    args.replace(ArgsList::new());
                });
                select_source(s);
            }
            MenuOptions::Settings => (), // Settings page to come later
            MenuOptions::Exit => s.quit(),
        }
    }

    s.pop_layer();

    let mut select = SelectView::new().on_submit(on_submit);

    select.add_item("Image:    Image a hard drive", MenuOptions::Image);
    select.add_item("Settings: (unavailable)", MenuOptions::Settings);
    select.add_item("Exit:     Exit grim", MenuOptions::Exit);

    s.add_layer(Dialog::around(select).title(format!("grim {}", env!("CARGO_PKG_VERSION"))));
}

pub fn select_source(s: &mut Cursive) {
    fn on_submit(s: &mut Cursive, selection: &sysinfo::HdInfo) {
        INFO.with(|info| {
            let mut info = info.borrow_mut();
            info.drive_product = Some(selection.product.clone());
            info.drive_serial = Some(selection.serial.clone());
        });
        ARGS.with(|args| {
            let mut args = args.borrow_mut();
            args.source_device = Some(selection.logical_name.clone());
        });
        examiner_info(s);
    }

    s.pop_layer();
    s.add_layer(Dialog::text("Getting list of hard drives, please wait..."));

    let cb_sink = s.cb_sink().clone();

    thread::spawn(move || {
        let list = sysinfo::get_all_disks();
        if let Err(e) = list {
            cb_sink
                .send(Box::new(move |s: &mut Cursive| {
                    s.pop_layer();
                    s.add_layer(Dialog::text(e.to_string()).button("Exit", Cursive::quit));
                    return;
                }))
                .unwrap();
            return;
        }

        if let Ok(host_serial) = sysinfo::get_pc_serial() {
            INFO.with(|info| {
                info.borrow_mut().host_serial = Some(host_serial);
            });
        }
        let mut disks = Vec::<(String, sysinfo::HdInfo)>::new();

        for disk in list.unwrap() {
            let size: String;
            if disk.units == "bytes" {
                size = format_bytes(disk.size);
            } else {
                size = format!("{} {}", disk.size, disk.units);
            }

            let display_string = format!("{} {} ({})", size, disk.product, disk.logical_name);

            disks.push((display_string.clone(), disk));
        }

        cb_sink
            .send(Box::new(move |s: &mut Cursive| {
                if disks.len() == 0 {
                    s.pop_layer();
                    s.add_layer(
                        Dialog::text("ERROR: No disks found! Are you running as root?")
                            .title("Error")
                            .button("Ok", main_menu),
                    );
                    return;
                }
                let mut select = SelectView::new().on_submit(on_submit);

                for disk in disks {
                    select.add_item(disk.0, disk.1);
                }

                s.pop_layer();
                s.add_layer(
                    Dialog::around(select)
                        .title("Select a disk to image")
                        .button("Cancel", main_menu),
                );
            }))
            .unwrap();
    });
}

/// Display the form for entering examiner and case information.
///
/// # Arguments
///
/// * `s` - A mutable reference to the `Cursive` instance to display on.
///
/// # Buttons
///
/// * "Next" - Move on to the information required by libewf by calling `required_info`.
pub fn examiner_info(s: &mut Cursive) {
    s.pop_layer();

    let fields = ARGS.with(|args| {
        let args = args.borrow();
        LinearLayout::vertical()
            .child(new_entry_box("Examiner Name", 256, &args.examiner_name))
            .child(new_entry_box("Case Number", 256, &args.case_number))
            .child(new_entry_box("Evidence Number", 256, &args.evidence_number))
            .child(new_entry_box("Description", 256, &args.description))
            .child(new_entry_box("Notes", 1024, &args.notes))
    });

    s.add_layer(
        Dialog::around(fields)
            .padding((1, 1, 1, 0))
            .title("Examiner information")
            .button("Back", select_source)
            .button("Next", |s| {
                if !ARGS.with(|args| -> bool {
                    let mut args = args.borrow_mut();
                    if let Err(_) =
                        extract_field_required(s, "Examiner Name", &mut args.examiner_name)
                    {
                        return false;
                    };
                    if let Err(_) = extract_field_required(s, "Case Number", &mut args.case_number)
                    {
                        return false;
                    }
                    if let Err(_) =
                        extract_field_required(s, "Evidence Number", &mut args.evidence_number)
                    {
                        return false;
                    }
                    extract_field_optional(s, "Description", &mut args.description);
                    extract_field_optional(s, "Notes", &mut args.notes);
                    return true;
                }) {
                    return;
                };
                required_info(s);
            }),
    );
}

/// Display the form for entering information required by libewf.
///
/// # Arguments
///
/// * `s` - A mutable reference to the `Cursive` instance to display on.
///
/// # Buttons
///
/// * "Next" - Currently not implemented.
/// * "Back" - Return to the examiner information form.
pub fn required_info(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::text("Not implemented!")
            .title("Required Information")
            .button("Back", examiner_info),
    );
}
