use cursive::views::Dialog;
use cursive::Cursive;

mod windows;

fn main() {
    let mut siv = Cursive::default();

    welcome(&mut siv);

    siv.run();
}

/// Display the welcome window and continue to the main menu once the user continues
///
/// # Arguments
///
/// * `s` - A mutable reference to the `Cursive` instance to display on.
pub fn welcome(s: &mut Cursive) {
    s.add_layer(
        Dialog::text(
            "     Visit our Github at github.com/demize/grim.\n\
             Comments can be addressed to demize@unstable.systems.",
        )
        .title(format!("grim {}", env!("CARGO_PKG_VERSION")))
        .button("Continue", windows::main_menu),
    );
}
