use crate::utils::send_notification;
use rs_utils::run_command;

pub struct Backlight;

impl Backlight {
    pub fn inc() {
        run_command("light", vec!["-A", "10"]).expect("must be able to increase backlight");
    }

    pub fn dec() {
        run_command("light", vec!["-U", "10"]).expect("must be able to decrease backlight");
    }

    pub fn print_status(notify: bool) {
        let output = run_command("light", vec!["-G"]).expect("must be able to get backlight");
        let output = output.trim();

        let message = format!("Backlight level: {}%", output);

        println!("{}", &message);

        if notify {
            send_notification(&message);
        }
    }
}
