use json;
use std::io;
use std::process::Command;

/// Hold information about a hard drive obtained from lshw.
pub struct HdInfo {
    /// The product string of the hard drive, usually the human-readable product name.
    pub product: String,
    /// The "filesystem" name of the hard drive.
    pub logical_name: String,
    /// The size of the hard drive, hopefully in bytes.
    pub size: f64,
    /// The actual unit used for `size`. At the time of writing,
    /// lshw will always use bytes, however this is not guaranteed.
    pub units: String,
    /// The serial number of the drive.
    pub serial: String,
}

/// Attempts to get the serial number of the machine running the application.
///
/// # Returns
/// - `Err` if lshw did not run correctly, or
/// - `Ok` with a blank string if lshw's output was valid but did not contain a serial number, or
/// - `Ok` with the parsed serial number
///
/// # Panics
/// - If lshw's output cannot be turned into a string
/// - If lshw's output cannot be parsed as JSON
pub fn get_pc_serial() -> io::Result<String> {
    let output = Command::new("lshw").arg("-quiet").arg("-json").output();
    if let Err(e) = output {
        return Err(e);
    }

    let output = String::from_utf8(output.unwrap().stdout).expect("Could not parse output of lshw");

    let parsed = json::parse(&output).expect("Could not parse output of lshw");

    if parsed["serial"].is_null() || !parsed["serial"].is_string() {
        Ok(String::from(""))
    } else {
        Ok(String::from(parsed["serial"].as_str().unwrap()))
    }
}

/// Get a list of all disks in the machine.
///
/// # Returns
/// - `Err` if lshw did not run correctly
/// - `Ok` with an empty vector if no disks were returned by lshw
/// - `Ok` with a vector containing all the disks lshw returned
///
/// # Panics
/// - If lshw's output cannot be turned into a string
/// - If lshw's output cannot be parsed as JSON
/// - If an element in the parsed output is called "children" but is not an array
pub fn get_all_disks() -> io::Result<Vec<HdInfo>> {
    let output = Command::new("lshw").arg("-quiet").arg("-json").output();
    if let Err(e) = output {
        return Err(e);
    }

    let output = String::from_utf8(output.unwrap().stdout).expect("Could not parse output of lshw");

    let parsed = json::parse(&output).expect("Could not parse output of lshw");
    let mut list = Vec::<HdInfo>::new();

    if parsed["children"].is_null() {
        return Ok(list);
    }

    if let json::JsonValue::Array(ref children) = parsed["children"] {
        parse_children(children, &mut list);
    } else {
        panic!("Could not parse output of lshw: invalid value for children");
    }

    Ok(list)
}

/// Parse a list of children and add any drives found to the list of drives provided. Recursive.
///
/// # Arguments
/// - children: A list of children
/// - list: A mutable reference to the list being compiled
fn parse_children(children: &[json::JsonValue], list: &mut Vec<HdInfo>) {
    for child in children {
        if !child["children"].is_null() {
            if let json::JsonValue::Array(ref grandchildren) = child["children"] {
                parse_children(grandchildren, list);
            } else {
                panic!("Could not parse output of lshw: invalid value for children");
            }
        }

        if child["class"] == "disk" && child["id"] == "disk" {
            let product: String;
            let logical_name: String;
            let size: f64;
            let units: String;
            let serial: String;

            if child["product"].is_null() || !child["product"].is_string() {
                product = String::from("");
            } else {
                product = String::from(child["product"].as_str().unwrap());
            }

            if child["logicalname"].is_null() || !child["logicalname"].is_string() {
                logical_name = String::from("");
            } else {
                logical_name = String::from(child["logicalname"].as_str().unwrap());
            }

            if child["size"].is_null() || !child["size"].is_number() {
                size = 0_f64;
            } else {
                size = child["size"].as_f64().expect("Invalid size from lshw");
            }

            if child["units"].is_null() || !child["units"].is_string() {
                units = String::from("");
            } else {
                units = String::from(child["units"].as_str().unwrap());
            }

            if child["serial"].is_null() || !child["serial"].is_string() {
                serial = String::from("");
            } else {
                serial = String::from(child["serial"].as_str().unwrap());
            }

            list.push(HdInfo {
                product: product.clone(),
                logical_name: logical_name.clone(),
                size,
                units: units.clone(),
                serial: serial.clone(),
            })
        }
    }
}
