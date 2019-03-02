use cursive::traits::Identifiable;
use cursive::views::{Dialog, EditView, LinearLayout};
use cursive::Cursive;
use std::cell::RefCell;

extern crate grim_rust;
use grim_rust::ewfargs::ArgsList;

thread_local!(static ARGS: RefCell<ArgsList> = RefCell::new(ArgsList::new()));

enum ExtractionError {
    Blank,
}

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

pub fn required_info(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::text("Not implemented!")
            .title("Required Information")
            .button("Back", examiner_info),
    );
}
