use cfg_aliases::cfg_aliases;

#[rustversion::nightly]
fn nightly() {
    cfg_aliases! {
        nightly_optimization: { feature="nightly_optimization" },
    }
}

#[rustversion::not(nightly)]
fn nightly() {
    cfg_aliases! {
        nightly_optimization: {false},
    }
}

fn main() {
    nightly();

    cfg_aliases! {
        debug_log: { all(feature="debug_log", debug_assertions) },
    }
}
