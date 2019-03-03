use cursive::traits::Identifiable;
use cursive::view::{Boxable, Scrollable};
use cursive::views::{Dialog, EditView, IdView, LinearLayout, Panel, SelectView};
use cursive::Cursive;

use pretty_bytes::converter::convert as format_bytes;
use std::cell::RefCell;
use std::thread;

extern crate grim_rust;
use grim_rust::ewfargs;
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
fn new_entry_box(name: &str, max_size: usize, default: &Option<String>) -> Panel<IdView<EditView>> {
    match default {
        None => Panel::new(EditView::new().max_content_width(max_size).with_id(name)),
        Some(content) => Panel::new(
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

    let mut select = SelectView::<MenuOptions>::new().on_submit(on_submit);

    select.add_item("Image:    Image a hard drive", MenuOptions::Image);
    select.add_item("Settings: (unavailable)", MenuOptions::Settings);
    select.add_item("Exit:     Exit grim", MenuOptions::Exit);

    s.add_layer(Dialog::around(select).title(format!("grim {}", env!("CARGO_PKG_VERSION"))));
}

/// Display the form for selecting a drive to image.
///
/// # Arguments
///
/// * `s` - A mutable reference to the `Cursive` instance to display on.
///
/// # Buttons
///
/// * "Cancel" - Return to the main menu.
/// * Submit the select view to continue to the examiner information form.
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

    // We want to display a message while getting the list of hard drives, which
    // might take a while
    s.pop_layer();
    s.add_layer(Dialog::text("Getting list of hard drives, please wait..."));

    // The message won't display until this function returns, however;
    // to work around that, we need to spawn a thread and get the list
    // of hard drives from inside it.

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
            let size = if disk.units == "bytes" {
                format_bytes(disk.size)
            } else {
                format!("{} {}", disk.size, disk.units)
            };

            let display_string = format!("{} {} ({})", size, disk.product, disk.logical_name);

            disks.push((display_string.clone(), disk));
        }

        cb_sink
            .send(Box::new(move |s: &mut Cursive| {
                if disks.is_empty() {
                    s.pop_layer();
                    s.add_layer(
                        Dialog::text("ERROR: No disks found! Are you running as root?")
                            .title("Error")
                            .button("Ok", main_menu),
                    );
                    return;
                }
                let mut select = SelectView::<sysinfo::HdInfo>::new().on_submit(on_submit);

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
                let success = ARGS.with(|args| -> bool {
                    let mut args = args.borrow_mut();
                    if extract_field_required(s, "Examiner Name", &mut args.examiner_name).is_err()
                        || extract_field_required(s, "Case Number", &mut args.case_number).is_err()
                        || extract_field_required(s, "Evidence Number", &mut args.evidence_number)
                            .is_err()
                    {
                        return false;
                    }
                    extract_field_optional(s, "Description", &mut args.description);
                    extract_field_optional(s, "Notes", &mut args.notes);
                    true
                });

                if !success {
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

    let num_sectors_select = SelectView::<ewfargs::NumSectors>::new()
        .item("16 Bytes", ewfargs::NumSectors::Sectors16)
        .item("32 Bytes", ewfargs::NumSectors::Sectors32)
        .item("64 Bytes", ewfargs::NumSectors::Sectors64)
        .item("128 Bytes", ewfargs::NumSectors::Sectors128)
        .item("256 Bytes", ewfargs::NumSectors::Sectors256)
        .item("512 Bytes", ewfargs::NumSectors::Sectors512)
        .item("1 Kilobyte", ewfargs::NumSectors::Sectors1024)
        .item("2 Kilobytes", ewfargs::NumSectors::Sectors2048)
        .item("4 Kilobytes", ewfargs::NumSectors::Sectors4096)
        .item("8 Kilobytes", ewfargs::NumSectors::Sectors8192)
        .item("16 Kilobytes", ewfargs::NumSectors::Sectors16384)
        .item("32 Kilobytes", ewfargs::NumSectors::Sectors32768)
        .scrollable()
        .max_height(5);

    s.add_layer(
        Dialog::around(Panel::new(num_sectors_select).title("Bytes per Sector"))
            .title("Required information")
            .button("Back", examiner_info)
            .button("Next", |_| ()),
    );
}
