//! Prints, for each file argument, whether the strict ZIP profile accepts it
//! and, if not, at which stage it is rejected.
//!
//!     cargo run -p interval-vest --example zipcheck -- FILE...
use interval_vest::ranked::{check_tiled, resolve, RankedFormatExec};
use interval_vest::zip::Zip;

fn stage(b: &[u8]) -> &'static str {
    let Some(j) = Zip.root(b) else { return "rejected: no end record ends the file" };
    let Some(rs) = resolve(&Zip, b, j) else { return "rejected: a record, rank or link check" };
    if check_tiled(&rs, &Zip.layout(&rs), b.len()) {
        "accepted"
    } else {
        "rejected: regions do not tile the file"
    }
}

fn main() {
    for path in std::env::args().skip(1) {
        match std::fs::read(&path) {
            Ok(b) => println!("{}\t{path}", stage(&b)),
            Err(e) => println!("error: {e}\t{path}"),
        }
    }
}
