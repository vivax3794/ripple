fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    ripple_transpiler::transform();
}
