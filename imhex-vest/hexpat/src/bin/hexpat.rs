//! Command-line front end.
//!
//! ```text
//! hexpat parse <pattern.hexpat> [-I dir]...          check syntax, print statement summary
//! hexpat run <pattern.hexpat> <data> [-I dir]... [--json [-m]] [--tree-json] [--format] [--hidden] [--max-entries N]
//! hexpat format -p <pattern.hexpat> -i <data> [-I dir]... [-f json] [-m] [-o out.json]
//! hexpat emit-vest <pattern.hexpat> [-I dir]... [-o out.vest]
//!
//! `--json` and `format -f json` produce the same JSON as the reference
//! implementation's `plcli format -f json`; `-m` adds its metadata fields.
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use hexpat::interp::dump::DumpOptions;
use hexpat::{Runtime, Value};

fn usage() -> ExitCode {
    eprintln!("usage:\n  hexpat parse <pattern> [-I dir]...\n  hexpat run <pattern> <data> [-I dir]... [--json [-m]] [--tree-json] [--format] [--hidden] [--max-entries N] [--in name=value]...\n  hexpat format -p <pattern> -i <data> [-I dir]... [-f json] [-m] [-o file]\n  hexpat emit-vest <pattern> [-I dir]... [-o file] [--root TYPE]");
    ExitCode::from(2)
}

struct Args {
    positional: Vec<String>,
    includes: Vec<PathBuf>,
    json: bool,
    tree_json: bool,
    meta: bool,
    pattern_opt: Option<String>,
    input_opt: Option<String>,
    formatter: String,
    format: bool,
    hidden: bool,
    max_entries: usize,
    output: Option<PathBuf>,
    root: Option<String>,
    in_vars: Vec<(String, String)>,
    quiet: bool,
}

fn parse_args() -> Option<Args> {
    let mut a = Args {
        positional: Vec::new(),
        includes: Vec::new(),
        json: false,
        tree_json: false,
        meta: false,
        pattern_opt: None,
        input_opt: None,
        formatter: "json".to_string(),
        format: false,
        hidden: false,
        max_entries: 64,
        output: None,
        root: None,
        in_vars: Vec::new(),
        quiet: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-I" | "--include" => a.includes.push(PathBuf::from(it.next()?)),
            "--json" => a.json = true,
            "--tree-json" => a.tree_json = true,
            "-m" | "--meta" | "--metadata" => a.meta = true,
            "-p" | "--pattern" => a.pattern_opt = Some(it.next()?),
            "-i" | "--input" => a.input_opt = Some(it.next()?),
            "-f" | "--formatter" => a.formatter = it.next()?,
            "--format" => a.format = true,
            "--hidden" => a.hidden = true,
            "--quiet" | "-q" => a.quiet = true,
            "--max-entries" => a.max_entries = it.next()?.parse().ok()?,
            "-o" | "--output" => a.output = Some(PathBuf::from(it.next()?)),
            "--root" => a.root = Some(it.next()?),
            "--in" => {
                let kv = it.next()?;
                let (k, v) = kv.split_once('=')?;
                a.in_vars.push((k.to_string(), v.to_string()));
            }
            s if s.starts_with("-I") => a.includes.push(PathBuf::from(&s[2..])),
            _ => a.positional.push(arg),
        }
    }
    Some(a)
}

/// Writes to stdout, ignoring a closed pipe (for example `| head`).
fn emit(text: &str) {
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    let _ = out.write_all(text.as_bytes());
    let _ = out.flush();
}

fn main() -> ExitCode {
    // Pattern evaluation recurses per nesting level; give it a large stack.
    let child = std::thread::Builder::new().stack_size(1 << 30).spawn(real_main).expect("spawn main thread");
    child.join().unwrap_or(ExitCode::from(1))
}

fn real_main() -> ExitCode {
    let Some(args) = parse_args() else { return usage() };
    if args.positional.is_empty() {
        return usage();
    }
    match args.positional[0].as_str() {
        "parse" => cmd_parse(&args),
        "run" => cmd_run(&args),
        "format" => cmd_format(&args),
        "emit-vest" => cmd_emit(&args),
        _ => usage(),
    }
}

fn read_pattern(path: &Path) -> Result<String, ExitCode> {
    std::fs::read_to_string(path).map_err(|e| {
        eprintln!("cannot read {}: {}", path.display(), e);
        ExitCode::from(1)
    })
}

fn cmd_parse(args: &Args) -> ExitCode {
    let Some(path) = args.positional.get(1) else { return usage() };
    let path = Path::new(path);
    let source = match read_pattern(path) {
        Ok(s) => s,
        Err(c) => return c,
    };
    let mut rt = Runtime::new(Vec::new());
    for inc in &args.includes {
        rt.add_include_path(inc);
    }
    match rt.load(&source, path.parent()) {
        Ok(program) => {
            if !args.quiet {
                println!("{}: {} top-level statement(s)", path.display(), program.statements.len());
                for s in &program.statements {
                    println!("  {}", describe(s));
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{}: {}", path.display(), e);
            ExitCode::from(1)
        }
    }
}

fn describe(s: &hexpat::ast::Stmt) -> String {
    use hexpat::ast::Stmt::*;
    match s {
        Import { path, .. } => format!("import {}", path),
        Using { name, .. } | UsingForward { name, .. } => format!("using {}", name),
        Namespace { name, body, .. } => format!("namespace {} ({} statements)", name.join("::"), body.len()),
        Struct(d) => format!("struct {} ({} members)", d.name, d.body.len()),
        Union(d) => format!("union {}", d.name),
        Bitfield(d) => format!("bitfield {}", d.name),
        Enum(e) => format!("enum {} ({} entries)", e.name, e.entries.len()),
        Function(f) => format!("fn {}({} params)", f.name, f.params.len()),
        Var(v) => format!("{} {}{}", v.ty, v.name, if v.placement.is_some() { " @ ..." } else { "" }),
        Array(a) => format!("{} {}[...]", a.ty, a.name),
        Pointer(p) => format!("{} *{}", p.ty, p.name),
        other => format!("{:?}", std::mem::discriminant(other)),
    }
}

fn cmd_run(args: &Args) -> ExitCode {
    let (Some(pattern), Some(data)) = (args.positional.get(1), args.positional.get(2)) else { return usage() };
    let path = Path::new(pattern);
    let source = match read_pattern(path) {
        Ok(s) => s,
        Err(c) => return c,
    };
    let data = match std::fs::read(data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("cannot read {}: {}", data, e);
            return ExitCode::from(1);
        }
    };
    let mut rt = Runtime::new(data);
    for inc in &args.includes {
        rt.add_include_path(inc);
    }
    for (k, v) in &args.in_vars {
        let value = match v.parse::<i128>() {
            Ok(i) => Value::Signed(i),
            Err(_) => match v.as_str() {
                "true" => Value::Bool(true),
                "false" => Value::Bool(false),
                _ => Value::Str(v.clone()),
            },
        };
        rt.set_in_variable(k, value);
    }
    let result = rt.run_source(&source, path.parent());
    let opts = DumpOptions { formatted: args.format, show_hidden: args.hidden, max_entries: args.max_entries };
    for line in &rt.console {
        emit(&format!("[out] {}\n", line));
    }
    for w in &rt.warnings {
        eprintln!("[warn] {}", w);
    }
    if !args.quiet {
        let text = if args.json {
            hexpat::interp::dump_json_imhex(&mut rt, args.meta)
        } else if args.tree_json {
            hexpat::interp::dump_json(&mut rt, &opts)
        } else {
            hexpat::interp::dump_text(&mut rt, &opts)
        };
        emit(&text);
        if args.json || args.tree_json {
            emit("\n");
        }
    }
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::from(1)
        }
    }
}

/// `hexpat format -p PATTERN -i DATA [-f json] [-m] [-o FILE]`, mirroring
/// `plcli format`.
fn cmd_format(args: &Args) -> ExitCode {
    let pattern = args.pattern_opt.clone().or_else(|| args.positional.get(1).cloned());
    let input = args.input_opt.clone().or_else(|| args.positional.get(2).cloned());
    let (Some(pattern), Some(input)) = (pattern, input) else { return usage() };
    if args.formatter != "json" {
        eprintln!("Invalid formatter. Valid formatters are: [json]");
        return ExitCode::from(2);
    }
    let path = Path::new(&pattern);
    let source = match read_pattern(path) {
        Ok(s) => s,
        Err(c) => return c,
    };
    let data = match std::fs::read(&input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open file '{}': {}", input, e);
            return ExitCode::from(1);
        }
    };
    let mut rt = Runtime::new(data);
    for inc in &args.includes {
        rt.add_include_path(inc);
    }
    if let Err(e) = rt.run_source(&source, path.parent()) {
        eprintln!("error: {}", e);
        return ExitCode::from(1);
    }
    let text = hexpat::interp::dump_json_imhex(&mut rt, args.meta);
    match &args.output {
        Some(o) => {
            if let Err(e) = std::fs::write(o, &text) {
                eprintln!("Failed to create output file: {}: {}", o.display(), e);
                return ExitCode::from(1);
            }
        }
        None => emit(&text),
    }
    ExitCode::SUCCESS
}

fn cmd_emit(args: &Args) -> ExitCode {
    let Some(pattern) = args.positional.get(1) else { return usage() };
    let path = Path::new(pattern);
    let source = match read_pattern(path) {
        Ok(s) => s,
        Err(c) => return c,
    };
    let mut rt = Runtime::new(Vec::new());
    for inc in &args.includes {
        rt.add_include_path(inc);
    }
    let program = match rt.load(&source, path.parent()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}: {}", path.display(), e);
            return ExitCode::from(1);
        }
    };
    match hexpat::emit::emit_vest(&program, rt.default_endian, args.root.as_deref()) {
        Ok(out) => {
            match &args.output {
                Some(o) => {
                    if let Err(e) = std::fs::write(o, &out.source) {
                        eprintln!("cannot write {}: {}", o.display(), e);
                        return ExitCode::from(1);
                    }
                }
                None => print!("{}", out.source),
            }
            for note in &out.notes {
                eprintln!("note: {}", note);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{}: {}", path.display(), e);
            ExitCode::from(1)
        }
    }
}
