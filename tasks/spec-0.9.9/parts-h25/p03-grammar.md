---

# Part III — Grammar

This is the complete grammar. `{x}` is zero or more, `[x]` optional, `|` alternation; terminals are
quoted or UPPERCASE (`IDENT`, `INT`, `FLOAT`, `STRING`, `CHAR`, `NEWLINE`, `INDENT`, `DEDENT`). The
parser is recursive descent with precedence climbing for expressions; the grammar is LL(2) except
where a note says otherwise. **Every construct used anywhere in this specification appears here**;
a construct in an example that this Part does not produce is a defect in the example.

## III.1 Compilation unit and imports

```text
file          := {directive} {NEWLINE} {import_decl} {item}
directive     := "#!" ("language" STRING | "threads" IDENT) NEWLINE
import_decl   := "import" module_path ["as" IDENT] NEWLINE
               | "from" module_path "import" import_list NEWLINE
               | ["pub"] "import" ("c" | "cpp") STRING [ffi_opts] ["as" IDENT] NEWLINE
module_path   := IDENT {"." IDENT}
import_list   := import_item {"," import_item} [","]
               | "(" import_item {"," import_item} [","] ")"
import_item   := IDENT ["as" IDENT] | "*"
ffi_opts      := "with" "(" ffi_opt {"," ffi_opt} [","] ")"
ffi_opt       := IDENT "=" expression
```

* `[GRM-2]` *(changed in 0.9.9)* A file contains imports and items and, in the **entry file** only
  (the file given to `ember run`/`ember build`, or the manifest's `entry`), statements at file scope,
  which form the implicit `main` (`[FN-8]`). A statement at file scope in any other module is `E0100`
  with the help `move it into a function`.
* `[GRM-37]` *(new in 0.9.9)* A **directive** is a line beginning `#!` before the imports.
  `#! language "<version>"` is accepted only for the current version (`[VER-8]`, `E0006`);
  `#! threads main|any|creator` sets the default thread contract of the file's exports (`[FFI-33]`).
  Any other directive is `E0104`. `import cpp` is Annex C.

## III.2 Items

```text
item          := {attribute} [visibility] item_body
attribute     := "@" attr_name ["(" [attr_arg {"," attr_arg} [","]] ")"] NEWLINE
attr_name     := IDENT {"." IDENT}                   -- dotted names are plugin namespaces
attr_arg      := expression | IDENT "=" expression
visibility    := "pub" ["(" ("package" | "read" | "package" "," "read") ")"]
item_body     := fn_decl | struct_decl | class_decl | enum_decl | interface_decl
               | extend_decl | const_decl | static_decl | type_alias | extern_block
               | comptime_block

fn_decl       := fn_header ":" block
               | fn_header NEWLINE                     -- bodiless: interface, extern, abstract only
fn_header     := ["extern" STRING] ["unsafe"] ["virtual" | "override"] ["gen"]
                 "fn" IDENT [generic_params] "(" [param_list] ")" ["->" type] [where_clause]
generic_params:= "[" generic_param {"," generic_param} [","] "]"
generic_param := IDENT [":" bound_list] ["=" type]
               | "const" IDENT ":" type ["=" expression]
bound_list    := type {"+" type}
where_clause  := "where" type ":" bound_list {"," type ":" bound_list}
param_list    := param {"," param} [","]
param         := ["mut" | "owned"] "self"
               | ["mut" | "owned"] IDENT ":" type ["=" expression]

struct_decl   := "struct" IDENT [generic_params] [implements] [where_clause] ":" type_body
class_decl    := ["open" | "abstract"] "class" IDENT [generic_params] ["(" type ")"]
                 [implements] [where_clause] ":" type_body
implements    := "implements" type {"," type}
type_body     := NEWLINE INDENT {member} DEDENT | "pass" NEWLINE
member        := {attribute} [visibility] (field_decl | fn_decl | const_decl | type_alias
               | "pass" NEWLINE)
field_decl    := ["let"] IDENT ":" type ["=" expression] NEWLINE

enum_decl     := "enum" IDENT [generic_params] [implements] [where_clause] ":"
                 NEWLINE INDENT {enum_member} DEDENT
enum_member   := {attribute} (variant | fn_decl | const_decl)
variant       := IDENT ["(" variant_field {"," variant_field} [","] ")"] ["=" expression] NEWLINE
variant_field := {attribute} [IDENT ":"] type

interface_decl:= "interface" IDENT [generic_params] [":" bound_list] [where_clause] ":"
                 NEWLINE INDENT {interface_member} DEDENT
interface_member := {attribute} (fn_decl | assoc_type | const_decl)
assoc_type    := "type" IDENT [":" bound_list] ["=" type] NEWLINE

extend_decl   := "extend" [generic_params] type [implements] [where_clause] ":" type_body
const_decl    := "const" IDENT [":" type] "=" expression NEWLINE
static_decl   := "static" ["mut"] IDENT ":" type "=" expression NEWLINE
type_alias    := "type" IDENT [generic_params] "=" type ["in" expression] NEWLINE
extern_block  := ["unsafe"] "extern" STRING ":" NEWLINE INDENT {extern_item} DEDENT
extern_item   := {attribute} (["safe"] fn_header NEWLINE
               | "static" ["mut"] IDENT ":" type NEWLINE
               | "type" IDENT NEWLINE)
comptime_block:= "comptime" ":" block
```

* `[GRM-1]` A class has at most one base class, written in parentheses: `class Door(Script):`.
* `[GRM-33]` *(new in 0.9.9)* A bodiless `fn_decl` is legal only in an `interface`, an `extern_block`, or as a
  `virtual` method of an `abstract class`. Elsewhere it is `E0110` with the help `add ':' and a body`.
* `[GRM-34]` *(new in 0.9.9)* `extend [T: B] Array[T] implements I:` declares a generic extension; the generic
  parameters after `extend` scope over the whole block. A parameter the target does not name and an
  implemented interface does makes a **blanket implementation**: `extend[K: Eq + Hash, V, Q:
  AsKey[K]] Map[K, V] implements Index[Q]` makes every `Map[K, V]` implement `Index[Q]` for each `Q`
  its bounds admit, each method of the block is generic over `Q`, and each use is checked as a
  written implementation is (ODR-042).
* `[GRM-8d]` A `type_alias` with an `in` clause declares a range type (`[RNG-1]`). The clause is
  legal only on an item-level alias with no generic parameters (`E2213` otherwise).

## III.3 Types

```text
type          := path_type | "ref" ["mut"] type | "*" ["mut"] type | tuple_type | fn_type
               | "dyn" bound_list | "some" bound_list | "[" type ";" expression "]"
               | "Self" | "void"
path_type     := path_segment {"." path_segment}
path_segment  := IDENT [generic_args]
generic_args  := "[" generic_arg {"," generic_arg} [","] "]"
generic_arg   := type | expression | IDENT "=" type          -- const argument; associated-type binding
tuple_type    := "(" ")" | "(" type "," [type {"," type} [","]] ")"
fn_type       := ["once"] ["extern" STRING] "fn" "(" [fn_type_param {"," fn_type_param} [","]] ")"
                 ["->" type]
fn_type_param := ["mut" | "owned"] type
```

* `[GRM-3]` `Array[T]`, `Map[K, V]`, `Option[T]`, `Result[T, E]`, `Span[T]`, `Box[T]`, and the other
  library types are ordinary `path_type`s; the grammar does not special-case them. Generic arguments
  may appear on any path segment: `SoA[Particle].Ref`.
* `[GRM-31]` *(new in 0.9.9)* A `some` type (`[TYP-32]`) is legal only as a function's return type, or nested inside
  one; elsewhere it is `E2260`.

## III.4 Statements

```text
block         := NEWLINE INDENT {statement} DEDENT | simple_stmt NEWLINE
statement     := {attribute} (simple_stmt NEWLINE | compound_stmt)
simple_stmt   := var_decl | assignment | expr_list | "pass"
var_decl      := IDENT ":" type ["=" expr_list]
assignment    := target_list "=" expr_list
               | target augassign expression
target_list   := target {"," target} [","]
target        := IDENT | postfix_expr | "_" | "(" target_list ")"
expr_list     := expression {"," expression} [","]
augassign     := "+=" | "-=" | "*=" | "/=" | "//=" | "%=" | "**=" | "&=" | "|=" | "^="
               | "<<=" | ">>="

compound_stmt := if_stmt | while_stmt | for_stmt | match_stmt | with_stmt | defer_stmt
               | unsafe_stmt | comptime_stmt | labeled_stmt | fn_decl
if_stmt       := "if" condition ":" block {"elif" condition ":" block} ["else" ":" block]
condition     := expression | pattern "=" expression
while_stmt    := "while" condition ":" block ["else" ":" block]
for_stmt      := "for" for_target "in" expression ":" block ["else" ":" block]
for_target    := pattern {"," pattern}
labeled_stmt  := IDENT ":" (while_stmt | for_stmt)
match_stmt    := "match" expression ":" NEWLINE INDENT {match_arm} DEDENT
match_arm     := pattern ["if" expression] ":" block
with_stmt     := "with" with_item {"," with_item} ":" block
with_item     := [IDENT "="] expression
defer_stmt    := "defer" ":" block
unsafe_stmt   := "unsafe" ":" block
comptime_stmt := "comptime" ":" block
```

* `[GRM-4]` *(changed in 0.9.9)* `x = e` where no `x` is in scope declares `x` with the type of `e`;
  where `x` is in scope it assigns. `x: T = e` always declares; a declaration may shadow a name from an
  enclosing block, and redeclaring a name in the same block is `E1020`. A name declared in every arm
  of an exhaustive branch is hoisted to the enclosing block (`[CTL-10]`).
* `[GRM-5]` `a, b = e` destructures a tuple, a struct or a fixed array. The right side is evaluated
  once, into a temporary, before any target is written (`[EXP-2]`).
* `[GRM-29]` *(new in 0.9.9)* An `expr_list` of two or more expressions, or of one expression followed by a comma, is a
  tuple: `a, b = b, a` swaps, and `return x, y` returns a tuple.
* `[GRM-6]` The optional `else` of `while` and `for` runs when the loop ends without `break`.
* `[GRM-7]` `defer` blocks run in reverse order at the exit of the enclosing block, on every path out
  of it, including `return`, `break` and `continue`. A `defer` block may not itself jump out (`E2160`).
* `[GRM-28]` *(new in 0.9.9)* A `fn_decl` inside a block declares a **local function**. It may capture locals of the
  enclosing function exactly as a lambda does (`[CLO-2]`), may call itself recursively, and is in scope
  from its declaration to the end of the block.
* `[GRM-18]` `;` never separates statements. `a = 1; b = 2` is `E0105`, whose help puts each statement
  on its own line. `;` appears only inside `[T; N]` and `[v; N]`.
* `[GRM-19]` *(changed in 0.9.9)* The pattern of a `condition` MUST be refutable. An irrefutable one
  is `E2036`; when it is a bare name already in scope (`if x = 5:`), the primary help is
  `did you mean 'x == 5'?`.
* `[GRM-20]` The attributes before a statement are limited to `@parallel`, `@unroll`, `@simd` and
  `@allow` (`[ATT-2]`).
* `[ATT-3]` A statement attribute attaches to the next compound statement, never to a simple one
  (`E0108`); `@simd`, `@parallel` and `@unroll` require a `for` (`E0108`). Several may precede one
  statement, one per line, in any order.

## III.5 Expressions

Precedence, lowest first. Operators on one row associate left unless noted.

| Level | Operators | Notes |
|---|---|---|
| 1 | `return e`, `break`, `continue`, `yield e` | jump expressions; must be a whole expression statement or a whole arm/branch body (`[GRM-16]`) |
| 2 | `fn(…) => e`, `owned fn …`, `once fn …` | lambda; body extends as far right as possible |
| 3 | `x if c else y` | right-associative |
| 4 | `or` | |
| 5 | `and` | |
| 6 | `not` (prefix) | |
| 7 | `==` `!=` `<` `>` `<=` `>=` · `is` `is not` · `in` `not in` | the six comparisons chain (`[GRM-25]`); `is`, `in` do not |
| 8 | `..` `..=` | either operand may be omitted; non-associative |
| 9 | `\|` | |
| 10 | `^` | |
| 11 | `&` | |
| 12 | `<<` `>>` | |
| 13 | `+` `-` | |
| 14 | `*` `/` `//` `%` | |
| 15 | `as` `as?` `as!` | postfix type conversion |
| 16 | `-` `~` `ref` `ref mut` (prefix) | `-x as u8` is `(-x) as u8`; `ref a.b[i]` borrows `a.b[i]` |
| 17 | `**` | right-associative; binds tighter than a prefix `-` on its left: `-2**2 == -4` |
| 18 | call, index, field, method, `?.`, postfix `?` | |
| 19 | atoms | |

```text
expression    := jump_expr | lambda | ternary
jump_expr     := "return" [expr_list] | "break" [IDENT] | "continue" [IDENT] | "yield" [expression]
lambda        := ["owned" | "once"] "fn" "(" [lambda_param {"," lambda_param} [","]] ")"
                 ["->" type] ("=>" expression | ":" block)
lambda_param  := ["mut" | "owned"] IDENT [":" type]
ternary       := or_expr ["if" or_expr "else" ternary]
or_expr       := and_expr {"or" and_expr}
and_expr      := not_expr {"and" not_expr}
not_expr      := "not" not_expr | comparison
comparison    := range_expr {comp_op range_expr}
comp_op       := "==" | "!=" | "<" | ">" | "<=" | ">=" | "is" ["not"] | ["not"] "in"
range_expr    := [bitor] (".." | "..=") [bitor] | bitor
bitor         := bitxor {"|" bitxor}
bitxor        := bitand {"^" bitand}
bitand        := shift {"&" shift}
shift         := additive {("<<" | ">>") additive}
additive      := multiplicative {("+" | "-") multiplicative}
multiplicative:= cast {("*" | "/" | "//" | "%") cast}
cast          := unary {("as" | "as?" | "as!") type}
unary         := ("-" | "~") unary | "ref" ["mut"] unary | power
power         := postfix_expr ["**" unary]
postfix_expr  := atom {postfix}
postfix       := "(" [arg {"," arg} [","]] ")" | "(" expression comp_clauses ")"
               | "[" expr_list "]" | "." IDENT | "." INT
               | "?." IDENT | "?"
arg           := expression | IDENT "=" expression
atom          := literal | IDENT | "self" | "Self" | "(" expression ")" | tuple_lit | list_lit
               | map_lit | set_lit | comprehension | match_expr | "comptime" "(" expression ")"
               | "@assume_noalloc" "(" expression ")"
tuple_lit     := "(" ")" | "(" expression "," [expression {"," expression} [","]] ")"
list_lit      := "[" [expression {"," expression} [","]] "]" | "[" expression ";" expression "]"
map_lit       := "{" "}" | "{" expression ":" expression {"," expression ":" expression} [","] "}"
set_lit       := "{" expression {"," expression} [","] "}"
comprehension := "[" expression comp_clauses "]"
               | "(" expression comp_clauses ")"
               | "{" expression ":" expression comp_clauses "}"
               | "{" expression comp_clauses "}"
comp_clauses  := "for" for_target "in" or_expr {"for" for_target "in" or_expr | "if" or_expr}
match_expr    := "match" expression ":" NEWLINE INDENT {pattern ["if" expression] "=>" expression NEWLINE}
                 DEDENT
```

* `[GRM-8]` `name[…]` in expression position is resolved during name resolution: if `name` denotes a
  generic function or type it is an instantiation, otherwise an index. `f[int](x)` instantiates then
  calls.
* `[GRM-8a]` Inside `[ ]` in expression position the parser commits to a type argument when the next
  token is `ref`, `*`, `dyn`, `some`, `fn`, `once`, `extern` or `void`; otherwise it parses an
  expression and name resolution reinterprets it as a type where `[GRM-8]` resolved an instantiation.
* `[GRM-8b]` An index whose argument is a type is `E2172`; an instantiation argument that is neither a
  type nor a constant is `E2173`.
* `[GRM-8c]` `IDENT = type` inside `[ ]` is an associated-type binding (`Iterator[Item = int]`).
* `[GRM-10]` A `match` whose arms use `pattern: block` is a statement; one whose arms use
  `pattern => expression` is an expression. Mixing the forms is `E0103`.
* `[GRM-11]` There are no block expressions. A value computed by several statements is written with a
  `match` expression, a conditional expression, a local function, or `comptime(…)`.
* `[GRM-16]` *(changed in 0.9.9)* `return`, `break` and `continue` are expressions of type `Never`;
  `yield e` is an expression of type `void` (`[CORO-3]`). A jump is either a whole expression statement or the whole
  body of a lambda, a match arm, or a conditional-expression branch; `a + return b` is `E0107`.
* `[GRM-15]` `owned e` in expression position is legal only as the iterable of a `for` or the
  scrutinee of a `match`, where it consumes `e`; elsewhere it is `E0109`.
* `[GRM-17]` A lambda with a `:` body inside brackets holds exactly one simple statement (`[LEX-6a]`).
  More is `E0106`, whose help is to declare a local function on the preceding line (`[GRM-28]`).
* `[GRM-21]` `gen fn` declares a generator (`[CORO-1]`). It is legal wherever `fn` is, except in an
  `extern` block.
* `[GRM-23]` `x in c` and `x not in c` are membership tests at comparison precedence. `not in` is one
  operator: `x not in xs` never parses as `(not x) in xs`. Membership does not chain: `a in b in c` is
  `E0102`.
* `[GRM-24]` *(new in 0.9.9)* Ember has one path separator, `.`. A path resolves left to right: package, module, type,
  associated item or variant. `::` is not a token (`[LEX-21]`).
* `[GRM-25]` *(new in 0.9.9)* A chain of the comparison operators `==`, `!=`, `<`, `>`, `<=`, `>=` means what it means in
  Python: `a < b <= c` is `a < b and b <= c`, each operand is evaluated at most once and left to right,
  and evaluation stops at the first false comparison. `is` and `in` may not appear in a chain
  (`E0102`).
* `[GRM-26]` *(new in 0.9.9)* A `{…}` atom is a map literal if its first element is followed by `:`, and a set literal
  otherwise. `{}` is an empty map; an empty set is written `Set[T]()`.
* `[GRM-27]` *(new in 0.9.9)* A comprehension is shorthand for an iterator pipeline and has exactly its meaning and
  cost: `[f(x) for x in xs if p(x)]` is `xs.iter().filter(fn(x) => p(x)).map(fn(x) => f(x)).collect()`
  into an `Array`; `{…}` forms collect into a `Set` or a `Map`. Clauses nest left to right as in
  Python. The loop variables are scoped to the comprehension.
* `[GRM-38]` *(new in 0.9.9)* A parenthesised comprehension `(e for x in it if c)` is a **generator
  expression**: the pipeline of `[GRM-27]` without the `collect`, a lazy `Iterator` that borrows what it
  iterates and allocates nothing. As the only argument of a call it needs no extra parentheses:
  `sum(x * x for x in xs)`, `any(p.hp <= 0 for p in players)`. `(e)` without `for` is a parenthesised
  expression, and `(e,)` a tuple.
* `[GRM-30]` *(new in 0.9.9)* `as?` and `as!` are single tokens (`[LEX-21]`): `h as? D` is a checked downcast
  (`[TYP-6]`).
* `[GRM-32]` *(new in 0.9.9)* `comptime(e)` evaluates the expression `e` at compile time (`[CT-6]`).
* `[GRM-36]` *(new in 0.9.9)* `ref e` and `ref mut e` borrow the place `e` (`[BRW-1]`). The operand MUST
  be a place — a local, a field, an index or a dereferenced reference — and `ref` of any other
  expression is `E0111`, whose help is to bind the value to a local first. `ref` binds like unary
  minus: `ref a.b[i]` borrows `a.b[i]`.

## III.6 Patterns

```text
pattern       := bind_pattern {"|" bind_pattern}
bind_pattern  := IDENT "@" primary_pattern | primary_pattern
primary_pattern := "_" | literal_pattern | range_pattern | IDENT
               | path "(" [field_pattern {"," field_pattern}] ["," ".."] ")"
               | path
               | "(" [pattern {"," pattern} [","]] ")"
               | "[" [pattern {"," pattern}] ["," ".." [IDENT]] "]"
               | "ref" ["mut"] IDENT
literal_pattern := ["-"] INT | STRING | CHAR | "true" | "false"
range_pattern := literal_pattern (".." | "..=") literal_pattern
field_pattern := pattern | IDENT "=" pattern
path          := IDENT {"." IDENT}
```

* `[GRM-12]` An identifier in pattern position names a unit variant or a `const` if one of that name is
  in scope; otherwise it binds a new name. A binding that shadows a variant of another enum in scope is
  `W1002`.
* `[GRM-13]` Matching a place that is not consumed binds `Copy` fields by value and other fields by
  reference; `match owned e:` binds by move.
* `[ENM-1]` *(changed in 0.9.9)* Inside a pattern whose scrutinee type is known, a variant may be
  written unqualified (`Circle(r)`); in an expression it is qualified (`Shape.Circle(1.0)`) unless
  imported.
