//! Regression tests against the ImHex-Patterns corpus (GPL-2.0, not vendored).
//!
//! Run `scripts/fetch-corpus.sh` (or point `IMHEX_PATTERNS` at a checkout)
//! and then:
//!
//! ```text
//! IMHEX_PATTERNS=vendor/ImHex-Patterns cargo test -p hexpat --test corpus -- --nocapture
//! ```
//!
//! Without the corpus the tests are skipped.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use hexpat::Runtime;

fn corpus_dir() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("IMHEX_PATTERNS") {
        let p = PathBuf::from(p);
        if p.join("patterns").is_dir() {
            return Some(p);
        }
    }
    let vendored = Path::new(env!("CARGO_MANIFEST_DIR")).join("../vendor/ImHex-Patterns");
    if vendored.join("patterns").is_dir() {
        return Some(vendored);
    }
    None
}

fn pattern_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for sub in ["patterns", "includes"] {
        let mut stack = vec![dir.join(sub)];
        while let Some(d) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if matches!(p.extension().and_then(|s| s.to_str()), Some("hexpat") | Some("pat")) {
                    files.push(p);
                }
            }
        }
    }
    files.sort();
    files
}

/// Every corpus file must parse with the tree-sitter grammar and lower to an
/// AST, except files that only work after macro expansion of statements.
#[test]
fn grammar_parses_the_corpus() {
    // Lowering recurses per nesting level; long `else if` chains need a
    // larger stack than the default test thread provides.
    std::thread::Builder::new().stack_size(1 << 30).spawn(grammar_parses_the_corpus_impl).unwrap().join().unwrap();
}

fn grammar_parses_the_corpus_impl() {
    let Some(dir) = corpus_dir() else {
        eprintln!("corpus not found; skipping");
        return;
    };
    let files = pattern_files(&dir);
    assert!(files.len() > 200, "expected a full corpus, found {} files", files.len());
    let mut raw_failures = Vec::new();
    let mut lowered_failures = Vec::new();
    for f in &files {
        let src = std::fs::read_to_string(f).unwrap();
        // 1. Raw tree-sitter parse (no preprocessing).
        let tree = hexpat::lower::parse_tree(&src).unwrap();
        if tree.root_node().has_error() {
            raw_failures.push(f.file_name().unwrap().to_string_lossy().to_string());
        }
        // 2. Preprocess (macro expansion, includes) and lower. Patterns in
        // subdirectories use includes relative to paths the reference test
        // runner does not set up either (it only globs patterns/*.hexpat).
        let top_level = f.parent() == Some(dir.join("patterns").as_path()) || f.starts_with(dir.join("includes"));
        if !top_level {
            continue;
        }
        let mut rt = Runtime::new(Vec::new());
        rt.add_include_path(dir.join("includes"));
        if let Err(e) = rt.load(&src, f.parent()) {
            lowered_failures.push(format!("{}: {}", f.file_name().unwrap().to_string_lossy(), e));
        }
    }
    eprintln!("raw parse failures: {:?}", raw_failures);
    eprintln!("lowering failures: {:?}", lowered_failures);
    // q3demo.hexpat uses a macro that expands to statements; it only parses
    // after preprocessing.
    assert!(raw_failures.len() <= 1, "raw parse failures: {:?}", raw_failures);
    assert!(lowered_failures.is_empty(), "lowering failures: {:?}", lowered_failures);
}

/// Evaluates every pattern that has a fixture in tests/patterns/test_data.
#[test]
fn interpreter_runs_the_fixtures() {
    let Some(dir) = corpus_dir() else {
        eprintln!("corpus not found; skipping");
        return;
    };
    let test_data = dir.join("tests/patterns/test_data");
    let mut ok = 0;
    let mut failures = Vec::new();
    let mut total = 0;
    let mut top_level: Vec<PathBuf> = std::fs::read_dir(dir.join("patterns"))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("hexpat"))
        .collect();
    top_level.sort();
    for pat in top_level {
        let name = pat.file_name().unwrap().to_string_lossy().to_string();
        // Fixtures are named `<pattern>.hexpat.<ext>` or live in `<pattern>.hexpat/`.
        let mut fixtures: Vec<PathBuf> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&test_data) {
            for e in rd.flatten() {
                let p = e.path();
                let fname = p.file_name().unwrap().to_string_lossy().to_string();
                if fname == name && p.is_dir() {
                    if let Ok(inner) = std::fs::read_dir(&p) {
                        fixtures.extend(inner.flatten().map(|x| x.path()).filter(|x| x.is_file()));
                    }
                } else if fname.starts_with(&format!("{}.", name)) && p.is_file() {
                    fixtures.push(p);
                }
            }
        }
        fixtures.sort();
        let Some(fixture) = fixtures.first() else { continue };
        total += 1;
        let src = std::fs::read_to_string(&pat).unwrap();
        let data = std::fs::read(fixture).unwrap();
        let start = Instant::now();
        // Run in a thread with a large stack (deeply nested patterns recurse).
        let src2 = src.clone();
        let inc = dir.join("includes");
        let base = pat.parent().map(|p| p.to_path_buf());
        let result = std::thread::Builder::new()
            .stack_size(1 << 30)
            .spawn(move || {
                let mut rt = Runtime::new(data);
                rt.add_include_path(inc);
                rt.run_source(&src2, base.as_deref()).map(|_| rt.patterns.len())
            })
            .unwrap()
            .join()
            .unwrap();
        let elapsed = start.elapsed();
        match result {
            Ok(n) => {
                ok += 1;
                if elapsed > Duration::from_secs(10) {
                    eprintln!("slow: {} took {:?}", name, elapsed);
                }
                let _ = n;
            }
            Err(e) => failures.push(format!("{}: {}", name, e)),
        }
    }
    eprintln!("fixtures: {} ok, {} failed of {}", ok, failures.len(), total);
    for f in &failures {
        eprintln!("  {}", f);
    }
    assert!(total >= 150, "expected fixtures, found {}", total);
    // Current coverage; raise as the interpreter improves.
    assert!(ok * 100 / total >= 85, "only {}/{} fixtures evaluate", ok, total);
}
