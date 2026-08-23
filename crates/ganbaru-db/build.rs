fn main() {
    println!("cargo:rerun-if-changed=../../apps/client/src-tauri/migrations");
}
