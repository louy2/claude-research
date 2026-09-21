//! The hexpat preprocessor.
//!
//! Mirrors the reference implementation: `#define` macros are substituted
//! token-wise (identifiers only, never inside strings or comments),
//! `#include` splices files from the include paths, `#ifdef`/`#ifndef`
//! sections are stripped, and `#pragma` values are collected for the caller.
//! Every other line is copied through unchanged so that tree-sitter positions
//! stay meaningful for the main file (included text keeps its own line
//! numbers relative to the included file).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Output of preprocessing a source file.
#[derive(Debug, Default, Clone)]
pub struct Preprocessed {
    /// The expanded source that is handed to the parser.
    pub source: String,
    /// `#pragma name value` pairs in order of appearance (including from includes).
    pub pragmas: Vec<(String, String)>,
    /// Macro definitions in effect at the end of the file.
    pub defines: HashMap<String, String>,
}

/// Preprocessor configuration.
#[derive(Debug, Default, Clone)]
pub struct Preprocessor {
    pub include_paths: Vec<PathBuf>,
    pub defines: HashMap<String, String>,
    included_once: HashSet<PathBuf>,
}

impl Preprocessor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_include_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.include_paths.push(path.into());
        self
    }

    pub fn define(mut self, name: &str, value: &str) -> Self {
        self.defines.insert(name.to_string(), value.to_string());
        self
    }

    /// Preprocesses `source`, resolving `#include` relative to `base_dir` and
    /// the configured include paths.
    pub fn run(&mut self, source: &str, base_dir: Option<&Path>) -> Result<Preprocessed> {
        let mut out = Preprocessed { defines: self.defines.clone(), ..Default::default() };
        let mut text = String::with_capacity(source.len());
        self.process(source, base_dir, &mut out, &mut text, 0)?;
        out.source = text;
        self.defines = out.defines.clone();
        Ok(out)
    }

    fn process(
        &mut self,
        source: &str,
        base_dir: Option<&Path>,
        out: &mut Preprocessed,
        text: &mut String,
        depth: usize,
    ) -> Result<()> {
        if depth > 32 {
            return Err(Error::new("include nesting too deep"));
        }
        let source = source.strip_prefix('\u{FEFF}').unwrap_or(source);
        // Skip-stack for #ifdef / #ifndef: true means the block is emitted.
        let mut cond_stack: Vec<bool> = Vec::new();
        let mut in_block_comment = false;
        for (idx, raw_line) in source.split('\n').enumerate() {
            let line_no = idx as u32 + 1;
            let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
            let trimmed = line.trim_start();
            let emitting = cond_stack.iter().all(|b| *b);
            if !in_block_comment && trimmed.starts_with('#') {
                let (directive, rest) = split_directive(trimmed);
                let rest = rest.trim();
                match directive {
                    "ifdef" | "ifndef" => {
                        let defined = out.defines.contains_key(rest);
                        cond_stack.push(if directive == "ifdef" { defined } else { !defined });
                        text.push('\n');
                        continue;
                    }
                    "endif" => {
                        cond_stack.pop();
                        text.push('\n');
                        continue;
                    }
                    _ => {}
                }
                if !emitting {
                    text.push('\n');
                    continue;
                }
                match directive {
                    "define" => {
                        let mut parts = rest.splitn(2, |c: char| c.is_whitespace());
                        let name = parts.next().unwrap_or("").to_string();
                        let value = parts.next().unwrap_or("").trim().to_string();
                        if name.is_empty() {
                            return Err(Error::at("#define without a name", line_no, 1));
                        }
                        out.defines.insert(name, value);
                    }
                    "undef" => {
                        out.defines.remove(rest);
                    }
                    "pragma" => {
                        let mut parts = rest.splitn(2, |c: char| c.is_whitespace());
                        let name = parts.next().unwrap_or("").to_string();
                        let value = parts.next().unwrap_or("").trim().to_string();
                        if name == "once" {
                            // Handled by the include machinery below.
                        }
                        out.pragmas.push((name, value));
                    }
                    "error" => {
                        return Err(Error::at(format!("#error {}", rest), line_no, 1));
                    }
                    "include" => {
                        let path = rest
                            .strip_prefix('"')
                            .and_then(|s| s.strip_suffix('"'))
                            .or_else(|| rest.strip_prefix('<').and_then(|s| s.strip_suffix('>')))
                            .ok_or_else(|| Error::at("malformed #include", line_no, 1))?;
                        let resolved = self.resolve_include(path, base_dir).ok_or_else(|| {
                            Error::at(format!("cannot find include \"{}\"", path), line_no, 1)
                        })?;
                        let canonical = resolved.canonicalize().unwrap_or(resolved.clone());
                        if self.included_once.contains(&canonical) {
                            text.push('\n');
                            continue;
                        }
                        let contents = std::fs::read_to_string(&resolved).map_err(|e| {
                            Error::at(format!("cannot read include {}: {}", resolved.display(), e), line_no, 1)
                        })?;
                        if contents.contains("#pragma once") {
                            self.included_once.insert(canonical);
                        }
                        let dir = resolved.parent().map(|p| p.to_path_buf());
                        // Keep the include on its own lines so that the main
                        // file's line numbers are only shifted, never merged.
                        text.push('\n');
                        self.process(&contents, dir.as_deref(), out, text, depth + 1)?;
                        text.push('\n');
                        continue;
                    }
                    other => {
                        return Err(Error::at(format!("unknown directive #{}", other), line_no, 1));
                    }
                }
                text.push('\n');
                continue;
            }
            if !emitting {
                text.push('\n');
                continue;
            }
            expand_line(line, &out.defines, &mut in_block_comment, text);
            text.push('\n');
        }
        // Remove the trailing newline added after the final line.
        text.pop();
        Ok(())
    }

    fn resolve_include(&self, path: &str, base_dir: Option<&Path>) -> Option<PathBuf> {
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Some(dir) = base_dir {
            candidates.push(dir.join(path));
        }
        for inc in &self.include_paths {
            candidates.push(inc.join(path));
        }
        candidates.into_iter().find(|p| p.is_file())
    }
}

/// Splits `#name rest` into ("name", "rest"), tolerating `# name`.
fn split_directive(line: &str) -> (&str, &str) {
    let body = line[1..].trim_start();
    let end = body.find(|c: char| !(c.is_alphanumeric() || c == '_')).unwrap_or(body.len());
    (&body[..end], &body[end..])
}

/// Copies one line, substituting macro names that occur as whole identifiers
/// outside of string, character and comment regions.
fn expand_line(line: &str, defines: &HashMap<String, String>, in_block_comment: &mut bool, out: &mut String) {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if *in_block_comment {
            if bytes[i..].starts_with(b"*/") {
                *in_block_comment = false;
                out.push_str("*/");
                i += 2;
            } else {
                let ch = line[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
            continue;
        }
        let c = bytes[i];
        if bytes[i..].starts_with(b"//") {
            out.push_str(&line[i..]);
            return;
        }
        if bytes[i..].starts_with(b"/*") {
            *in_block_comment = true;
            out.push_str("/*");
            i += 2;
            continue;
        }
        if c == b'"' || c == b'\'' {
            let quote = c;
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            let end = i.min(bytes.len());
            out.push_str(&line[start..end]);
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let ident = &line[start..i];
            match defines.get(ident) {
                Some(value) => out.push_str(value),
                None => out.push_str(ident),
            }
            continue;
        }
        if c.is_ascii_digit() {
            // Keep numbers (which may contain letters such as 0xFF) intact.
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'\'' || bytes[i] == b'.') {
                i += 1;
            }
            out.push_str(&line[start..i]);
            continue;
        }
        let ch = line[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_defines_outside_strings() {
        let mut pp = Preprocessor::new();
        let out = pp.run("#define K 4\nu8 a[K]; // K\nchar s[] = \"K\";\n#pragma endian big\n", None).unwrap();
        assert_eq!(out.source, "\nu8 a[4]; // K\nchar s[] = \"K\";\n\n");
        assert_eq!(out.pragmas, vec![("endian".to_string(), "big".to_string())]);
    }

    #[test]
    fn ifdef_strips_blocks() {
        let mut pp = Preprocessor::new().define("X", "");
        let out = pp.run("#ifdef X\na\n#endif\n#ifndef X\nb\n#endif\nc", None).unwrap();
        assert_eq!(out.source, "\na\n\n\n\n\nc");
    }
}
