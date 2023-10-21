use std::process::Command;

const INPUT_CSS_PATH: &str = "./css/input.css";
const PUBLIC_DIR: &str = "./assets/";

fn main() {
    run_tailwind();
}

fn run_tailwind() {
    let mut command = Command::new("tailwind");

    command
        .args([
            "-i",
            INPUT_CSS_PATH,
            "-o",
            &(PUBLIC_DIR.to_string() + "tailwind.css"),
            "--minify",
        ])
        .spawn()
        .expect("couldn't run tailwind. Please run it manually");
}
