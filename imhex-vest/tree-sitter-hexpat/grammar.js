/**
 * Tree-sitter grammar for the ImHex pattern language (".hexpat" / ".pat").
 *
 * The rule structure mirrors the reference recursive-descent parser in
 * WerWolv/PatternLanguage (lib/source/pl/core/parser.cpp). Where the reference
 * parser is context-sensitive (for example it only accepts type definitions at
 * the top level) this grammar is deliberately more permissive: a single
 * statement set is shared by every block kind, and the consumer validates the
 * placement of constructs after parsing.
 *
 * Binary operator precedence, from loosest to tightest, follows the reference
 * parser exactly (note that `|`, `^` and `&` bind tighter than comparisons):
 *   ?:  ||  ^^  &&  == !=  < > <= >=  |  ^  &  << >>  + -  * / %  unary
 */

const PREC = {
  ternary: 1,
  bool_or: 2,
  bool_xor: 3,
  bool_and: 4,
  equality: 5,
  relational: 6,
  bit_or: 7,
  bit_xor: 8,
  bit_and: 9,
  shift: 10,
  additive: 11,
  multiplicative: 12,
  unary: 13,
  cast: 14,
  postfix: 15,
};

const BUILTIN_TYPES = [
  'u8', 'u16', 'u24', 'u32', 'u48', 'u64', 'u96', 'u128',
  's8', 's16', 's24', 's32', 's48', 's64', 's96', 's128',
  'float', 'double', 'bool', 'char', 'char16', 'str', 'auto',
];

function commaSep(rule) {
  return optional(commaSep1(rule));
}

function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)));
}

export default grammar({
  name: 'hexpat',

  extras: $ => [
    /\s/,
    /\uFEFF/,
    $.comment,
    $.preproc_directive,
  ],

  word: $ => $.identifier,

  conflicts: $ => [
    // `Foo<...>` as a type application versus `Foo < ...` as a comparison, and
    // a bare identifier as a type name versus a variable reference.
    [$.type, $._primary_expression],
    [$.custom_type, $._primary_expression],
    [$._template_argument, $._expression],
    // `x;` is an anonymous member of type `x` in the reference parser.
    [$.variable_declaration, $._primary_expression],
  ],

  supertypes: $ => [
    $._statement,
    $._expression,
    $._type_definition,
  ],

  rules: {
    source_file: $ => repeat($._statement),

    // ------------------------------------------------------------------
    // Statements
    // ------------------------------------------------------------------

    _statement: $ => choice(
      $.import_statement,
      $.using_declaration,
      $.using_forward_declaration,
      $.namespace_definition,
      $._type_definition,
      $.function_definition,
      $.variable_declaration,
      $.array_declaration,
      $.pointer_declaration,
      $.pointer_array_declaration,
      $.multi_variable_declaration,
      $.padding_declaration,
      $.bitfield_field,
      $.bitfield_padding,
      $.bitfield_sized_field,
      $.assignment_statement,
      $.compound_assignment_statement,
      $.expression_statement,
      $.if_statement,
      $.match_statement,
      $.try_catch_statement,
      $.while_statement,
      $.for_statement,
      $.return_statement,
      $.break_statement,
      $.continue_statement,
      $.empty_statement,
    ),

    empty_statement: _ => ';',

    block: $ => seq('{', repeat($._statement), '}'),

    _body: $ => choice($.block, $._statement),

    // -- preprocessor --------------------------------------------------
    //
    // Directives are line-oriented, so the whole line is one token, and they
    // are extras because the reference implementation strips them before
    // parsing (an attribute list may therefore follow a `#endif`). The
    // consumer splits the token into the directive name and its argument.

    preproc_directive: _ => token(prec(1, /#[A-Za-z_]+[^\n]*/)),

    // -- imports and aliases ----------------------------------------------

    import_statement: $ => seq(
      'import',
      choice(
        seq('*', 'from', field('path', $._import_path), 'as', field('alias', $.identifier)),
        seq(field('path', $._import_path), optional(seq('as', field('alias', $.scoped_identifier)))),
      ),
      ';',
    ),

    _import_path: $ => choice($.string_literal, $.import_path),

    import_path: $ => seq($.identifier, repeat(seq('.', $.identifier))),

    using_declaration: $ => seq(
      'using',
      field('name', $.identifier),
      optional(field('template_parameters', $.template_parameters)),
      '=',
      field('type', $.type),
      optional(field('attributes', $.attribute_list)),
      ';',
    ),

    using_forward_declaration: $ => seq(
      'using',
      field('name', $.identifier),
      optional(field('template_parameters', $.template_parameters)),
      ';',
    ),

    namespace_definition: $ => seq(
      'namespace',
      optional(field('auto', 'auto')),
      field('name', choice($.identifier, $.scoped_identifier)),
      '{',
      repeat($._statement),
      '}',
    ),

    // -- type definitions ---------------------------------------------------

    _type_definition: $ => choice(
      $.struct_definition,
      $.union_definition,
      $.enum_definition,
      $.bitfield_definition,
    ),

    struct_definition: $ => seq(
      'struct',
      field('name', $.identifier),
      optional(field('template_parameters', $.template_parameters)),
      optional(seq(':', field('parents', commaSep1($.custom_type)))),
      field('body', $.block),
      optional(field('attributes', $.attribute_list)),
    ),

    union_definition: $ => seq(
      'union',
      field('name', $.identifier),
      optional(field('template_parameters', $.template_parameters)),
      field('body', $.block),
      optional(field('attributes', $.attribute_list)),
    ),

    enum_definition: $ => seq(
      'enum',
      field('name', $.identifier),
      ':',
      field('type', $.type),
      '{',
      commaSep($.enum_entry),
      optional(','),
      '}',
      optional(field('attributes', $.attribute_list)),
    ),

    enum_entry: $ => seq(
      field('name', $.identifier),
      optional(seq(
        '=',
        field('value', $._expression),
        optional(seq('...', field('end', $._expression))),
      )),
      optional(field('attributes', $.attribute_list)),
    ),

    bitfield_definition: $ => seq(
      'bitfield',
      field('name', $.identifier),
      optional(field('template_parameters', $.template_parameters)),
      field('body', $.block),
      optional(field('attributes', $.attribute_list)),
    ),

    template_parameters: $ => seq('<', commaSep1($.template_parameter), '>'),

    template_parameter: $ => choice(
      field('type_name', $.identifier),
      seq('auto', field('value_name', $.identifier)),
    ),

    // -- functions ----------------------------------------------------------

    function_definition: $ => seq(
      'fn',
      field('name', $.identifier),
      field('parameters', $.parameter_list),
      field('body', $.block),
    ),

    parameter_list: $ => seq('(', commaSep(choice($.parameter, $.parameter_pack)), ')'),

    parameter: $ => seq(
      field('type', $.type),
      optional(field('name', $.identifier)),
      optional(seq('=', field('default', $._expression))),
    ),

    parameter_pack: $ => seq('auto', '...', field('name', $.identifier)),

    // -- declarations -------------------------------------------------------

    _declaration_tail: $ => seq(
      optional(field('attributes', $.attribute_list)),
      ';',
    ),

    placement: $ => seq(
      '@',
      field('address', $._expression),
      optional(seq('in', field('section', $._expression))),
    ),

    variable_declaration: $ => choice(
      seq(
        optional(field('const', 'const')),
        field('type', $.type),
        field('name', $.identifier),
        optional(choice(
          field('placement', $.placement),
          seq('=', field('value', $._expression)),
          seq(field('in', 'in'), optional(seq('=', field('value', $._expression)))),
          field('out', 'out'),
        )),
        $._declaration_tail,
      ),
      // Anonymous member: `u8;`, `std::mem::AlignTo<4>;`, `IFDS @ 0x10;`
      prec.dynamic(1, seq(
        field('type', $.type),
        optional(field('placement', $.placement)),
        $._declaration_tail,
      )),
    ),

    array_declaration: $ => seq(
      optional(field('const', 'const')),
      field('type', $.type),
      field('name', $.identifier),
      '[',
      optional(field('size', $._array_size)),
      ']',
      optional(choice(
        field('placement', $.placement),
        seq('=', field('value', $.initializer_list)),
      )),
      $._declaration_tail,
    ),

    _array_size: $ => choice($.while_size, $._expression),

    while_size: $ => seq('while', '(', field('condition', $._expression), ')'),

    pointer_declaration: $ => seq(
      field('type', $.type),
      '*',
      field('name', $.identifier),
      ':',
      field('pointer_type', $.type),
      optional(field('placement', $.placement)),
      $._declaration_tail,
    ),

    pointer_array_declaration: $ => seq(
      field('type', $.type),
      '*',
      field('name', $.identifier),
      '[',
      optional(field('size', $._array_size)),
      ']',
      ':',
      field('pointer_type', $.type),
      optional(field('placement', $.placement)),
      $._declaration_tail,
    ),

    multi_variable_declaration: $ => seq(
      optional(field('const', 'const')),
      field('type', $.type),
      field('name', $.identifier),
      repeat1(seq(',', field('name', $.identifier))),
      $._declaration_tail,
    ),

    padding_declaration: $ => seq(
      'padding',
      '[',
      field('size', $._array_size),
      ']',
      $._declaration_tail,
    ),

    initializer_list: $ => seq('{', commaSep($._expression), optional(','), '}'),

    // -- bitfield entries ---------------------------------------------------

    bitfield_field: $ => seq(
      optional(field('sign', choice('unsigned', 'signed'))),
      field('name', $.identifier),
      ':',
      field('size', $._expression),
      $._declaration_tail,
    ),

    bitfield_padding: $ => seq('padding', ':', field('size', $._expression), $._declaration_tail),

    bitfield_sized_field: $ => seq(
      field('type', $.type),
      field('name', $.identifier),
      ':',
      field('size', $._expression),
      $._declaration_tail,
    ),

    // -- simple statements --------------------------------------------------

    assignment_statement: $ => seq(
      field('left', $._lvalue),
      '=',
      field('right', $._expression),
      ';',
    ),

    compound_assignment_statement: $ => seq(
      field('left', $._lvalue),
      field('operator', choice('+=', '-=', '*=', '/=', '%=', '|=', '&=', '^=', '<<=', '>>=')),
      field('right', $._expression),
      ';',
    ),

    _lvalue: $ => choice(
      $.identifier,
      $.dollar,
      $.parent,
      $.this,
      $.member_expression,
      $.index_expression,
    ),

    expression_statement: $ => seq($._expression, ';'),

    if_statement: $ => prec.right(seq(
      'if',
      '(',
      field('condition', $._expression),
      ')',
      field('consequence', $._body),
      optional(seq('else', field('alternative', $._body))),
    )),

    match_statement: $ => seq(
      'match',
      '(',
      field('value', commaSep1($._expression)),
      ')',
      '{',
      repeat($.match_case),
      '}',
    ),

    match_case: $ => seq(
      '(',
      commaSep1($.match_pattern),
      ')',
      ':',
      field('body', $._body),
    ),

    // `a ... b | c ... d` is parsed as `a ... (b | c) ... d`; the consumer
    // splits top-level `|` expressions into alternatives, which is exactly
    // what the reference parser produces.
    match_pattern: $ => choice(
      $.wildcard,
      seq($._expression, repeat(seq('...', $._expression))),
    ),

    wildcard: _ => '_',

    try_catch_statement: $ => seq(
      'try',
      field('body', $.block),
      optional(seq('catch', field('handler', $.block))),
    ),

    while_statement: $ => seq(
      'while',
      '(',
      field('condition', $._expression),
      ')',
      field('body', $._body),
    ),

    for_statement: $ => seq(
      'for',
      '(',
      field('initializer', $._for_clause),
      ',',
      field('condition', $._expression),
      ',',
      field('update', $._for_clause),
      ')',
      field('body', $._body),
    ),

    _for_clause: $ => choice(
      $.for_variable_declaration,
      $.for_assignment,
      $.for_compound_assignment,
      $._expression,
    ),

    for_variable_declaration: $ => seq(
      field('type', $.type),
      field('name', $.identifier),
      optional(seq('=', field('value', $._expression))),
    ),

    for_assignment: $ => seq(field('left', $._lvalue), '=', field('right', $._expression)),

    for_compound_assignment: $ => seq(
      field('left', $._lvalue),
      field('operator', choice('+=', '-=', '*=', '/=', '%=', '|=', '&=', '^=', '<<=', '>>=')),
      field('right', $._expression),
    ),

    return_statement: $ => seq('return', optional($._expression), ';'),
    break_statement: _ => seq('break', ';'),
    continue_statement: _ => seq('continue', ';'),

    // ------------------------------------------------------------------
    // Attributes
    // ------------------------------------------------------------------

    // Opened by one token so that `x @ addr [[attr]]` is not an index
    // expression; closed by two tokens because `] ]` occurs in the wild.
    attribute_list: $ => seq('[[', commaSep1($.attribute), ']', ']'),

    attribute: $ => seq(
      field('name', choice($.identifier, $.scoped_identifier)),
      optional(field('arguments', $.arguments)),
    ),

    // ------------------------------------------------------------------
    // Types
    // ------------------------------------------------------------------

    type: $ => seq(
      optional(field('ref', 'ref')),
      optional(field('endian', $.endian)),
      field('base', choice($.builtin_type, $.custom_type)),
    ),

    endian: _ => choice('le', 'be'),

    builtin_type: _ => choice(...BUILTIN_TYPES),

    custom_type: $ => prec.right(seq(
      field('name', choice($.identifier, $.scoped_identifier)),
      optional(field('template_arguments', $.template_arguments)),
    )),

    // The closing bracket gets lexical precedence over `>>` and `>=` so that
    // nested arguments such as `List<Pair<u8, u8>>` close correctly.
    template_arguments: $ => seq(
      '<',
      commaSep1($._template_argument),
      alias(token(prec(1, '>')), '>'),
    ),

    _template_argument: $ => choice(
      prec.dynamic(1, $.type),
      $._expression,
    ),

    // ------------------------------------------------------------------
    // Expressions
    // ------------------------------------------------------------------

    _expression: $ => choice(
      $.ternary_expression,
      $.binary_expression,
      $.unary_expression,
      $.cast_expression,
      $.reinterpret_expression,
      $.call_expression,
      $.member_expression,
      $.index_expression,
      $.type_operator_expression,
      $._primary_expression,
    ),

    _primary_expression: $ => choice(
      $.number_literal,
      $.char_literal,
      $.string_literal,
      $.boolean_literal,
      $.dollar,
      $.null,
      $.parent,
      $.this,
      $.identifier,
      $.scoped_identifier,
      $.parenthesized_expression,
    ),

    parenthesized_expression: $ => seq('(', $._expression, ')'),

    ternary_expression: $ => prec.right(PREC.ternary, seq(
      field('condition', $._expression),
      '?',
      field('consequence', $._expression),
      ':',
      field('alternative', $._expression),
    )),

    binary_expression: $ => {
      const table = [
        ['||', PREC.bool_or],
        ['^^', PREC.bool_xor],
        ['&&', PREC.bool_and],
        ['==', PREC.equality],
        ['!=', PREC.equality],
        ['<', PREC.relational],
        ['>', PREC.relational],
        ['<=', PREC.relational],
        ['>=', PREC.relational],
        ['|', PREC.bit_or],
        ['^', PREC.bit_xor],
        ['&', PREC.bit_and],
        ['<<', PREC.shift],
        ['>>', PREC.shift],
        ['+', PREC.additive],
        ['-', PREC.additive],
        ['*', PREC.multiplicative],
        ['/', PREC.multiplicative],
        ['%', PREC.multiplicative],
      ];
      return choice(...table.map(([op, p]) => prec.left(p, seq(
        field('left', $._expression),
        field('operator', op),
        field('right', $._expression),
      ))));
    },

    unary_expression: $ => prec(PREC.unary, seq(
      field('operator', choice('+', '-', '!', '~')),
      field('operand', $._expression),
    )),

    // `u32(x)`, `be u16(x)`, `str(x)`
    cast_expression: $ => prec(PREC.cast, seq(
      optional(field('endian', $.endian)),
      field('type', $.builtin_type),
      '(',
      field('value', $._expression),
      ')',
    )),

    // `x as Type`
    reinterpret_expression: $ => prec.left(PREC.cast, seq(
      field('value', $._expression),
      'as',
      field('type', $.type),
    )),

    call_expression: $ => prec(PREC.postfix, seq(
      field('function', choice($.identifier, $.scoped_identifier)),
      field('arguments', $.arguments),
    )),

    arguments: $ => seq('(', commaSep($._expression), ')'),

    member_expression: $ => prec.left(PREC.postfix, seq(
      field('object', $._expression),
      '.',
      field('member', choice($.identifier, $.parent)),
    )),

    index_expression: $ => prec.left(PREC.postfix, seq(
      field('object', $._expression),
      '[',
      field('index', $._expression),
      ']',
    )),

    type_operator_expression: $ => seq(
      field('operator', choice('sizeof', 'addressof', 'typenameof')),
      '(',
      field('argument', choice($.type, $._expression)),
      ')',
    ),

    // ------------------------------------------------------------------
    // Terminals
    // ------------------------------------------------------------------

    scoped_identifier: $ => seq($.identifier, repeat1(seq('::', $.identifier))),

    identifier: _ => /[A-Za-z_][A-Za-z0-9_]*/,

    dollar: _ => '$',
    null: _ => 'null',
    parent: _ => 'parent',
    this: _ => 'this',
    boolean_literal: _ => choice('true', 'false'),

    number_literal: _ => token(choice(
      /0[xX][0-9a-fA-F']+[uU]?/,
      /0[bB][01']+[uU]?/,
      /0[oO][0-7']+[uU]?/,
      /[0-9][0-9']*\.[0-9']*([eE][+-]?[0-9]+)?[fFdD]?/,
      /[0-9][0-9']*([eE][+-]?[0-9]+)?[uUfFdD]?/,
    )),

    // `'''` (an unescaped quote) is accepted by the reference lexer.
    char_literal: _ => token(seq(
      "'",
      choice(
        /[^'\\\n]/,
        "'",
        /\\[^xuU\n]/,
        /\\x[0-9a-fA-F]{2}/,
        /\\u[0-9a-fA-F]{4}/,
        /\\U[0-9a-fA-F]{8}/,
      ),
      "'",
    )),

    string_literal: _ => token(seq(
      '"',
      repeat(choice(/[^"\\\n]/, /\\./)),
      '"',
    )),

    // Plain and documentation comments (`///`, `//!`, `/** */`, `/*! */`).
    comment: _ => token(choice(
      seq('//', /[^\n]*/),
      seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/'),
    )),
  },
});
