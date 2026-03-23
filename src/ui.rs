use owo_colors::OwoColorize;

pub(crate) fn green_std_out(message: String) {
    println!("{}", message.green());
}

pub(crate) fn yellow_std_out(message: String) {
    println!("{}", message.yellow());
}

pub(crate) fn red_std_err(message: String) {
    eprintln!("{}", message.red());
}
