use crate::utils::{run_command, send_notification};

pub struct Backlight;

impl Backlight {
    pub fn inc() {
        run_command(
            "xbacklight",
            vec!["-inc", "10", "-time", "50", "-steps", "1"],
        )
        .expect("must be able to increase backlight");
    }

    pub fn dec() {
        run_command(
            "xbacklight",
            vec!["-dec", "10", "-time", "50", "-steps", "1"],
        )
        .expect("must be able to decrease backlight");
    }

    pub fn print_status(notify: bool) {
        let output =
            run_command("xbacklight", vec!["-getf"]).expect("must be able to get backlight");

        let message = format!("Backlight level: {}%", output);

        println!("{}", &message);

        if notify {
            send_notification(&message);
        }
    }
}
