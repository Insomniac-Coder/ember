---

# Part II — Lexical Structure

## II.1 Source encoding

* `[LEX-1]` Source files are UTF-8. A byte-order mark is accepted and ignored; the formatter removes
  it. An invalid UTF-8 sequence is `E0001`.
* `[LEX-2]` *(changed in 0.9.9)* Line endings are LF or CRLF, both normalised to LF before
  tokenisation. The formatter always writes LF unless the manifest sets `[format] line_endings =
  "crlf"`; output never depends on the host platform.
* `[LEX-3]` The source extension is `.em`.

## II.2 Indentation and line structure

Ember is indentation-sensitive exactly as Python 3 is, with stricter rules:

* `[LEX-4]` Indentation MUST use spaces. A tab at the start of a logical line is `E0002`, with a
  fix-it replacing it by four spaces.
* `[LEX-5]` The lexer emits `NEWLINE`, `INDENT` and `DEDENT` with Python's algorithm: an indentation
  stack starting at `[0]`; a deeper line pushes and emits `INDENT`; a shallower line pops until equal,
  emitting one `DEDENT` per pop; an indentation not on the stack is `E0003 inconsistent dedent`.
* `[LEX-6]` Inside `(`, `[` and `{`, and inside a triple-quoted string, newlines do not end a logical
  line and indentation is not significant.
* `[LEX-6a]` Inside brackets, a lambda's `:` body is a single simple statement ended by the enclosing
  closing bracket or by a `,` at the same bracket depth. `f(fn(x): total += x, 4)` passes two
  arguments. A lambda body needing more than one statement is written as a local function.
* `[LEX-7]` A backslash at the end of a physical line joins it to the next. The formatter never
  emits one.
* `[LEX-8]` Blank lines and comment-only lines do not affect indentation.
* `[LEX-9]` An indented block MUST be introduced by a line ending in `:`. A `:` not followed by an
  `INDENT` is `E0004 expected an indented block`, unless the block is one simple statement on the same
  line (`if x: return`).

## II.3 Comments

```text
# line comment
## doc comment: attaches to the next declaration; Markdown body
# SAFETY: justification for the `unsafe` block that follows ([LEX-23])
#$ test-harness annotation; an ordinary comment to the compiler
#! language "0.9.9"  (only before the imports: a directive, [GRM-37])
```

* `[LEX-10]` *(changed in 0.9.9)* There are no block comments. A line beginning `#!` before the first import or item is a
  directive (`[GRM-37]`), not a comment; anywhere else it is a comment.
* `[LEX-11]` *(changed in 0.9.9)* A `##` comment attaches to the next declaration, ignoring blank
  lines. One not followed by a declaration documents nothing and is discarded in silence. Apart from
  `[LEX-23]`, a comment never affects compilation, including by producing a diagnostic.
* `[LEX-11a]` A `##` comment on a comment-only line is emitted after the `INDENT`/`DEDENT` tokens of
  the next content line, so a doc comment written inside a block attaches inside that block.
* `[LEX-23]` *(new in 0.9.9)* A line comment whose text begins `SAFETY:` or `SAFETY(<category>):`, placed on the line
  immediately before an `unsafe:` block or `unsafe fn`, or at the end of the `unsafe:` line itself,
  is that construct's **safety note**. The lexer records it in the comment side table (§II.7);
  `[UNS-8]` reads it. `<category>` is one of `ffi`, `layout`, `aliasing`, `intrinsic`, `performance`,
  `uninit`.

## II.4 Identifiers and keywords

```text
identifier := XID_Start XID_Continue*          (Unicode; NFC-normalised)
```

* `[LEX-12]` Identifiers are NFC-normalised; two identifiers are the same iff their NFC forms are
  byte-equal.
* `[LEX-13]` A lone `_` is the discard pattern and never names a variable.
* `[LEX-14]` A raw identifier `r#name` uses a keyword as a name (for imported C fields such as
  `type`).

**Keywords** (reserved everywhere; 49):

```text
and        as         break      class      comptime   const      continue
defer      dyn        elif       else       enum       extend     extern
false      fn         for        if         implements import     in
interface  is         let        match      mut        not        open
or         override   owned      pass       pub        ref        return
self       Self       static     struct     super      true       type
unsafe     virtual    void       where      while      with       yield
```

* `[LEX-15]` *(changed in 0.9.9)* The table above is the complete reserved set. **Contextual
  keywords** are keywords only in the stated position and identifiers everywhere else: `abstract`
  before `class`; `from` at the start of an import; `gen` and `once` immediately before `fn`; `some`
  at the start of a type; `c` and `cpp` between `import` or `overlay` and a string; `language` and
  `threads` after `#!`; `overlay` at the start of an item, and `rename` and `hide` inside an overlay
  (`[GRM-35]`). **Reserved for a future version** (lexed as keywords; using one as a name is `E0005`,
  whose message names the reservation): `async`, `await`, `macro`, `union`.
* `[LEX-22]` *(changed in 0.9.9)* Ember has no lifetime syntax and never will. A `'` begins a
  character literal and nothing else; an unterminated one is `E0008`.

## II.5 Literals

```text
int_lit     := dec_lit | hex_lit | oct_lit | bin_lit
dec_lit     := digit ("_"? digit)*
hex_lit     := "0x" "_"? hexdigit ("_"? hexdigit)*
oct_lit     := "0o" "_"? octdigit ("_"? octdigit)*
bin_lit     := "0b" "_"? bindigit ("_"? bindigit)*
int_suffix  := "i8"|"i16"|"i32"|"i64"|"i128"|"u8"|"u16"|"u32"|"u64"|"u128"|"isize"|"usize"
float_lit   := dec_lit "." dec_lit exponent? | dec_lit exponent | dec_lit float_suffix
exponent    := ("e"|"E") ("+"|"-")? dec_lit
float_suffix:= "f16" | "f32" | "f64"
char_lit    := "'" char_body "'"                       -- exactly one Unicode scalar value
string_lit  := '"' string_body* '"'
multiline   := '"""' … '"""'                            -- common leading indentation removed
raw_string  := 'r"' … '"' | 'r#"' … '"#'  (up to 8 #)
bytes_lit   := 'b"' byte_body* '"'                      -- type Span[u8], static region
cstr_lit    := 'c"' byte_body* '"'                      -- type cstr, NUL-terminated, static
fstring     := 'f"' (text | "{" expression ("=")? ("!r")? (":" format_spec)? "}")* '"'
escape      := "\n" | "\r" | "\t" | "\0" | "\\" | "\"" | "\'" | "\x" hex hex | "\u{" hex{1,6} "}"
```

* `[LEX-16]` *(changed in 0.9.9)* An integer literal without a suffix is an **untyped integer**: it
  takes the type its context expects (any integer type, or a float type — `Vec3(1, 2, 3)`), and with
  no context it is `int` (`i64`). A value that does not fit the resulting type is `E2010`.
* `[LEX-17]` *(changed in 0.9.9)* A float literal without a suffix is an **untyped float**: it takes
  the float type its context expects, and with no context it is `float` (`f64`). Literals are the only
  values that convert implicitly (`[TYP-4]`).
* `[LEX-17a]` A float literal that receives `f32` or `f16` and whose decimal value is not the value of
  the nearest representable number to within one unit in the last place produces
  `W2015 literal loses precision at f32`, offering the `f64` spelling.
* `[LEX-24]` *(new in 0.9.9)* A unary minus applied directly to an untyped integer literal forms a negative constant
  of the literal's eventual type. If that type is unsigned the program is rejected with `E2010`
  (`-1` does not fit `u32`), and where the context is an index (`xs[-1]`) with `E2011` whose fix-its
  are `xs.last()` and `xs[xs.len() - 1]` (`[TYP-31]`).
* `[LEX-18]` `1.` followed by an identifier character is a method call on `1`; `1.0` is a float.
* `[LEX-19]` *(changed in 0.9.9)* An f-string `{…}` contains a full expression. `{{` and `}}` are
  literal braces. `{expr=}` prints the expression's source text, `=`, then its value (Python's
  debugging form); `{expr!r}` uses `Debug` instead of `Display`. The format spec follows Python's
  mini-language: `[[fill]align][sign][#][0][width][,|_][.precision][type]`, `align ∈ {<,>,^}`,
  `type ∈ {d,x,X,b,o,e,E,f,F,g,G,%,s,?}` (`?` is `Debug`). A spec that does not apply to the value's
  type is `E2250`, naming both.
* `[LEX-20]` *(changed in 0.9.9)* A string literal has type `str` with the static region. At a site
  expecting `String` it produces a `String` (`[TXT-9]`).
* `[LEX-25]` *(new in 0.9.9)* A raw string contains no escapes; a `r#"…"#` raw string may contain `"`. A multiline
  string removes the longest common run of leading spaces from its lines and drops a first line that
  is empty.

## II.6 Operators and punctuation

```text
Arithmetic:   +  -  *  /  //  %  **
Bitwise:      &  |  ^  ~  <<  >>
Comparison:   ==  !=  <  >  <=  >=  is  is not  in  not in
Logical:      and  or  not
Assignment:   =  +=  -=  *=  /=  //=  %=  **=  &=  |=  ^=  <<=  >>=
Range:        ..  ..=
Access:       .  ?.  [ ]  ( )
Cast:         as  as?  as!
Other:        ,  :  ;  ->  =>  @  ?  { }
```

* `[LEX-21]` *(changed in 0.9.9)* Tokenisation is maximal munch: `//=` before `//` before `/`,
  `**=` before `**`, `..=` before `..`, `?.` is one token. `as?` and `as!` are single tokens when `as`
  is followed immediately (no space) by `?` or `!`; `!` has no other use. There is no `::` token; a
  `::` in source is `E0100` with the help `use '.' for paths` (`[GRM-24]`).

## II.7 Token stream contract

The lexer produces tokens `{kind, span, flags}`. `INDENT`, `DEDENT` and `NEWLINE` are real tokens;
doc comments are tokens; other comments are dropped from the stream and recorded, with their spans, in
a side table used by the formatter and by `[LEX-23]`. The lexer never stops at the first error: an
invalid character produces one `Error` token per maximal run of invalid characters (`[DIA-20]`), and
parsing continues.
