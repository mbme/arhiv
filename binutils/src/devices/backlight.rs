use crate::utils::send_notification;
use rs_utils::run_command;

pub struct Backlight;

const STEP: f32 = 5.0;

impl Backlight {
    pub fn inc() {
        let level = Backlight::get_level();

        Backlight::set_level(level + STEP);
    }

    pub fn dec() {
        let level = Backlight::get_level();

        Backlight::set_level(level - STEP);
    }

    fn set_level(level: f32) {
        let level: f32 = level.max(1.0).min(100.0);

        run_command("light", vec!["-S", &level.to_string()])
            .expect("must be able to set backlight level");
    }

    fn get_level() -> f32 {
        let output = run_command("light", vec!["-G"]).expect("must be able to get backlight level");
        let output = output.trim();

        let level: f32 = output
            .parse()
            .expect("must be able to parse backlight level");

        level.round()
    }

    pub fn print_status(notify: bool) {
        let level = Backlight::get_level();

        let message = format!("Backlight level: {}%", level);

        println!("{}", &message);

        if notify {
            send_notification(&message);
        }
    }
}
