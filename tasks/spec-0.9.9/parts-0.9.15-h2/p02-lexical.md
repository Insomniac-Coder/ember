---

# Part II — Lexical Structure

## II.1 Source encoding

* `[LEX-1]` Source files are UTF-8. A byte-order mark is accepted and ignored; the formatter removes
  it. An invalid UTF-8 sequence is `E0001`.
* `[LEX-2]` *(changed in 0.9.9)* Line endings are LF or CRLF, both normalised to LF before
  tokenisation. The formatter always writes LF (`[FMT-1]`); output never depends on the host platform
  (ODR-060).
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
* `[LEX-23]` *(new in 0.9.9)* *(changed in 0.9.15)* A line comment whose text begins `SAFETY:` or `SAFETY(<category>):`, placed on the line
  immediately before an `unsafe:` block, an `unsafe` implementation (`unsafe extend`, `unsafe
  extern`, `unsafe overlay`), an `unsafe import c` of C source (`[FFI-51]`) or an `@unsafe(...)` attribute
  (`[FFI-26]`), or at the end of the `unsafe:` line itself, is that construct's **safety
  note**: it says why that code is sound. The lexer records it in the comment side table (§II.7);
  `[UNS-8]` reads it. `<category>` is one of `ffi`, `layout`, `aliasing`, `intrinsic`, `performance`,
  `uninit`. An `unsafe fn` carries none: what its callers must make true is its `@safety`
  (`[UNS-7]`), and a safety note before an `unsafe fn` or at the end of its line is warning `W3013`,
  whose message says to remove it, a safety note going above each `unsafe:` block inside the function.

## II.4 Identifiers and keywords

```text
identifier := XID_Start XID_Continue*          (Unicode; NFC-normalised)
```

* `[LEX-12]` Identifiers are NFC-normalised; two identifiers are the same iff their NFC forms are
  byte-equal.
* `[LEX-13]` A lone `_` is the discard pattern and never names a variable.
* `[LEX-14]` *(changed in 0.9.15)* A raw identifier `r#name` uses a keyword as a name (for imported C fields such as
  `type`).

**Keywords** (reserved everywhere; 52):

```text
and            as             break          class          comptime       const
continue       defer          dyn            elif           else           enum
extern         false          fn             for            if             implements
import         in             interface      is             let            match
mut            not            or             override       owns           pass
pub            ref            return         Self           shares         sharedref
static         struct         super          this           thread_shared  true
type           unsafe         using          virtual        void           weakref
where          while          with           yield
```

* `[LEX-15]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* The table above is the complete reserved set. **Contextual
  keywords** are keywords only in the stated position and identifiers everywhere else: `abstract`
  and `open` before `class` (so `File` can have its `open`, `[STD-22]`, ODR-134); `from` at the start
  of an import, after a function's result type and after a field's type (`[GRM-46]`); `extend` at the start of an item when the type
  it extends follows, directly or after generic parameters (`[GRM-34]`); `gen` and `once` immediately before `fn`; `safe` before `fn` in an extern block; `some`
  at the start of a type; `c` and `cpp` between `import` or `overlay` and a string; `language` and
  `threads` after `#!`; `overlay` at the start of an item, and `rename` and `hide` inside an overlay
  (`[GRM-35]`); `union` at the start of an item followed by a name, where it is reserved for a future
  version (`E0005`), so `Set` can have its `union` method (`[STD-16]`); `copy` directly before `of`,
  and `of` directly after `copy` (`copy of x`, so `mem.copy` and a count's `of` stay names,
  `[GRM-43]`); `exists` between the value and `as` in an `if`, `elif` or `while` header (`[GRM-44]`);
  `cast` at the start of an operand directly before a name or
  `this` (`cast s as Circle`, `[GRM-30]`), so `cast(p)` stays a call; `guarded` and `by` after the type
  of a field or a `static` (`[GRM-46]`). The reserved words `not` and
  `is` are also method names immediately after `fn` in a type's body, an `extend` block or an
  interface, and after `.` and `?.`, so `Not` can have its method `not` (`a.not()` is `~a`, ODR-040)
  and `AnyError` its method `is[E]()` (`[ERR-8]`, ODR-106); a function standing alone, an item or a
  local function, may take neither (`E0005`; `r#is` is its raw spelling). **Reserved for a future version** (lexed as keywords; using one as a
  name is `E0005`, whose message names the reservation): `async`, `await`, `macro`. **Reserved and
  not words of Ember** (lexed as keywords so that no program uses them as names; any use is `E1010`,
  whose message gives the word to write): `owned` (write `owns`: `owns x: T`, `owns this`, `mesh: owns
  Mesh`, `fn() owns x:`, and in a C contract `@ffi(param(widget, owns, one))`, verdict 102) and `self` (inside a method its object is `this`, `[GRM-41]`).
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
cstr_lit    := 'c"' byte_body* '"'                      -- type CStrView, NUL-terminated, static
fstring     := 'f"' (text | "{" expression ("=")? ("!r")? (":" format_spec)? "}")* '"'
escape      := "\n" | "\r" | "\t" | "\0" | "\\" | "\"" | "\'" | "\x" hex hex | "\u{" hex{1,6} "}"
```

* `[LEX-16]` *(changed in 0.9.9)* An integer literal without a suffix is an **untyped integer**: it
  takes the type its context expects (any integer type, or a float type — `Vec3(1, 2, 3)`), and with
  no context it is `int` (`i64`). A value that does not fit the resulting type is `E2010`.
* `[LEX-17]` *(changed in 0.9.9)* A float literal without a suffix is an **untyped float**: it takes
  the float type its context expects, and with no context it is `float` (`f64`). Literals are the only
  values that convert implicitly (`[TYP-4]`).
* `[LEX-17a]` *(changed in 0.9.9)* A float literal that receives `f32` or `f16` and has more significant
  digits than the type keeps (more than 9 for `f32`, 5 for `f16`) produces
  `W2015 literal loses precision at f32`, offering the `f64` spelling. `0.1` at `f32` is not warned: the
  program chose the type, and every decimal literal is rounded to it.
* `[LEX-24]` *(new in 0.9.9)* A unary minus applied directly to an untyped integer literal forms a negative constant
  of the literal's eventual type. If that type is unsigned the program is rejected with `E2010`
  (`-1` does not fit `u32`), and where the context is an index (`xs[-1]`) with `E2011` whose fix-its
  are `xs.last()` and `xs[xs.len() - 1]` (`[TYP-31]`).
* `[LEX-18]` `1.` followed by an identifier character is a method call on `1`; `1.0` is a float.
* `[LEX-19]` *(changed in 0.9.9)* An f-string `{…}` contains a full expression. `{{` and `}}` are
  literal braces. `{expr=}` prints the expression's source text, `=`, then its value (Python's
  debugging form). `{expr}` uses `Display`, or `Debug` when the type has no `Display`, as printing
  does (`[STD-9]`); `{expr!r}` always uses `Debug`. The format spec follows Python's
  mini-language: `[[fill]align][sign][#][0][width][,|_][.precision][type]`, `align ∈ {<,>,^}`,
  `type ∈ {d,x,X,b,o,e,E,f,F,g,G,%,s,?}` (`?` is `Debug`). A spec that does not apply to the value's
  type is `E2250`, naming both.
* `[LEX-20]` *(changed in 0.9.9)* *(changed in 0.9.15)* A string literal has type `StrView` with the static region: a view
  of text written in the program. At a site expecting `String` it produces a `String` (`[TXT-9]`).
* `[LEX-25]` *(new in 0.9.9)* A raw string contains no escapes; a `r#"…"#` raw string may contain `"`. A multiline
  string removes the longest common run of leading spaces from its lines and drops a first line that
  is empty.
* `[LEX-26]` *(new in 0.9.10)* A literal is written as the grammar above gives it: an escape is one of
  `escape`'s forms, a character literal one Unicode scalar value, a raw string's prefix at most eight
  `#` and a `"`, a byte or C string ASCII characters or escapes, and an f-string's `}` doubled outside
  a hole. Anything else is `E0009` (malformed literal). A string left unclosed, or an f-string's `{`,
  is `E0101` (unclosed delimiter or literal); an unterminated character literal is `E0008`
  (`[LEX-22]`).

## II.6 Operators and punctuation

```text
Arithmetic:   +  -  *  /  //  %  **
Bitwise:      &  |  ^  ~  <<  >>
Comparison:   ==  !=  <  >  <=  >=  is  is not  in  not in
Logical:      and  or  not
Assignment:   =  +=  -=  *=  /=  //=  %=  **=  &=  |=  ^=  <<=  >>=
Range:        ..  ..=
Access:       .  ?.  [ ]  ( )
Conversion:   as
Other:        ,  :  ;  ->  =>  @  ?  { }
```

* `[LEX-21]` *(changed in 0.9.9)* *(changed in 0.9.15)* Tokenisation is maximal munch: `//=` before `//` before `/`,
  `**=` before `**`, `..=` before `..`, `?.` is one token. `as?` and `as!` are single tokens when `as`
  is followed immediately (no space) by `?` or `!`, lexed only so that either is refused with
  `E1010`, whose message gives the kind test or the cast to write (`[GRM-30]`); `!` has no other use.
  There is no `::` token; a `::` in source is `E0100` with the help `use '.' for paths` (`[GRM-24]`).

## II.7 Token stream contract

The lexer produces tokens `{kind, span, flags}`. `INDENT`, `DEDENT` and `NEWLINE` are real tokens;
doc comments are tokens; other comments are dropped from the stream and recorded, with their spans, in
a side table used by the formatter and by `[LEX-23]`. The lexer never stops at the first error: an
invalid character produces one `Error` token per maximal run of invalid characters (`[DIA-20]`), and
parsing continues.
