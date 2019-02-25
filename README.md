# grim
`grim` is a TUI for libewf, written in Rust and designed for use as part of a forensic boot CD. Its planned features include:

- Acquisition of a hard drive to E01 or Ex01 format
- Writing a single image to multiple destinations
- Automatic verification of written images
- File-based configuration of common settings, to allow for faster, easier, and less mistake-prone in-field use

grim is a work in progress, and the list of planned features is subject to change.

## Building
To build grim, you will need rust and cargo installed; cargo will manage most of the dependencies itself. The exception is ewfutils (part of libewf), which you will have to have installed and on your PATH in order to run grim (but not to build it).