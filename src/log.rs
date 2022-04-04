enum Color {
    Red, Green, Blue
}

fn color_to_ansi(color: &Color) -> &str {
    match color {
        Color::Red => "\x1b[31m",
        Color::Green => "\x1b[32m",
        Color::Blue => "\x1b[34m",
    }
}

fn print_line(color: &Color, level: &str, message: &str) {
    println!("{}[{}] {}\x1b[0m", color_to_ansi(color), level, message);
}

pub fn info(message: &String) {
    print_line(&Color::Blue, "INFO", message);
}
pub fn error(message: &String) {
    print_line(&Color::Red, "ERR!", message);
}
pub fn done(message: &String) {
    print_line(&Color::Green, "DONE", message);
}
