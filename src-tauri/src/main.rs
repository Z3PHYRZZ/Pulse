// language: Rust, file: main.rs, target: Windows 11 x86_64, edition 2021
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    pulse::run();
}
