//! Compile the listed C-files

extern crate gcc;

fn main() {
    gcc::Config::new().file("src/client/native/console_raw.c").compile("librawconsole.a");
}
