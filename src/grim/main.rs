use cursive::views::Dialog;
use cursive::Cursive;
use users;

mod windows;

fn main() {
    let mut siv = Cursive::default();
    let effective_uid = users::get_effective_uid();

    if effective_uid != 0 {
        siv.add_layer(
            Dialog::text("WARNING: You are not root!\nThis program may not work as expected.")
                .title("Warning")
                .button("Ok", |s| {
                    s.pop_layer();
                    welcome(s);
                }),
        );
    } else {
        welcome(&mut siv);
    }

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
