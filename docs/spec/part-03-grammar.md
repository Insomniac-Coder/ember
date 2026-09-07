# Part III — Grammar

This is the complete v1 grammar in EBNF. `{x}` = zero or more, `[x]` = optional, `|` = alternation. Terminal tokens are in quotes or UPPERCASE. The parser is a hand-written recursive-descent parser with Pratt-style expression parsing; the grammar is LL(2) except where noted.

## III.1 Compilation unit

```ebnf
file            := [directive] {NEWLINE} {import_decl} {item}
directive       := "#!" "language" string_lit NEWLINE

import_decl     := "import" module_path ["as" identifier] NEWLINE
                 | "from" module_path "import" import_list NEWLINE
                 | "import" "c" string_lit [ffi_opts] NEWLINE          (* C header or source *)
                 | "import" "cpp" string_lit [ffi_opts] NEWLINE        (* C++ header *)
module_path     := identifier {"." identifier}
import_list     := import_item {"," import_item} | "(" import_item {"," import_item} [","] ")"
import_item     := identifier ["as" identifier] | "*"
ffi_opts        := "with" "(" ffi_opt {"," ffi_opt} ")"
ffi_opt         := identifier "=" expression                          (* e.g. link="vulkan-1", overlay="vk.embind.em" *)

item            := {attribute} [visibility] item_body
visibility      := "pub" ["(" vis_args ")"]
vis_args        := "package" | "read" | "package" "," "read"      (* `read` is valid on fields only *)
item_body       := fn_decl | struct_decl | class_decl | enum_decl | interface_decl
                 | extend_decl | const_decl | static_decl | type_alias | extern_block
                 | comptime_block | test_decl
attribute       := "@" identifier ["(" [attr_args] ")"] NEWLINE?
attr_args       := attr_arg {"," attr_arg}
attr_arg        := expression | identifier "=" expression
```

## III.2 Declarations

```ebnf
fn_decl         := fn_header ":" block
                 | fn_header NEWLINE                                    (* only inside interface/extern *)
fn_header       := ["unsafe"] ["virtual" | "override"] "fn" identifier [generic_params]
                   "(" [param_list] ")" ["->" type] [where_clause]
generic_params  := "[" generic_param {"," generic_param} "]"
generic_param   := identifier [":" bound_list] ["=" type]
                 | "const" identifier ":" type                          (* const generic *)
bound_list      := type {"+" type}
where_clause    := "where" bound {"," bound}
bound           := type ":" bound_list
param_list      := param {"," param} [","]
param           := receiver | [mode] identifier ":" type ["=" expression]
receiver        := "self" | "mut" "self" | "owned" "self" | "self" ":" type
mode            := "mut" | "owned"

struct_decl     := "struct" identifier [generic_params] [implements_clause] [where_clause] ":" type_body
class_decl      := ["open" | "abstract"] "class" identifier [generic_params]
                   ["(" type ")"] [implements_clause] [where_clause] ":" type_body
implements_clause := "implements" type {"," type}
type_body       := NEWLINE INDENT {type_member} DEDENT | "pass" NEWLINE
type_member     := {attribute} [visibility] (field_decl | fn_decl | const_decl | type_alias | "pass" NEWLINE)
field_decl      := identifier ":" type ["=" expression] NEWLINE

enum_decl       := "enum" identifier [generic_params] [implements_clause] ":" NEWLINE INDENT {enum_member} DEDENT
enum_member     := {attribute} (variant | fn_decl | const_decl)
variant         := identifier ["(" variant_fields ")"] ["=" expression] NEWLINE
variant_fields  := (identifier ":" type | type) {"," (identifier ":" type | type)}

interface_decl  := "interface" identifier [generic_params] [":" bound_list] [where_clause] ":" NEWLINE
                   INDENT {interface_member} DEDENT
interface_member:= {attribute} (fn_decl | "type" identifier [":" bound_list] NEWLINE | const_decl)

extend_decl     := "extend" type [implements_clause] [where_clause] ":" type_body

const_decl      := "const" identifier [":" type] "=" expression NEWLINE
static_decl     := "static" ["mut"] identifier ":" type "=" expression NEWLINE
type_alias      := "type" identifier [generic_params] "=" type NEWLINE

extern_block    := ["unsafe"] "extern" string_lit ":" NEWLINE INDENT {extern_item} DEDENT
extern_item     := {attribute} (fn_header NEWLINE | static_decl | "type" identifier NEWLINE)

comptime_block  := "comptime" ":" block
test_decl       := "@test" NEWLINE fn_decl                              (* attribute form; no special syntax *)
```

Notes:
* A class's base class is written in parentheses: `class Door(Script):`. `[GRM-1]` At most one base class.
* `abstract class` may declare `virtual fn` without a body (abstract method); `[GRM-2]` a non-abstract class MUST implement all inherited abstract methods.
* `fn_decl` inside a `struct`/`class`/`enum`/`extend` body is a method if its first parameter is a receiver, otherwise an associated (static) function called as `Type.name(...)`.

## III.3 Types

```ebnf
type            := path_type | ref_type | ptr_type | tuple_type | fn_type | dyn_type | array_type | "Self" | "void" | "!"
path_type       := identifier {"." identifier} [generic_args]
generic_args    := "[" generic_arg {"," generic_arg} "]"
generic_arg     := type | expression                                    (* const generic argument *)
                 | identifier "=" type                                  (* associated type binding *)
ref_type        := "ref" ["mut"] type
ptr_type        := "*" ["mut"] type                                     (* raw pointer *)
tuple_type      := "(" ")" | "(" type "," [type {"," type}] ")"
fn_type         := ["extern" string_lit] "fn" "(" [type {"," type}] ")" ["->" type]
dyn_type        := "dyn" bound_list
array_type      := "[" type ";" expression "]"                          (* fixed-size inline array, e.g. [f32; 16] *)
```

`[GRM-3]` `Span[T]`, `MutSpan[T]`, `Array[T]`, `Option[T]`, `Result[T, E]`, `Box[T]`, `Shared[T]`, `Weak[T]` are ordinary library types spelled with `path_type`; the grammar does not special-case them.

## III.4 Statements

```ebnf
block           := NEWLINE INDENT {statement} DEDENT | simple_stmt NEWLINE
statement       := simple_stmt NEWLINE | compound_stmt
simple_stmt     := small_stmt {";" small_stmt} [";"]
small_stmt      := var_decl | assignment | expression | "return" [expression] | "break" [label]
                 | "continue" [label] | "pass" | "defer" ":" ...      (* defer is compound, see below *)
var_decl        := pattern ":" type ["=" expression]                    (* typed declaration, may be uninitialised *)
assignment      := target_list ("=" | augassign) expression
target_list     := target {"," target}                                  (* tuple destructuring *)
target          := identifier | postfix_expr | "_" | "(" target_list ")"
augassign       := "+=" | "-=" | "*=" | "/=" | "%=" | "**=" | "&=" | "|=" | "^=" | "<<=" | ">>="

compound_stmt   := if_stmt | while_stmt | for_stmt | match_stmt | with_stmt | defer_stmt
                 | unsafe_stmt | comptime_stmt | labeled_stmt
if_stmt         := "if" condition ":" block {"elif" condition ":" block} ["else" ":" block]
condition       := expression | pattern "=" expression                  (* `if Some(x) = opt:` *)
while_stmt      := "while" condition ":" block ["else" ":" block]
for_stmt        := "for" pattern "in" expression ":" block ["else" ":" block]
labeled_stmt    := identifier ":" (while_stmt | for_stmt)               (* `outer: for ...` *)
match_stmt      := "match" expression ":" NEWLINE INDENT {match_arm} DEDENT
match_arm       := pattern ["if" expression] ":" block
with_stmt       := "with" with_item {"," with_item} ":" block
with_item       := [pattern "="] expression
defer_stmt      := "defer" ":" block
unsafe_stmt     := "unsafe" ":" block
comptime_stmt   := "comptime" ":" block
```

Semantics notes tied to the grammar:
* `[GRM-4]` `x = expr` where `x` is not in scope declares `x` with the inferred type of `expr`. Where `x` is in scope, it assigns. `x: T = expr` always declares (shadowing in a nested block is allowed; redeclaring in the same block is `E1020`).
* `[GRM-5]` `a, b = expr` destructures a tuple/struct (pattern assignment); all names are declared or assigned consistently.
* `[GRM-6]` The optional `else` on `while`/`for` runs when the loop ends without `break` (Python semantics).
* `[GRM-7]` `defer` blocks run in reverse order at block exit, including on `return` and on panic-unwind when unwinding is enabled.

## III.5 Expressions

Precedence, lowest to highest (each row left-associative unless noted):

| Level | Operators | Notes |
|---|---|---|
| 1 | `x if c else y` | ternary, right-assoc |
| 2 | `or` | |
| 3 | `and` | |
| 4 | `not` (prefix) | |
| 5 | `==` `!=` `<` `>` `<=` `>=` `is` `is not` `in` `not in` | non-associative; chaining `a < b < c` is `E0102` (no Python chaining) |
| 6 | `..` `..=` | non-associative |
| 7 | `\|` | |
| 8 | `^` | |
| 9 | `&` | |
| 10 | `<<` `>>` | |
| 11 | `+` `-` | |
| 12 | `*` `/` `%` | |
| 13 | `-` `~` (prefix) | |
| 14 | `**` | right-assoc; binds tighter than unary minus on its left: `-2**2 == -4` |
| 15 | `?` (postfix try), `as` (cast) | |
| 16 | call `f(...)`, index `a[...]`, field `.x`, method `.m(...)`, optional chaining `?.`, generic instantiation `f[T]` | |
| 17 | atoms | |

```ebnf
expression      := ternary
ternary         := or_expr ["if" or_expr "else" ternary]
...                                                                     (* per table *)
postfix_expr    := atom {postfix}
postfix         := "(" [arg_list] ")" | "[" index_args "]" | "." identifier | "?." identifier
                 | "." int_lit (* tuple field *) | "?" | "as" type
arg_list        := arg {"," arg} [","]
arg             := expression | identifier "=" expression               (* named argument *)
index_args      := expression {"," expression}                          (* multi-dim index → Index[(A,B)] *)
atom            := literal | identifier | "self" | "(" expression ")" | tuple_lit | array_lit
                 | lambda | match_expr | block_expr | "Self" | path_expr
tuple_lit       := "(" ")" | "(" expression "," [expression {"," expression}] ")"
array_lit       := "[" [expression {"," expression} [","]] "]" | "[" expression ";" expression "]"
lambda          := ["owned"] "fn" "(" [lambda_params] ")" ["->" type] ("=>" expression | ":" block)
lambda_params   := lambda_param {"," lambda_param}
lambda_param    := [mode] identifier [":" type]
match_expr      := "match" expression ":" NEWLINE INDENT {pattern ["if" expression] "=>" expression NEWLINE} DEDENT
path_expr       := identifier "::" identifier {"::" identifier}         (* explicit path, e.g. Shape::Circle; `.` also works *)
```

`[GRM-8]` **Generic-instantiation vs indexing.** `name[...]` in expression position is parsed as an `IndexOrInstantiate` node and resolved during name resolution: if `name` resolves to a generic function or type, it is an instantiation; otherwise an index. `[GRM-9]` A call immediately following (`f[i32](x)`) does not change this rule.

`[GRM-10]` **Statement vs expression `match`.** `match e:` followed by arms using `pattern: block` is a statement; arms using `pattern => expr` make it an expression. Mixing is `E0103`.

`[GRM-11]` Block expressions `(: ... )`: **not in v1**. Use a helper function or a `match` expression.

## III.6 Patterns

```ebnf
pattern         := alt_pattern
alt_pattern     := bind_pattern {"|" bind_pattern}
bind_pattern    := identifier "@" primary_pattern | primary_pattern
primary_pattern := "_" | literal | range_pattern | identifier            (* binding or unit variant/const *)
                 | path_type "(" [field_patterns] ")"                    (* variant/struct/tuple-struct *)
                 | "(" [pattern {"," pattern}] ")"                       (* tuple *)
                 | "[" [pattern {"," pattern} ["," ".." [identifier]]] "]"  (* slice *)
                 | "ref" ["mut"] identifier                               (* bind by reference *)
                 | "Some" "(" pattern ")" | "None" | "Ok" "(" pattern ")" | "Err" "(" pattern ")"
range_pattern   := literal ".." literal | literal "..=" literal
field_patterns  := field_pattern {"," field_pattern} ["," ".."]
field_pattern   := pattern | identifier "=" pattern                       (* positional or named *)
```

* `[GRM-12]` An identifier in pattern position resolves to a unit variant or `const` if one of that name is in scope; otherwise it is a fresh binding. The compiler warns `W1002` when a binding shadows a same-named variant in another enum to catch typos.
* `[GRM-13]` Patterns bind by value for `Copy` types and by **reference** (`ref`) otherwise when matching on a place expression that is not consumed; `match owned x:` consumes and binds by move. This mirrors Rust's default binding modes.

## III.7 Grammar of attributes recognised by the compiler

Unknown attributes are `E0104` unless prefixed with a registered plugin namespace (`@ragev.field`). Recognised v1 attributes (full semantics where referenced):

| Attribute | Applies to | Kind |
|---|---|---|
| `@derive(A, B, …)` | struct, class, enum | code generation (Part XIV) |
| `@layout(c)` `@packed` `@align(N)` `@repr(int_type)` | struct, enum | layout contract (Part IX) |
| `@gpu_layout(std140 \| std430 \| scalar)` | struct | layout contract (Part XVII) |
| `@noalloc` `@nosync` `@nopanic`(v2) | fn | hard contract (Part X) |
| `@inline` `@noinline` `@cold` `@hot` | fn | hint |
| `@simd` `@parallel` `@unroll(N)` | for statement, fn | hint / contract (Part XII) |
| `@overflow(panic \| wrap \| saturate)` | fn, module | semantics (Part VI) |
| `@fastmath` | fn | semantics (Part VI) |
| `@must_use` `@deprecated("msg")` | fn, type | diagnostics |
| `@export("symbol")` | fn, static | ABI (Part XVI) |
| `@ffi(...)` | extern fn / overlay | FFI contract (Part XVI) |
| `@test` `@bench` `@should_panic` | fn | toolchain |
| `@view` | struct | marks a borrow-carrying type (Part VII) |
| `@move_only` | struct | disables `Copy` derivation |
| `@sync` `@thread_local` | class | overrides `Sync` derivation (Part XI) |
| `@reflect` `@serialize` | type | metadata generation (Part XIV) |

---
