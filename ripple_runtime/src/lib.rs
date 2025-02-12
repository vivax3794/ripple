//mod hello {
//    include!(concat!(env!("OUT_DIR"), "/hello.rs"));
//}

#[macro_export]
macro_rules! component {
    ($module:ident) => {
        mod $module {
            include!(concat!(env!("OUT_DIR"), "/", stringify!($module), ".rs"));
        }
    };
}
