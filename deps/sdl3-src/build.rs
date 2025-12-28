fn main() {
    println!("cargo::rerun-if-changed=src/");
    println!("cargo::rerun-if-changed=CMakeLists.txt");
}
