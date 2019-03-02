use cursive::views::{Dialog, SelectView};
use cursive::Cursive;

mod windows;

fn main() {
    let mut siv = Cursive::default();

    welcome(&mut siv);

    siv.run();
}

pub fn welcome(s: &mut Cursive) {
    s.add_layer(
        Dialog::text(
            "     Visit our Github at github.com/demize/grim.\n\
             Comments can be addressed to demize@unstable.systems.",
        )
        .title(format!("grim {}", env!("CARGO_PKG_VERSION")))
        .button("Continue", main_menu),
    );
}

pub fn main_menu(s: &mut Cursive) {
    enum MenuOptions {
        Image,
        Settings,
        Exit,
    }
    fn on_submit(s: &mut Cursive, selection: &MenuOptions) {
        match selection {
            MenuOptions::Image => windows::examiner_info(s),
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
