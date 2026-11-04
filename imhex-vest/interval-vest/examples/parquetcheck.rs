//! Prints, for each file argument, whether the strict Parquet profile
//! accepts it and, if not, at which stage it is rejected.
//!
//!     cargo run -p interval-vest --example parquetcheck -- [--regions] FILE...
//!
//! With `--regions`, also prints the regions of each file in offset order.
use interval_vest::parquet::Parquet;
use interval_vest::ranked::{check_tiled, resolve, RankedFormatExec};

fn stage(b: &[u8]) -> &'static str {
    let Some(j) = Parquet.root(b) else { return "rejected: shorter than the tail" };
    let Some(rs) = resolve(&Parquet, b, j) else {
        return "rejected: a record, rank or link check";
    };
    if check_tiled(&rs, &Parquet.layout(&rs), b.len()) {
        "accepted"
    } else {
        "rejected: regions do not tile the file"
    }
}

fn print_regions(b: &[u8]) {
    let Some(j) = Parquet.root(b) else { return };
    let Some(rs) = resolve(&Parquet, b, j) else { return };
    let mut spans: Vec<(usize, usize, String)> =
        rs.iter().map(|r| (r.start, r.len, format!("{:?}", r.kind))).collect();
    spans.sort();
    let mut pos = 0;
    for (start, len, kind) in spans {
        if start != pos {
            println!("    {pos}..{start}: not covered");
        }
        println!("    {start}..{}: {kind}", start + len);
        pos = pos.max(start + len);
    }
    if pos != b.len() {
        println!("    {pos}..{}: not covered", b.len());
    }
}

fn main() {
    let mut regions = false;
    for arg in std::env::args().skip(1) {
        if arg == "--regions" {
            regions = true;
            continue;
        }
        match std::fs::read(&arg) {
            Ok(b) => {
                println!("{}\t{arg}", stage(&b));
                if regions {
                    print_regions(&b);
                }
            },
            Err(e) => println!("error: {e}\t{arg}"),
        }
    }
}
