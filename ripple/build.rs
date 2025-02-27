use cfg_aliases::cfg_aliases;

#[rustversion::nightly]
fn emit_nightly() {
    println!("cargo:rustc-cfg=nightly");
}

#[rustversion::not(nightly)]
fn emit_nightly() {}

fn main() {
    emit_nightly();

    cfg_aliases! {
        debug_log: { all(feature="debug_log", debug_assertions) },
        nightly_optimization: { all(feature="nightly_optimization", nightly) },
        unsafe_optimization: {all(feature="unsafe_optimization", any(not(debug_assertions), feature="force_unsafe_optimization"))},
        nightly_unsafe: {all(nightly_optimization, unsafe_optimization)}
    }
}
