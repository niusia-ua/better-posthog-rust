const COMMANDS: &[&str] = &["capture", "batch"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS).build();
}
