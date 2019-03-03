# grim
`grim` is a TUI for libewf, written in Rust and designed for use as part of a forensic boot CD. Its planned features include:

- Acquisition of a hard drive to E01 or Ex01 format
- Writing a single image to multiple destinations
- Automatic verification of written images
- File-based configuration of common settings, to allow for faster, easier, and less mistake-prone in-field use

grim is a work in progress, and the list of planned features is subject to change.

## Dependencies

grim is written in Rust, and so most dependencies are managed automatically by Cargo. There are exceptions, however. In order to run grim, you will need:

- libewf, with ewfacquirestream and ewfverify on your PATH
- lshw available on your PATH

## Building

You can build grim with `cargo build` in the root directory. Currently there is no way to package grim for distribution as a binary, but that will eventually be the preferred way to run it. For now, just run it with `cargo run` or manually copy the executable.