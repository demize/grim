use cursive::traits::Identifiable;
use cursive::view::Boxable;
use cursive::views::{
    BoxView, Checkbox, Dialog, EditView, IdView, LinearLayout, ListView, SelectView, TextView,
};
use cursive::Cursive;

use pretty_bytes::converter::convert as format_bytes;
use std::cell::RefCell;
use std::thread;

// For now, we just use this to validate the input
// It isn't guaranteed to stop ewfacquirestream from crashing, but it should help
// Maybe eventually we'll pass the size in bytes rather than just the user's input?
use convert_byte_size_string::convert_to_bytes;

extern crate grim_rust;
use grim_rust::ewfargs;
use grim_rust::ewfargs::ArgsList;
use grim_rust::sysinfo;
use grim_rust::LoggingInfo;

// Some things need to both be mutable and available to all our forms, so thread
// local storage is the ideal solution
thread_local! {
    static ARGS: RefCell<ArgsList> = RefCell::new(ArgsList::new());
    static INFO: RefCell<LoggingInfo> = RefCell::new(LoggingInfo::new());
}

// This could be expanded later, but for now is just used when the field is blank
enum ExtractionError {
    Blank,
}

/// Return an IdView containing an Edit view, with the ID `name`.
/// Please use this to generate all text inputs, as it makes the code much cleaner.
fn new_entry_box<F>(
    name: &str,
    max_size: usize,
    default: &Option<String>,
    next: F,
) -> BoxView<IdView<EditView>>
where
    F: Fn(&mut Cursive, &str) + 'static,
{
    match default {
        None => EditView::new()
            .on_submit(next)
            .max_content_width(max_size)
            .with_id(name)
            .min_width(45),
        Some(content) => EditView::new()
            .on_submit(next)
            .max_content_width(max_size)
            .content(content.clone())
            .with_id(name)
            .min_width(45),
    }
}

/// Extract the information from the entry box with the given ID into the Option specified. Displays an infobox when it encounters an empty input.
///
/// # Arguments
///
/// * `s` - A mutable reference to the `Cursive` instance to display on.
/// * `from` - The ID of the entry box to extract from.
/// * `to` - A mutable reference to the Option to extract the text from the entry box into.
///
/// # Return values
///
/// Returns an empty result if the entry box was extracted successfully, or `ExtractionError::Blank` if the field was blank.
///
/// # Panics
///
/// Panics when the entry box cannot be found.
fn extract_entrybox_required(
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

/// Extract the information from the entry box with the given ID into the Option specified.
///
/// # Arguments
///
/// * `s` - A mutable reference to the `Cursive` instance to display on.
/// * `from` - The ID of the entry box to extract from.
/// * `to` - A mutable reference to the Option to extract the text from the entry box into.
///
/// # Panics
///
/// Panics when the entry box cannot be found.
fn extract_entrybox_optional(s: &mut Cursive, from: &str, to: &mut Option<String>) {
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

/// Submit the examiner info form
fn examiner_info_next(s: &mut Cursive, _: &str) {
    let success = ARGS.with(|args| -> bool {
        let mut args = args.borrow_mut();
        if extract_entrybox_required(s, "Examiner Name", &mut args.examiner_name).is_err()
            || extract_entrybox_required(s, "Case Number", &mut args.case_number).is_err()
            || extract_entrybox_required(s, "Evidence Number", &mut args.evidence_number).is_err()
        {
            return false;
        }
        extract_entrybox_optional(s, "Description", &mut args.description);
        extract_entrybox_optional(s, "Notes", &mut args.notes);
        true
    });

    if !success {
        return;
    };
    target_info(s);
}

/// Display the form for entering examiner and case information.
///
/// # Arguments
///
/// * `s` - A mutable reference to the `Cursive` instance to display on.
///
/// # Buttons
///
/// * "Back" - Return to the source selection form.
/// * "Next" - Move on to the information required by libewf by calling `technical_options`.
pub fn examiner_info(s: &mut Cursive) {
    s.pop_layer();

    let fields = ARGS.with(|args| {
        let args = args.borrow();
        ListView::new()
            .child(
                "Examiner Name",
                new_entry_box(
                    "Examiner Name",
                    256,
                    &args.examiner_name,
                    examiner_info_next,
                ),
            )
            .child(
                "Case Number",
                new_entry_box("Case Number", 256, &args.case_number, examiner_info_next),
            )
            .child(
                "Evidence Number",
                new_entry_box(
                    "Evidence Number",
                    256,
                    &args.evidence_number,
                    examiner_info_next,
                ),
            )
            .child(
                "Description",
                new_entry_box("Description", 256, &args.description, examiner_info_next),
            )
            .child(
                "Notes",
                new_entry_box("Notes", 1024, &args.notes, examiner_info_next),
            )
    });

    s.add_layer(
        Dialog::around(fields)
            .padding((1, 1, 1, 0))
            .title("Examiner information")
            .button("Back", select_source)
            .button("Next", |s| examiner_info_next(s, "")),
    );
}

/// Submit the target info form
fn target_info_next(s: &mut Cursive) {
    let success = ARGS.with(|args| -> bool {
        let mut args = args.borrow_mut();
        if extract_entrybox_required(s, "Filename", &mut args.target_filename).is_err()
            || extract_entrybox_required(s, "Target directory", &mut args.target_dir).is_err()
        {
            return false;
        }

        let secondary_enabled = s
            .call_on_id("Two copies", |view: &mut Checkbox| -> bool {
                view.is_checked()
            })
            .unwrap();

        if secondary_enabled {
            if extract_entrybox_required(
                s,
                "Secondary target directory",
                &mut args.secondary_target_dir,
            )
            .is_err()
            {
                return false;
            }
        } else {
            args.secondary_target_dir = None;
        }

        // Extract the value from the select boxes
        args.ewf_format = s
            .call_on_id(
                "EwfFormat",
                |view: &mut SelectView<ewfargs::EwfFormat>| -> ewfargs::EwfFormat {
                    *(view.selection().unwrap()).clone()
                },
            )
            .unwrap();
        args.compression_type = s
            .call_on_id(
                "Compression type",
                |view: &mut SelectView<ewfargs::CompressionType>| -> ewfargs::CompressionType {
                    *(view.selection().unwrap()).clone()
                },
            )
            .unwrap();

        // Extract the values from the checkboxes
        let mut hashes = ewfargs::DigestType::MD5;

        hashes |= s
            .call_on_id("SHA1", |view: &mut Checkbox| {
                if view.is_checked() {
                    ewfargs::DigestType::SHA1
                } else {
                    ewfargs::DigestType::MD5
                }
            })
            .unwrap();
        hashes |= s
            .call_on_id("SHA256", |view: &mut Checkbox| {
                if view.is_checked() {
                    ewfargs::DigestType::SHA256
                } else {
                    ewfargs::DigestType::MD5
                }
            })
            .unwrap();

        args.digest_type = hashes;

        let segment = s
            .call_on_id("Segment", |view: &mut Checkbox| view.is_checked())
            .unwrap();
        if segment {
            let mut temp_size: Option<String> = None;
            let extraction_result = extract_entrybox_required(s, "Segment size", &mut temp_size);
            if extraction_result.is_err() {
                return false;
            }

            let temp_size_str = temp_size.unwrap();

            match convert_to_bytes(&temp_size_str) {
                Ok(_) => args.segment_file_size = Some(temp_size_str.clone()),
                Err(_) => {
                    s.add_layer(Dialog::info("Invalid value for segment size"));
                    return false;
                }
            }
        } else {
            args.segment_file_size = None;
        }

        true
    });
    if success {
        technical_options(s);
    }
}

/// Display the form for entering information about the target.
///
/// /// # Arguments
///
/// * `s` - A mutable reference to the `Cursive` instance to display on.
///
/// # Buttons
///
/// * "Next" - Continues to the required information form.
/// * "Back" - Returns to the examiner informaitn form.
pub fn target_info(s: &mut Cursive) {
    s.pop_layer();

    let mut two_copies = false;
    let mut segment = false;
    let fields = ARGS.with(|args| {
        let args = args.borrow();

        two_copies = args.secondary_target_dir.is_some();
        segment = args.segment_file_size.is_some();

        let ewf_select = SelectView::<ewfargs::EwfFormat>::new()
            .popup()
            .item("FTK", ewfargs::EwfFormat::FTK)
            .item("Encase2", ewfargs::EwfFormat::Encase2)
            .item("Encase3", ewfargs::EwfFormat::Encase3)
            .item("Encase4", ewfargs::EwfFormat::Encase4)
            .item("Encase5", ewfargs::EwfFormat::Encase5)
            .item("Encase6", ewfargs::EwfFormat::Encase6)
            .item("Encase7", ewfargs::EwfFormat::Encase7)
            .item("Linen5", ewfargs::EwfFormat::Linen5)
            .item("Linen6", ewfargs::EwfFormat::Linen6)
            .item("Linen7", ewfargs::EwfFormat::Linen7)
            .item("EwfX", ewfargs::EwfFormat::EwfX)
            .selected(args.ewf_format as usize)
            .with_id("EwfFormat");

        let mut sha1_box = Checkbox::new();
        let mut sha256_box = Checkbox::new();

        // Check the boxes for any digests that are already specified in args
        if (args.digest_type & (ewfargs::DigestType::SHA1)) == ewfargs::DigestType::SHA1 {
            sha1_box = sha1_box.checked();
        }
        if (args.digest_type & (ewfargs::DigestType::SHA256)) == ewfargs::DigestType::SHA256 {
            sha256_box = sha256_box.checked();
        }

        let hash_boxes = LinearLayout::horizontal()
            .child(Checkbox::new().checked().disabled())
            .child(TextView::new("MD5 (required) "))
            .child(sha1_box.with_id("SHA1"))
            .child(TextView::new("SHA1   "))
            .child(sha256_box.with_id("SHA256"))
            .child(TextView::new("SHA256"));

        let compression_select = SelectView::<ewfargs::CompressionType>::new()
            .popup()
            .item("None", ewfargs::CompressionType::None)
            .item("Empty Block", ewfargs::CompressionType::EmptyBlock)
            .item("Fast", ewfargs::CompressionType::Fast)
            .item("Best", ewfargs::CompressionType::Best)
            .selected(args.compression_type as usize)
            .with_id("Compression type");

        ListView::new()
            .child(
                "Filename (no extension)",
                new_entry_box("Filename", 255, &args.target_filename, |s, _| {
                    target_info_next(s)
                }),
            )
            .child(
                "Target directory",
                new_entry_box("Target directory", 255, &args.target_dir, |s, _| {
                    target_info_next(s)
                }),
            )
            .child(
                "Make two copies?",
                Checkbox::new()
                    .on_change(|s, checked| {
                        s.call_on_id("Secondary target directory", |view: &mut EditView| {
                            view.set_enabled(checked)
                        });
                    })
                    .with_id("Two copies"),
            )
            .child(
                "Secondary target directory",
                new_entry_box(
                    "Secondary target directory",
                    255,
                    &args.secondary_target_dir,
                    |s, _| target_info_next(s),
                ),
            )
            .child(
                "Split image into segments?",
                Checkbox::new()
                    .on_change(|s, checked| {
                        s.call_on_id("Segment size", |view: &mut EditView| {
                            view.set_enabled(checked)
                        });
                    })
                    .with_id("Segment"),
            )
            .child(
                "Segment size",
                new_entry_box("Segment size", 255, &args.segment_file_size, |s, _| {
                    target_info_next(s)
                }),
            )
            .child("Target File Format", ewf_select)
            .child("Generate hashes", hash_boxes)
            .child("Compression level", compression_select)
    });
    s.add_layer(
        Dialog::around(fields)
            .button("Back", examiner_info)
            .button("Next", target_info_next)
            .title("Target information"),
    );

    // Set the checkbox for two copies or disable the secondary entry based on the current value
    if two_copies {
        s.call_on_id("Two copies", |view: &mut Checkbox| view.check());
    } else {
        s.call_on_id("Secondary target directory", |view: &mut EditView| {
            view.set_enabled(false)
        });
    }

    // Likewise for segment size
    if segment {
        s.call_on_id("Segment", |view: &mut Checkbox| view.check());
    } else {
        s.call_on_id("Segment size", |view: &mut EditView| {
            view.set_enabled(false)
        });
    }
}

fn technical_options_next(s: &mut Cursive) {
    let success = ARGS.with(|args| -> bool {
        let mut args = args.borrow_mut();

        args.bytes_per_sector =
            match s.call_on_id("Bytes per sector", |view: &mut EditView| -> Option<i32> {
                let v = view.get_content();
                if v.is_empty() {
                    None
                } else {
                    match v.parse::<i32>() {
                        Err(_) => None,
                        Ok(val) => Some(val),
                    }
                }
            }) {
                None => panic!("Can't find element with ID Bytes per sector"),
                Some(opt) if opt.is_some() => Some(opt.unwrap()),
                Some(_) => {
                    s.add_layer(Dialog::info("Invalid value for Bytes per sector"));
                    return false;
                }
            };

        // Extract num_sectors
        args.num_sectors = s
            .call_on_id(
                "Sectors to read at once",
                |value: &mut SelectView<ewfargs::NumSectors>| -> ewfargs::NumSectors {
                    *(value.selection().unwrap()).clone()
                },
            )
            .unwrap();

        true
    });

    if success {
        s.add_layer(Dialog::info("We did it!"));
    }
}

/// Display the form for entering technical options. These will likely be left at the default,
/// but the user should still be able to change them.
///
/// # Arguments
///
/// * `s` - A mutable reference to the `Cursive` instance to display on.
///
/// # Buttons
///
/// * "Next" - Currently not implemented.
/// * "Back" - Return to the target information form.
fn technical_options(s: &mut Cursive) {
    s.pop_layer();

    let fields = ARGS.with(|args| {
        let args = args.borrow();

        let num_sectors_select = SelectView::<ewfargs::NumSectors>::new()
            .popup()
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
            .selected(ewfargs::NumSectors::default() as usize);

        let bytes_per_sector = match args.bytes_per_sector {
            Some(v) => Some(v.to_string()),
            None => None,
        };

        ListView::new()
            .child(
                "Bytes per sector",
                new_entry_box("Bytes per sector", 32, &bytes_per_sector, |s, _| {
                    technical_options_next(s)
                }),
            )
            .child("Sectors to read at once", num_sectors_select)
    });

    s.add_layer(
        Dialog::around(fields)
            .title("Technical options")
            .button("Back", target_info)
            .button("Next", |_| ()),
    );
}
