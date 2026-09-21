//! Lowering from the tree-sitter concrete syntax tree to the [`ast`].
//!
//! The grammar is permissive (one statement set for every block kind), so
//! this module is where the tree is checked for syntax errors and where a few
//! constructs are normalised the way the reference parser does it: compound
//! assignments become plain assignments, `a ... b | c` match patterns are
//! split into alternatives, and multi-variable declarations become several
//! declarations.

use tree_sitter::{Node, Parser, Tree};

use crate::ast::*;
use crate::error::{Error, Result};

/// Parses hexpat source text into a tree-sitter [`Tree`].
pub fn parse_tree(source: &str) -> Result<Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_hexpat::LANGUAGE.into())
        .map_err(|e| Error::new(format!("cannot load hexpat grammar: {}", e)))?;
    parser.parse(source, None).ok_or_else(|| Error::new("tree-sitter returned no tree"))
}

/// Parses and lowers a hexpat source file.
pub fn parse_program(source: &str) -> Result<Program> {
    let tree = parse_tree(source)?;
    lower_program(&tree, source)
}

/// Lowers a parsed tree. Fails with the location of the first syntax error.
pub fn lower_program(tree: &Tree, source: &str) -> Result<Program> {
    let root = tree.root_node();
    if root.has_error() {
        if let Some(err) = first_error(root) {
            let p = err.start_position();
            let text = err.utf8_text(source.as_bytes()).unwrap_or("");
            let snippet: String = text.chars().take(40).collect();
            let what = if err.is_missing() {
                format!("missing {}", err.kind())
            } else {
                format!("unexpected \"{}\"", snippet)
            };
            return Err(Error::at(format!("syntax error: {}", what), p.row as u32 + 1, p.column as u32 + 1));
        }
    }
    let mut l = Lowerer { src: source.as_bytes() };
    let mut statements = Vec::new();
    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        l.lower_statement(child, &mut statements)?;
    }
    Ok(Program { statements })
}

fn first_error(node: Node) -> Option<Node> {
    if node.is_error() || node.is_missing() {
        return Some(node);
    }
    if !node.has_error() {
        return None;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(e) = first_error(child) {
            return Some(e);
        }
    }
    None
}

struct Lowerer<'s> {
    src: &'s [u8],
}

fn line_of(node: Node) -> u32 {
    node.start_position().row as u32 + 1
}

fn syntax_error(node: Node, msg: impl Into<String>) -> Error {
    let p = node.start_position();
    Error::at(msg.into(), p.row as u32 + 1, p.column as u32 + 1)
}

impl<'s> Lowerer<'s> {
    fn text(&self, node: Node) -> String {
        node.utf8_text(self.src).unwrap_or("").to_string()
    }

    fn field<'t>(&self, node: Node<'t>, name: &str) -> Result<Node<'t>> {
        node.child_by_field_name(name)
            .ok_or_else(|| syntax_error(node, format!("{} is missing its `{}`", node.kind(), name)))
    }

    fn named_children<'t>(&self, node: Node<'t>) -> Vec<Node<'t>> {
        let mut cursor = node.walk();
        node.named_children(&mut cursor)
            .filter(|c| c.kind() != "comment" && c.kind() != "preproc_directive")
            .collect()
    }

    fn has_anonymous_child(&self, node: Node, text: &str) -> bool {
        let mut cursor = node.walk();
        let found = node.children(&mut cursor).any(|c| !c.is_named() && c.kind() == text);
        found
    }

    // ------------------------------------------------------------------
    // Statements
    // ------------------------------------------------------------------

    fn lower_block(&mut self, node: Node) -> Result<Vec<Stmt>> {
        let mut out = Vec::new();
        for child in self.named_children(node) {
            self.lower_statement(child, &mut out)?;
        }
        Ok(out)
    }

    /// Lowers a `_body`: either a block or a single statement.
    fn lower_body(&mut self, node: Node) -> Result<Vec<Stmt>> {
        if node.kind() == "block" {
            self.lower_block(node)
        } else {
            let mut out = Vec::new();
            self.lower_statement(node, &mut out)?;
            Ok(out)
        }
    }

    fn lower_statement(&mut self, node: Node, out: &mut Vec<Stmt>) -> Result<()> {
        let line = line_of(node);
        match node.kind() {
            "comment" | "preproc_directive" | "empty_statement" => {}
            "import_statement" => {
                let path_node = self.field(node, "path")?;
                let path = match path_node.kind() {
                    "string_literal" => self.string_value(path_node)?,
                    _ => self
                        .named_children(path_node)
                        .iter()
                        .map(|c| self.text(*c))
                        .collect::<Vec<_>>()
                        .join("/"),
                };
                let alias = node.child_by_field_name("alias").map(|a| self.text(a));
                let all = self.has_anonymous_child(node, "*");
                out.push(Stmt::Import { line, path, alias, all });
            }
            "using_declaration" => {
                let name = self.text(self.field(node, "name")?);
                let template_params = self.lower_template_params(node)?;
                let ty = self.lower_type(self.field(node, "type")?)?;
                let attrs = self.lower_attrs(node)?;
                out.push(Stmt::Using { line, name, template_params, ty, attrs });
            }
            "using_forward_declaration" => {
                let name = self.text(self.field(node, "name")?);
                let template_params = self.lower_template_params(node)?;
                out.push(Stmt::UsingForward { line, name, template_params });
            }
            "namespace_definition" => {
                let name = self.lower_name_path(self.field(node, "name")?);
                let auto = node.child_by_field_name("auto").is_some();
                let mut body = Vec::new();
                for child in self.named_children(node) {
                    if child.kind() == "identifier" || child.kind() == "scoped_identifier" {
                        continue;
                    }
                    self.lower_statement(child, &mut body)?;
                }
                out.push(Stmt::Namespace { line, name, auto, body });
            }
            "struct_definition" | "union_definition" | "bitfield_definition" => {
                let name = self.text(self.field(node, "name")?);
                let template_params = self.lower_template_params(node)?;
                let mut parents = Vec::new();
                let mut cursor = node.walk();
                for p in node.children_by_field_name("parents", &mut cursor) {
                    if !p.is_named() {
                        continue;
                    }
                    parents.push(self.lower_custom_type(p, false, None)?);
                }
                let body = self.lower_block(self.field(node, "body")?)?;
                let attrs = self.lower_attrs(node)?;
                let def = StructDef { line, name, template_params, parents, body, attrs };
                out.push(match node.kind() {
                    "struct_definition" => Stmt::Struct(def),
                    "union_definition" => Stmt::Union(def),
                    _ => Stmt::Bitfield(def),
                });
            }
            "enum_definition" => {
                let name = self.text(self.field(node, "name")?);
                let ty = self.lower_type(self.field(node, "type")?)?;
                let mut entries = Vec::new();
                for child in self.named_children(node) {
                    if child.kind() != "enum_entry" {
                        continue;
                    }
                    let ename = self.text(self.field(child, "name")?);
                    let value = match child.child_by_field_name("value") {
                        Some(v) => Some(self.lower_expr(v)?),
                        None => None,
                    };
                    let end = match child.child_by_field_name("end") {
                        Some(v) => Some(self.lower_expr(v)?),
                        None => None,
                    };
                    entries.push(EnumEntry { name: ename, value, end });
                }
                let attrs = self.lower_attrs(node)?;
                out.push(Stmt::Enum(EnumDef { line, name, ty, entries, attrs }));
            }
            "function_definition" => {
                let name = self.text(self.field(node, "name")?);
                let mut params = Vec::new();
                let mut pack = None;
                for p in self.named_children(self.field(node, "parameters")?) {
                    match p.kind() {
                        "parameter" => {
                            let ty = self.lower_type(self.field(p, "type")?)?;
                            let pname = p
                                .child_by_field_name("name")
                                .map(|n| self.text(n))
                                .unwrap_or_else(|| params.len().to_string());
                            let default = match p.child_by_field_name("default") {
                                Some(d) => Some(self.lower_expr(d)?),
                                None => None,
                            };
                            params.push(Param { ty, name: pname, default });
                        }
                        "parameter_pack" => {
                            pack = Some(self.text(self.field(p, "name")?));
                        }
                        _ => {}
                    }
                }
                let body = self.lower_block(self.field(node, "body")?)?;
                out.push(Stmt::Function(FunctionDef { line, name, params, pack, body }));
            }
            "variable_declaration" => {
                let constant = node.child_by_field_name("const").is_some();
                let ty = self.lower_type(self.field(node, "type")?)?;
                let name = node.child_by_field_name("name").map(|n| self.text(n)).unwrap_or_default();
                let placement = self.lower_placement(node)?;
                let value = match node.child_by_field_name("value") {
                    Some(v) => Some(self.lower_expr(v)?),
                    None => None,
                };
                let in_var = node.child_by_field_name("in").is_some();
                let out_var = node.child_by_field_name("out").is_some();
                let attrs = self.lower_attrs(node)?;
                out.push(Stmt::Var(VarDecl { line, constant, ty, name, placement, value, in_var, out_var, attrs }));
            }
            "multi_variable_declaration" => {
                let constant = node.child_by_field_name("const").is_some();
                let ty = self.lower_type(self.field(node, "type")?)?;
                let attrs = self.lower_attrs(node)?;
                let mut cursor = node.walk();
                for n in node.children_by_field_name("name", &mut cursor) {
                    if !n.is_named() {
                        continue;
                    }
                    out.push(Stmt::Var(VarDecl {
                        line,
                        constant,
                        ty: ty.clone(),
                        name: self.text(n),
                        placement: None,
                        value: None,
                        in_var: false,
                        out_var: false,
                        attrs: attrs.clone(),
                    }));
                }
            }
            "array_declaration" => {
                let constant = node.child_by_field_name("const").is_some();
                let ty = self.lower_type(self.field(node, "type")?)?;
                let name = self.text(self.field(node, "name")?);
                let size = self.lower_array_size(node)?;
                let placement = self.lower_placement(node)?;
                let init = match node.child_by_field_name("value") {
                    Some(list) => {
                        let mut items = Vec::new();
                        for c in self.named_children(list) {
                            items.push(self.lower_expr(c)?);
                        }
                        Some(items)
                    }
                    None => None,
                };
                let attrs = self.lower_attrs(node)?;
                out.push(Stmt::Array(ArrayDecl { line, constant, ty, name, size, placement, init, attrs }));
            }
            "pointer_declaration" | "pointer_array_declaration" => {
                let ty = self.lower_type(self.field(node, "type")?)?;
                let name = self.text(self.field(node, "name")?);
                let array = if node.kind() == "pointer_array_declaration" {
                    Some(self.lower_array_size(node)?)
                } else {
                    None
                };
                let pointer_ty = self.lower_type(self.field(node, "pointer_type")?)?;
                let placement = self.lower_placement(node)?;
                let attrs = self.lower_attrs(node)?;
                out.push(Stmt::Pointer(PointerDecl { line, ty, name, array, pointer_ty, placement, attrs }));
            }
            "padding_declaration" => {
                let size = self.lower_array_size(node)?;
                let attrs = self.lower_attrs(node)?;
                out.push(Stmt::Padding { line, size, attrs });
            }
            "bitfield_field" => {
                let signed = node.child_by_field_name("sign").map(|s| self.text(s) == "signed").unwrap_or(false);
                let name = self.text(self.field(node, "name")?);
                let size = self.lower_expr(self.field(node, "size")?)?;
                let attrs = self.lower_attrs(node)?;
                out.push(Stmt::BitfieldField(BitfieldField { line, name, size, signed, ty: None, attrs }));
            }
            "bitfield_padding" => {
                let size = self.lower_expr(self.field(node, "size")?)?;
                let attrs = self.lower_attrs(node)?;
                out.push(Stmt::BitfieldField(BitfieldField {
                    line,
                    name: "$padding$".to_string(),
                    size,
                    signed: false,
                    ty: None,
                    attrs,
                }));
            }
            "bitfield_sized_field" => {
                let ty = self.lower_type(self.field(node, "type")?)?;
                let name = self.text(self.field(node, "name")?);
                let size = self.lower_expr(self.field(node, "size")?)?;
                let attrs = self.lower_attrs(node)?;
                let signed = matches!(ty.base, TypeBase::Builtin(b) if b.is_signed());
                out.push(Stmt::BitfieldField(BitfieldField { line, name, size, signed, ty: Some(ty), attrs }));
            }
            "assignment_statement" => {
                let target = self.lower_lvalue(self.field(node, "left")?)?;
                let value = self.lower_expr(self.field(node, "right")?)?;
                out.push(Stmt::Assign { line, target, value });
            }
            "compound_assignment_statement" => {
                let left = self.field(node, "left")?;
                let target = self.lower_lvalue(left)?;
                let op_text = self.text(self.field(node, "operator")?);
                let op = BinaryOp::from_str(&op_text[..op_text.len() - 1])
                    .ok_or_else(|| syntax_error(node, "unknown compound operator"))?;
                let rhs = self.lower_expr(self.field(node, "right")?)?;
                let current = self.lower_expr(left)?;
                let value = Expr::Binary { op, left: Box::new(current), right: Box::new(rhs) };
                out.push(Stmt::Assign { line, target, value });
            }
            "expression_statement" => {
                let child = self
                    .named_children(node)
                    .into_iter()
                    .next()
                    .ok_or_else(|| syntax_error(node, "empty expression statement"))?;
                let expr = self.lower_expr(child)?;
                out.push(Stmt::Expr { line, expr });
            }
            "if_statement" => {
                let cond = self.lower_expr(self.field(node, "condition")?)?;
                let then = self.lower_body(self.field(node, "consequence")?)?;
                let otherwise = match node.child_by_field_name("alternative") {
                    Some(a) => self.lower_body(a)?,
                    None => Vec::new(),
                };
                out.push(Stmt::If { line, cond, then, otherwise });
            }
            "match_statement" => {
                let mut values = Vec::new();
                let mut cursor = node.walk();
                for v in node.children_by_field_name("value", &mut cursor) {
                    if !v.is_named() {
                        continue;
                    }
                    values.push(self.lower_expr(v)?);
                }
                let mut cases = Vec::new();
                for c in self.named_children(node) {
                    if c.kind() != "match_case" {
                        continue;
                    }
                    let mut patterns = Vec::new();
                    for p in self.named_children(c) {
                        if p.kind() == "match_pattern" {
                            patterns.push(self.lower_match_pattern(p)?);
                        }
                    }
                    let body = self.lower_body(self.field(c, "body")?)?;
                    if patterns.len() != values.len() {
                        return Err(syntax_error(
                            c,
                            format!("match case has {} patterns but {} values are matched", patterns.len(), values.len()),
                        ));
                    }
                    cases.push(MatchCase { patterns, body });
                }
                out.push(Stmt::Match { line, values, cases });
            }
            "try_catch_statement" => {
                let body = self.lower_block(self.field(node, "body")?)?;
                let handler = match node.child_by_field_name("handler") {
                    Some(h) => self.lower_block(h)?,
                    None => Vec::new(),
                };
                out.push(Stmt::TryCatch { line, body, handler });
            }
            "while_statement" => {
                let cond = self.lower_expr(self.field(node, "condition")?)?;
                let body = self.lower_body(self.field(node, "body")?)?;
                out.push(Stmt::While { line, cond, body });
            }
            "for_statement" => {
                let init = self.lower_for_clause(self.field(node, "initializer")?)?;
                let cond = self.lower_expr(self.field(node, "condition")?)?;
                let update = self.lower_for_clause(self.field(node, "update")?)?;
                let body = self.lower_body(self.field(node, "body")?)?;
                out.push(Stmt::For { line, init: Box::new(init), cond, update: Box::new(update), body });
            }
            "return_statement" => {
                let value = match self.named_children(node).into_iter().next() {
                    Some(v) => Some(self.lower_expr(v)?),
                    None => None,
                };
                out.push(Stmt::Return { line, value });
            }
            "break_statement" => out.push(Stmt::Break { line }),
            "continue_statement" => out.push(Stmt::Continue { line }),
            "block" => {
                // A bare block is not a hexpat construct, but tolerate it as a
                // sequence of statements (the reference parser would reject it).
                let stmts = self.lower_block(node)?;
                out.extend(stmts);
            }
            other => return Err(syntax_error(node, format!("unexpected node `{}`", other))),
        }
        Ok(())
    }

    fn lower_for_clause(&mut self, node: Node) -> Result<Stmt> {
        let line = line_of(node);
        match node.kind() {
            "for_variable_declaration" => {
                let ty = self.lower_type(self.field(node, "type")?)?;
                let name = self.text(self.field(node, "name")?);
                let value = match node.child_by_field_name("value") {
                    Some(v) => Some(self.lower_expr(v)?),
                    None => None,
                };
                Ok(Stmt::Var(VarDecl {
                    line,
                    constant: false,
                    ty,
                    name,
                    placement: None,
                    value,
                    in_var: false,
                    out_var: false,
                    attrs: Vec::new(),
                }))
            }
            "for_assignment" => {
                let target = self.lower_lvalue(self.field(node, "left")?)?;
                let value = self.lower_expr(self.field(node, "right")?)?;
                Ok(Stmt::Assign { line, target, value })
            }
            "for_compound_assignment" => {
                let left = self.field(node, "left")?;
                let target = self.lower_lvalue(left)?;
                let op_text = self.text(self.field(node, "operator")?);
                let op = BinaryOp::from_str(&op_text[..op_text.len() - 1])
                    .ok_or_else(|| syntax_error(node, "unknown compound operator"))?;
                let rhs = self.lower_expr(self.field(node, "right")?)?;
                let current = self.lower_expr(left)?;
                let value = Expr::Binary { op, left: Box::new(current), right: Box::new(rhs) };
                Ok(Stmt::Assign { line, target, value })
            }
            _ => Ok(Stmt::Expr { line, expr: self.lower_expr(node)? }),
        }
    }

    fn lower_match_pattern(&mut self, node: Node) -> Result<CasePattern> {
        let children = self.named_children(node);
        if children.len() == 1 && children[0].kind() == "wildcard" {
            return Ok(CasePattern::Wildcard);
        }
        // The grammar yields `e0 ... e1 ... e2`; each expression may be a
        // `|` chain that separates alternatives. Flatten into a token list.
        #[derive(Clone)]
        enum Tok {
            E(Expr),
            Range,
            Or,
        }
        let mut toks: Vec<Tok> = Vec::new();
        for (i, c) in children.iter().enumerate() {
            if i > 0 {
                toks.push(Tok::Range);
            }
            let e = self.lower_expr(*c)?;
            let mut parts = Vec::new();
            flatten_or(e, &mut parts);
            for (j, p) in parts.into_iter().enumerate() {
                if j > 0 {
                    toks.push(Tok::Or);
                }
                toks.push(Tok::E(p));
            }
        }
        let mut alts: Vec<(Expr, Option<Expr>)> = Vec::new();
        let mut i = 0;
        while i < toks.len() {
            let start = match &toks[i] {
                Tok::E(e) => e.clone(),
                _ => return Err(syntax_error(node, "malformed match pattern")),
            };
            i += 1;
            let mut end = None;
            if i < toks.len() && matches!(toks[i], Tok::Range) {
                i += 1;
                end = match toks.get(i) {
                    Some(Tok::E(e)) => Some(e.clone()),
                    _ => return Err(syntax_error(node, "malformed match range")),
                };
                i += 1;
            }
            alts.push((start, end));
            if i < toks.len() {
                if !matches!(toks[i], Tok::Or) {
                    return Err(syntax_error(node, "malformed match pattern"));
                }
                i += 1;
            }
        }
        Ok(CasePattern::Alternatives(alts))
    }

    fn lower_lvalue(&mut self, node: Node) -> Result<LValue> {
        Ok(match node.kind() {
            "identifier" => LValue::Ident(self.text(node)),
            "dollar" => LValue::Dollar,
            _ => LValue::Path(self.lower_expr(node)?),
        })
    }

    fn lower_placement(&mut self, node: Node) -> Result<Option<Placement>> {
        let Some(p) = node.child_by_field_name("placement") else { return Ok(None) };
        let address = self.lower_expr(self.field(p, "address")?)?;
        let section = match p.child_by_field_name("section") {
            Some(s) => Some(self.lower_expr(s)?),
            None => None,
        };
        Ok(Some(Placement { address, section }))
    }

    fn lower_array_size(&mut self, node: Node) -> Result<ArraySize> {
        let Some(size) = node.child_by_field_name("size") else { return Ok(ArraySize::Unsized) };
        if size.kind() == "while_size" {
            let cond = self.lower_expr(self.field(size, "condition")?)?;
            Ok(ArraySize::While(cond))
        } else {
            Ok(ArraySize::Fixed(self.lower_expr(size)?))
        }
    }

    fn lower_attrs(&mut self, node: Node) -> Result<Vec<Attribute>> {
        let Some(list) = node.child_by_field_name("attributes") else { return Ok(Vec::new()) };
        let mut attrs = Vec::new();
        for a in self.named_children(list) {
            if a.kind() != "attribute" {
                continue;
            }
            let name = self.lower_name_path(self.field(a, "name")?);
            let mut args = Vec::new();
            if let Some(arguments) = a.child_by_field_name("arguments") {
                for arg in self.named_children(arguments) {
                    args.push(self.lower_expr(arg)?);
                }
            }
            attrs.push(Attribute { name, args });
        }
        Ok(attrs)
    }

    fn lower_template_params(&mut self, node: Node) -> Result<Vec<TemplateParam>> {
        let Some(list) = node.child_by_field_name("template_parameters") else { return Ok(Vec::new()) };
        let mut params = Vec::new();
        for p in self.named_children(list) {
            if let Some(n) = p.child_by_field_name("type_name") {
                params.push(TemplateParam { name: self.text(n), is_type: true });
            } else if let Some(n) = p.child_by_field_name("value_name") {
                params.push(TemplateParam { name: self.text(n), is_type: false });
            }
        }
        Ok(params)
    }

    // ------------------------------------------------------------------
    // Types
    // ------------------------------------------------------------------

    fn lower_name_path(&self, node: Node) -> Vec<String> {
        match node.kind() {
            "scoped_identifier" => self.named_children(node).iter().map(|c| self.text(*c)).collect(),
            _ => vec![self.text(node)],
        }
    }

    fn lower_type(&mut self, node: Node) -> Result<TypeRef> {
        let reference = node.child_by_field_name("ref").is_some();
        let endian = node.child_by_field_name("endian").map(|e| match self.text(e).as_str() {
            "be" => Endian::Big,
            _ => Endian::Little,
        });
        let base = self.field(node, "base")?;
        match base.kind() {
            "builtin_type" => {
                let name = self.text(base);
                let b = BuiltinType::from_name(&name)
                    .ok_or_else(|| syntax_error(base, format!("unknown builtin type {}", name)))?;
                Ok(TypeRef { reference, endian, base: TypeBase::Builtin(b) })
            }
            _ => self.lower_custom_type(base, reference, endian),
        }
    }

    fn lower_custom_type(&mut self, node: Node, reference: bool, endian: Option<Endian>) -> Result<TypeRef> {
        let name = self.lower_name_path(self.field(node, "name")?);
        let mut args = Vec::new();
        if let Some(list) = node.child_by_field_name("template_arguments") {
            for a in self.named_children(list) {
                if a.kind() == "type" {
                    args.push(TemplateArg::Type(self.lower_type(a)?));
                } else {
                    args.push(TemplateArg::Expr(self.lower_expr(a)?));
                }
            }
        }
        Ok(TypeRef { reference, endian, base: TypeBase::Custom { name, args } })
    }

    // ------------------------------------------------------------------
    // Expressions
    // ------------------------------------------------------------------

    fn lower_expr(&mut self, node: Node) -> Result<Expr> {
        Ok(match node.kind() {
            "number_literal" => Expr::Literal(parse_number(&self.text(node)).map_err(|m| syntax_error(node, m))?),
            "char_literal" => {
                let bytes = unescape(&self.text(node)[1..self.text(node).len() - 1]).map_err(|m| syntax_error(node, m))?;
                Expr::Literal(Literal::Char(bytes.first().copied().unwrap_or(0)))
            }
            "string_literal" => Expr::Literal(Literal::String(self.string_value(node)?)),
            "boolean_literal" => Expr::Literal(Literal::Bool(self.text(node) == "true")),
            "dollar" => Expr::Dollar,
            "null" => Expr::Null,
            "parent" => Expr::Parent,
            "this" => Expr::This,
            "identifier" => Expr::Ident(self.text(node)),
            "scoped_identifier" => Expr::Scoped(self.lower_name_path(node)),
            "parenthesized_expression" => {
                let inner = self
                    .named_children(node)
                    .into_iter()
                    .next()
                    .ok_or_else(|| syntax_error(node, "empty parentheses"))?;
                self.lower_expr(inner)?
            }
            "ternary_expression" => Expr::Ternary {
                cond: Box::new(self.lower_expr(self.field(node, "condition")?)?),
                then: Box::new(self.lower_expr(self.field(node, "consequence")?)?),
                otherwise: Box::new(self.lower_expr(self.field(node, "alternative")?)?),
            },
            "binary_expression" => {
                let op_text = self.text(self.field(node, "operator")?);
                let op = BinaryOp::from_str(&op_text)
                    .ok_or_else(|| syntax_error(node, format!("unknown operator {}", op_text)))?;
                Expr::Binary {
                    op,
                    left: Box::new(self.lower_expr(self.field(node, "left")?)?),
                    right: Box::new(self.lower_expr(self.field(node, "right")?)?),
                }
            }
            "unary_expression" => {
                let op = match self.text(self.field(node, "operator")?).as_str() {
                    "+" => UnaryOp::Plus,
                    "-" => UnaryOp::Neg,
                    "!" => UnaryOp::Not,
                    _ => UnaryOp::BitNot,
                };
                Expr::Unary { op, operand: Box::new(self.lower_expr(self.field(node, "operand")?)?) }
            }
            "cast_expression" => {
                let endian = node.child_by_field_name("endian").map(|e| match self.text(e).as_str() {
                    "be" => Endian::Big,
                    _ => Endian::Little,
                });
                let tname = self.text(self.field(node, "type")?);
                let b = BuiltinType::from_name(&tname).ok_or_else(|| syntax_error(node, "bad cast type"))?;
                Expr::Cast {
                    ty: TypeRef { reference: false, endian, base: TypeBase::Builtin(b) },
                    value: Box::new(self.lower_expr(self.field(node, "value")?)?),
                }
            }
            "reinterpret_expression" => Expr::Reinterpret {
                value: Box::new(self.lower_expr(self.field(node, "value")?)?),
                ty: self.lower_type(self.field(node, "type")?)?,
            },
            "call_expression" => {
                let name = self.lower_name_path(self.field(node, "function")?);
                let mut args = Vec::new();
                for a in self.named_children(self.field(node, "arguments")?) {
                    args.push(self.lower_expr(a)?);
                }
                Expr::Call { name, args, line: line_of(node) }
            }
            "member_expression" => Expr::Member {
                object: Box::new(self.lower_expr(self.field(node, "object")?)?),
                member: self.text(self.field(node, "member")?),
            },
            "index_expression" => Expr::Index {
                object: Box::new(self.lower_expr(self.field(node, "object")?)?),
                index: Box::new(self.lower_expr(self.field(node, "index")?)?),
            },
            "type_operator_expression" => {
                let op = self.text(self.field(node, "operator")?);
                let arg = self.field(node, "argument")?;
                let arg = if arg.kind() == "type" {
                    TypeOrExpr::Type(self.lower_type(arg)?)
                } else {
                    TypeOrExpr::Expr(Box::new(self.lower_expr(arg)?))
                };
                match op.as_str() {
                    "sizeof" => Expr::SizeOf(arg),
                    "addressof" => Expr::AddressOf(arg),
                    _ => Expr::TypeNameOf(arg),
                }
            }
            "type" => {
                // A bare identifier in an ambiguous position (template
                // argument) lands here; treat it as a name reference.
                let t = self.lower_type(node)?;
                match t.base {
                    TypeBase::Custom { name, args } if args.is_empty() && !t.reference && t.endian.is_none() => {
                        if name.len() == 1 {
                            Expr::Ident(name.into_iter().next().unwrap())
                        } else {
                            Expr::Scoped(name)
                        }
                    }
                    _ => return Err(syntax_error(node, "expected an expression, found a type")),
                }
            }
            other => return Err(syntax_error(node, format!("unexpected expression node `{}`", other))),
        })
    }

    fn string_value(&self, node: Node) -> Result<String> {
        let t = self.text(node);
        let inner = &t[1..t.len() - 1];
        let bytes = unescape(inner).map_err(|m| syntax_error(node, m))?;
        Ok(bytes_to_string(&bytes))
    }
}

/// Splits a left-nested `|` chain into its operands, in source order.
fn flatten_or(e: Expr, out: &mut Vec<Expr>) {
    match e {
        Expr::Binary { op: BinaryOp::BitOr, left, right } => {
            flatten_or(*left, out);
            flatten_or(*right, out);
        }
        other => out.push(other),
    }
}

/// Converts raw bytes to a `String`, mapping non-UTF-8 bytes to U+00XX so
/// that byte-exact comparisons still work.
pub fn bytes_to_string(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    }
}

/// Decodes hexpat escape sequences into bytes.
pub fn unescape(s: &str) -> std::result::Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            let mut buf = [0u8; 4];
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            continue;
        }
        let Some(e) = chars.next() else { return Err("dangling backslash".into()) };
        match e {
            'n' => out.push(b'\n'),
            'r' => out.push(b'\r'),
            't' => out.push(b'\t'),
            'a' => out.push(7),
            'b' => out.push(8),
            'f' => out.push(12),
            'v' => out.push(11),
            '0' => out.push(0),
            '\\' => out.push(b'\\'),
            '"' => out.push(b'"'),
            '\'' => out.push(b'\''),
            'x' => {
                let hex: String = (0..2).filter_map(|_| chars.next()).collect();
                let v = u8::from_str_radix(&hex, 16).map_err(|_| format!("bad \\x escape \\x{}", hex))?;
                out.push(v);
            }
            'u' | 'U' => {
                let n = if e == 'u' { 4 } else { 8 };
                let hex: String = (0..n).filter_map(|_| chars.next()).collect();
                let v = u32::from_str_radix(&hex, 16).map_err(|_| format!("bad unicode escape {}", hex))?;
                let ch = char::from_u32(v).ok_or_else(|| format!("invalid code point {:x}", v))?;
                let mut buf = [0u8; 4];
                out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
            }
            other => {
                out.push(b'\\');
                let mut buf = [0u8; 4];
                out.extend_from_slice(other.encode_utf8(&mut buf).as_bytes());
            }
        }
    }
    Ok(out)
}

/// Parses a hexpat numeric literal (`0x1F`, `0b101`, `0o17`, `12'345`, `1.5f`, `3u`).
pub fn parse_number(text: &str) -> std::result::Result<Literal, String> {
    let cleaned: String = text.chars().filter(|c| *c != '\'').collect();
    let lower = cleaned.to_ascii_lowercase();
    let is_hex = lower.starts_with("0x");
    let float_suffix = !is_hex && (lower.ends_with('f') || lower.ends_with('d'));
    let is_float = cleaned.contains('.') || float_suffix || (!is_hex && lower.contains('e'));
    if is_float {
        let mut body = cleaned.as_str();
        if float_suffix {
            body = &body[..body.len() - 1];
        }
        let v: f64 = body.parse().map_err(|_| format!("invalid float literal {}", text))?;
        return Ok(Literal::Float(v));
    }
    let unsigned = lower.ends_with('u');
    let body = if unsigned { &cleaned[..cleaned.len() - 1] } else { cleaned.as_str() };
    let (digits, radix) = if let Some(rest) = body.strip_prefix("0x").or_else(|| body.strip_prefix("0X")) {
        (rest, 16)
    } else if let Some(rest) = body.strip_prefix("0b").or_else(|| body.strip_prefix("0B")) {
        (rest, 2)
    } else if let Some(rest) = body.strip_prefix("0o").or_else(|| body.strip_prefix("0O")) {
        (rest, 8)
    } else {
        (body, 10)
    };
    let v = u128::from_str_radix(digits, radix).map_err(|_| format!("invalid integer literal {}", text))?;
    if unsigned {
        Ok(Literal::Unsigned(v))
    } else if v <= i128::MAX as u128 {
        Ok(Literal::Signed(v as i128))
    } else {
        Ok(Literal::Unsigned(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        assert_eq!(parse_number("0xFF").unwrap(), Literal::Signed(255));
        assert_eq!(parse_number("0b101").unwrap(), Literal::Signed(5));
        assert_eq!(parse_number("1'000").unwrap(), Literal::Signed(1000));
        assert_eq!(parse_number("7u").unwrap(), Literal::Unsigned(7));
        assert_eq!(parse_number("1.5f").unwrap(), Literal::Float(1.5));
    }

    #[test]
    fn lowers_struct_with_match_ranges() {
        let src = "struct A { u8 x; match (x) { (1 ... 3 | 5): u8 a; (_): u16 b; } };";
        let p = parse_program(src).unwrap();
        let Stmt::Struct(s) = &p.statements[0] else { panic!() };
        let Stmt::Match { cases, .. } = &s.body[1] else { panic!() };
        match &cases[0].patterns[0] {
            CasePattern::Alternatives(alts) => {
                assert_eq!(alts.len(), 2);
                assert!(alts[0].1.is_some());
                assert!(alts[1].1.is_none());
            }
            _ => panic!(),
        }
        assert_eq!(cases[1].patterns[0], CasePattern::Wildcard);
    }

    #[test]
    fn reports_syntax_errors_with_location() {
        let err = parse_program("struct A { u8 ; ; } ; struct {").unwrap_err();
        assert!(err.line.is_some());
    }
}
