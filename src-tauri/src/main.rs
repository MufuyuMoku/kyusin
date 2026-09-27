// Tanpa jendela konsol tambahan di Windows untuk build rilis.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    kyusin_app::run()
}
