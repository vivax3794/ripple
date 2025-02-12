use std::path::Path;
use std::{env, fs};

pub fn transform() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("hello.rs");
    fs::write(
        dest_path,
        r#"
        pub fn hello() -> i32 {
            100
        }
        "#,
    )
    .unwrap();
}
