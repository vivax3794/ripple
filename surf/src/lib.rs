#![feature(map_try_insert)]
#![feature(allocator_api)]
pub mod css;
pub mod lsp;

use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
