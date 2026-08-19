//! Desktop entry point. Android enters through the `android_main` symbol that
//! `#[bevy_main]` generates in `lib.rs`, so this exists only for `cargo run`.

fn main() {
    dot_tower::main();
}
