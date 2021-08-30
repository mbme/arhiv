use regex::Regex;
use rs_utils::run_command;

pub fn send_notification(message: &str) {
    run_command("notify-send", vec!["-u", "low", message])
        .expect("must be able to send notification");
}

#[must_use]
pub fn match_str(regex: &Regex, s: &str) -> Option<String> {
    regex.captures(s).map(|captures| {
        captures
            .get(1)
            .expect("group 1 must be present")
            .as_str()
            .to_string()
    })
}
