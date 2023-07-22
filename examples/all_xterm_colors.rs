use colorz::{xterm::XtermColor, Colorize};

fn main() {
    for i in 0..=255 {
        let color = XtermColor::from(i);
        println!("{:?}", color.fg(color));
    }
}
