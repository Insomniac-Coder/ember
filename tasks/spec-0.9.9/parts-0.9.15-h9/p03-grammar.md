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
directive     := "#!" ("language" STRING | "threads" IDENT | "module" module_attr) NEWLINE
module_attr   := attr_name ["(" [attr_arg {"," attr_arg} [","]] ")"]
import_decl   := "import" module_path ["as" IDENT] NEWLINE
               | "from" module_path "import" import_list NEWLINE
               | ["pub"] ["unsafe"] "import" ("c" | "cpp") STRING [ffi_opts] ["as" IDENT] NEWLINE
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
* `[GRM-37]` *(new in 0.9.9)* *(changed in 0.9.15)* A **directive** is a line beginning `#!` before the imports.
  `#! language "<version>"` is accepted only for the current version (`[VER-8]`, `E0006`);
  `#! threads main|any|creator` sets the default thread contract of the file's exports (`[FFI-33]`);
  `creator` on a static free-function export uses that rule's successful module-initialization
  anchor, not a per-callable instance.
  `#! module name(args)` applies one attribute to this source module, using the same name and
  argument grammar as `@name(args)` without the `@` (ODR-084). Only attributes whose documented
  targets include modules are permitted (`[ATT-1]`); for example, `#! module overflow(saturate)`.
  Different module attributes may appear on separate directive lines; repeating the same attribute
  name is `E0104`. A directive after an import or item, an unknown directive, or a module attribute
  with an invalid target or arguments is `E0104`. A valid but unimplemented module attribute is
  `E0900` (`[ATT-6]`), never silently ignored. `import cpp` is Annex C; `unsafe import c` before a
  C source file compiled into the program is Part XVI's.

## III.2 Items

```text
item          := {attribute} [type_params] [visibility] item_body
type_params   := generic_params NEWLINE                -- above a fn, struct or class only ([GRM-40])
attribute     := "@" attr_name ["(" [attr_arg {"," attr_arg} [","]] ")"] NEWLINE
              | "@" "unsafe" "(" attr_name ["(" [attr_arg {"," attr_arg} [","]] ")"] ")" NEWLINE
                                                     -- an attribute whose promise the compiler cannot check ([FFI-26])
attr_name     := IDENT {"." IDENT}                   -- dotted names are plugin namespaces
attr_arg      := expression | IDENT "=" expression
visibility    := "pub" ["(" ("package" | "read" | "package" "," "read") ")"]
item_body     := fn_decl | struct_decl | class_decl | enum_decl | interface_decl
               | extend_decl | const_decl | static_decl | type_alias | extern_block
               | comptime_block

fn_decl       := fn_header ":" block
               | fn_header NEWLINE                     -- bodiless: interface, extern, abstract only
fn_header     := ["extern" STRING] ["unsafe"] ["static" | "virtual" | "override"] ["gen"]
                 "fn" IDENT "(" [param_list] ")" [fn_takes] ["->" type ["from" IDENT]]
                 [where_clause]
fn_takes      := "owns" "this"                         -- a method that takes its object over
               | capture_list                          -- a local function ([GRM-42])
generic_params:= "[" generic_param {"," generic_param} [","] "]"
generic_param := IDENT ["..."] [":" "needs" needs] ["=" type]   -- `...`: a pack ([TYP-47])
               | "const" IDENT ":" type ["=" expression]
needs         := type | "(" type {"," type} [","] ")"     -- one bare, several in round brackets ([TYP-17])
bound_list    := type {"+" type}                        -- after `dyn` and `some` only
where_clause  := "where" type ":" "needs" needs {"," type ":" "needs" needs}
param_list    := param {"," param} [","]
param         := [param_word] IDENT ":" type ["..."] ["=" expression]   -- `Ts...`: the pack's values
param_word    := "ref" ["mut"] | "const" "ref" "mut" | "owns"

struct_decl   := "struct" IDENT [implements] [where_clause] ":" type_body
class_decl    := ["open" | "abstract"] "class" IDENT ["(" type ")"]
                 [implements] [where_clause] ":" type_body
implements    := "implements" type {"," type}
type_body     := NEWLINE INDENT {member} DEDENT | "pass" NEWLINE
member        := {attribute} [type_params] [visibility] (field_decl | fn_decl | const_decl
               | type_alias | "pass" NEWLINE)
field_decl    := ["let"] IDENT ":" type ["from" (IDENT | "static")] ["guarded" "by" IDENT]
                 ["=" expression] NEWLINE

enum_decl     := "enum" IDENT [generic_params] [implements] [where_clause] ":"
                 NEWLINE INDENT {enum_member} DEDENT
enum_member   := {attribute} (variant | [type_params] fn_decl | const_decl)
variant       := IDENT ["(" variant_field {"," variant_field} [","] ")"] ["=" expression] NEWLINE
variant_field := {attribute} [IDENT ":"] type

interface_decl:= ["unsafe"] "interface" IDENT [generic_params] [":" "needs" needs] [where_clause] ":"
                 NEWLINE INDENT {interface_member} DEDENT
interface_member := {attribute} ([type_params] fn_decl | assoc_type | const_decl)
assoc_type    := "type" IDENT [":" "needs" needs] ["=" type] NEWLINE

extend_decl   := ["unsafe"] "extend" [generic_params] type [implements] [where_clause] ":" type_body
const_decl    := "const" IDENT [":" type] "=" expression NEWLINE
static_decl   := "static" ["mut"] IDENT ":" type ["guarded" "by" IDENT] "=" expression NEWLINE
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
* `[GRM-34]` *(new in 0.9.9)* *(changed in 0.9.10)* `extend [T: needs B] Array[T] implements I:` declares a generic extension; the generic
  parameters after `extend` scope over the whole block. A parameter the target does not name and an
  implemented interface does makes a **blanket implementation**: `extend[K: needs (Eq, Hash), V,
  Q: needs AsKey[K]] Map[K, V] implements Index[Q]` makes every `Map[K, V]` implement `Index[Q]` for each `Q`
  its bounds admit, each method of the block is generic over `Q`, and each use is checked as a
  written implementation is (ODR-042). The target may be a parameter standing alone (`extend[T, U]
  T implements Wrap[U]`, every type's); a parameter of the block that neither the target nor an
  implemented interface names is `E2020`. A call of a blanket method whose arguments and wanted
  result leave such a parameter open takes it from the one instance of the interface the
  receiver's type implements through the block (an instance of a generic function has it from its
  bound); with two it stays open (`E2060`, ODR-112).
* `[GRM-8d]` A `type_alias` with an `in` clause declares a range type (`[RNG-1]`). The clause is
  legal only on an item-level alias with no generic parameters (`E2213` otherwise).
* `[GRM-40]` *(new in 0.9.15)* **Type parameters go on a line of their own**, directly above the
  function, method, struct or class they belong to, at its indentation and below its attributes:
  `[T: needs (Add[Output = T], Default)]` above `fn sum(xs: Span[T]) -> T:`, `[F: needs fn(Item)]` above
  `fn for_each(owns f: F) owns this:`, `[T]` above `struct Pool:`. The type or function is used as
  before (`Pool[int]`, `sum(xs)`, `f[int](x)`), and a lambda has none. An enum, an interface, a
  `type` alias and an `extend` block keep their brackets after the name or after `extend`
  (`interface Add[Rhs = Self]:`, `type Pair[T] = (T, T)`, `extend[T: needs B] Array[T]`). Brackets after the
  name of a function, method, struct or class are `E0100`, whose message gives the two lines to
  write (`[T]` above `fn first(xs: Array[T]) -> T:`); a bracket line not directly above one of them is
  `E0100`. A line that begins with `[` is a type-parameter line when the line after it begins a
  function, struct or class declaration (with `pub`, `open`, `abstract`, `static`, `virtual`,
  `override`, `gen`, `unsafe` or `extern` before it, or none); this is the one place where the parser
  looks past a line's end, at the next line's first tokens.
* `[GRM-41]` *(new in 0.9.15)* **A function's first line** gives its parts in this order: the
  parameters, each with the word in front of its name; `owns this`, for a method that takes its
  object over, or, for a local function, its capture words (`[GRM-42]`); `->` and the result type;
  `from` and what the result points into (`fn finish() owns this -> u64:`, `fn next_word() -> StrView
  from src:`). A first line in another order is `E0118`, whose message gives it in order. A
  parameter's word is `ref` (a read-only view, the default, which may be left out), `ref mut`,
  `const ref mut` or `owns` (`[FN-1]`), written on its name: written in its type instead
  (`c: ref mut Counter`) it is `E0117` ("the mode goes on the name: write `ref mut c: Counter`"), and
  `mut` alone on a name, here, on a lambda parameter, on a pattern name or on `this`, is `E0113`,
  whose message gives the line with `ref mut` in the program's own names. A method does not list its
  object among its parameters; inside it the object is `this` (`[FN-4]`), and `self` anywhere is
  `E1010` ("inside a method its object is `this`"). A function in a type body is a method unless it
  is written `static fn`, which has no object and is called on the type
  (`static fn starting_at(k: int) -> Counter:`, called `Counter.starting_at(5)`); `static fn`
  outside a type body is `E2115`, whose message says to remove `static`, and `this` where there is no
  object (a function outside a type body, a `static fn`) is `E1010`. Nothing else follows the
  brackets: a clause that says what the function changes or reads (`changes FRAME`, `reads
  renderer`) is `E0100`, whose message gives the first line without it, since the compiler works out
  what every function changes and reads from its code (Part X).

## III.3 Types

```text
type          := base_type ["using" path_type]
base_type     := path_type | "ref" ["mut"] base_type | "sharedref" "mut" base_type
               | "owns" base_type | "shares" ["mut"] base_type | "thread_shared" base_type
               | "weakref" base_type | "*" ["mut"] base_type | tuple_type | fn_type
               | "dyn" bound_list | "some" bound_list | "[" type ";" expression "]"
               | "Self" | "void"
path_type     := path_segment {"." path_segment}
path_segment  := IDENT [generic_args]
generic_args  := "[" generic_arg {"," generic_arg} [","] "]"
generic_arg   := type | expression | IDENT "=" type          -- const argument; associated-type binding
               | IDENT "..."                               -- a pack spelled out ([TYP-47])
tuple_type    := "(" ")" | "(" type "," [type {"," type} [","]] ")"
fn_type       := ["once"] ["unsafe"] ["extern" STRING] "fn" "(" [fn_type_param {"," fn_type_param} [","]] ")"
                 ["->" type]
fn_type_param := [param_word] type
```

* `[GRM-3]` *(changed in 0.9.15)* `Array[T]`, `Map[K, V]`, `Option[T]`, `Result[T, E]`, `Span[T]`, and the other
  library types are ordinary `path_type`s; the grammar does not special-case them. Generic arguments
  may appear on any path segment: `SoA[Particle].Ref`. The words before a type (`owns`, `shares`,
  `shares mut`, `thread_shared`, `weakref`, `ref`, `ref mut`, `sharedref mut`) and `using` after one
  say who owns the value, how it may be reached and where its memory comes from (`[TYP-45]`,
  `[TYP-46]`); `using` applies to the whole type before it, so `owns Mesh using LevelArena` is an
  `owns Mesh` whose block comes from a `LevelArena`. A function type's parameter words are the
  parameters' own (`fn(ref mut Ui)`, `fn(owns String) -> int`), and `thread_shared fn()` is a function
  any thread may run.
* `[GRM-31]` *(new in 0.9.9)* *(changed in 0.9.10)* A `some` type (`[TYP-32]`) is legal only as a function's or a
  method's return type, or nested inside one; elsewhere it is `E2260`. A method an interface declares
  is not one: each implementation would choose its own type, so an associated type states it
  (`type Iter: needs Iterator[Item = T]`) (ODR-144).

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
               | unsafe_stmt | safe_stmt | comptime_stmt | labeled_stmt | [type_params] fn_decl
if_stmt       := "if" condition ":" block {"elif" condition ":" block} ["else" ":" block]
condition     := expression | pattern "=" expression
               | expression "exists" "as" opened_name              -- [GRM-44]
               | expression "is" type "as" opened_name             -- if and elif only, [GRM-45]
opened_name   := [name_word] IDENT | "(" opened_name {"," opened_name} [","] ")"
name_word     := "ref" ["mut"] | "copy" "of"
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
safe_stmt     := "safe" ":" block                                -- [UNS-12]
comptime_stmt := "comptime" ":" block
```

* `[GRM-4]` *(changed in 0.9.9)* *(changed in 0.9.15)* `x = e` where no `x` is in scope declares `x` with the type of `e`;
  where `x` is in scope it assigns: the new value goes into the place `x` stands for (`[TYP-14]`). `x:
  T = e` always declares; a declaration may shadow a name from an enclosing block, and redeclaring a
  name in the same block is `E1020`. A declaration inside a loop's body declares its name once per
  round: each round's name is a new one. A name declared in every arm of an exhaustive branch is
  hoisted to the enclosing block (`[CTL-10]`).
* `[GRM-5]` `a, b = e` destructures a tuple, a struct or a fixed array. The right side is evaluated
  once, into a temporary, before any target is written (`[EXP-2]`).
* `[GRM-29]` *(new in 0.9.9)* An `expr_list` of two or more expressions, or of one expression followed by a comma, is a
  tuple: `a, b = b, a` swaps, and `return x, y` returns a tuple.
* `[GRM-6]` The optional `else` of `while` and `for` runs when the loop ends without `break`.
* `[GRM-7]` `defer` blocks run in reverse order at the exit of the enclosing block, on every path out
  of it, including `return`, `break` and `continue`. A `defer` block may not itself jump out (`E2160`).
* `[GRM-28]` *(new in 0.9.9)* *(changed in 0.9.15)* A `fn_decl` inside a block declares a **local function**. It may capture locals of the
  enclosing function exactly as a lambda does (`[CLO-2]`), with the same capture words after its
  brackets (`fn push_a() thread_shared log:`), may call itself recursively, and is in scope from its
  declaration to the end of the block. It is a value of its function type and may be passed by its
  name, so a long lambda may be written inside the call or as a local function passed by its name
  (`a = thread.spawn(push_a)`).
* `[GRM-18]` `;` never separates statements. `a = 1; b = 2` is `E0105`, whose help puts each statement
  on its own line. `;` appears only inside `[T; N]` and `[v; N]`.
* `[GRM-19]` *(changed in 0.9.9)* *(changed in 0.9.15)* The pattern of a `condition` MUST be refutable. An irrefutable one
  is `E2036`; when it is a bare name already in scope (`if x = 5:`), the primary help is
  `did you mean 'x == 5'?`. The pattern form tests a variant other than an `Option`'s (`if Ok(v) =
  r:`, `if State.Walking(ref mut path) = n.state:`); `Some(...)` as the pattern of an `if`, `elif` or
  `while` header is `E0115`, since an `Option` is opened with `exists as` (`[GRM-44]`).
* `[GRM-20]` The attributes before a statement are limited to `@parallel`, `@unroll`, `@simd` and
  `@allow` (`[ATT-2]`).
* `[ATT-3]` A statement attribute attaches to the next compound statement, never to a simple one
  (`E0108`); `@simd`, `@parallel` and `@unroll` require a `for` (`E0108`). Several may precede one
  statement, one per line, in any order.
* `[GRM-44]` *(new in 0.9.15)* **Opening an `Option` or a `weakref` link: `exists as`.**
  `if cam.target exists as p:` runs its block when `cam.target` holds something (an `Option` holds a
  value; a `weakref` link's target still lives) and names that thing `p` inside the block; `else:`
  runs when it holds nothing; `while cur.parent exists as p:` repeats while it holds something,
  naming it each round. `exists` says what the line checks and `as p` gives the name, as in
  `import x as y`. The name is declared there with its words, as a pattern name is (`[GRM-13]`):
  plain `p` is a read-only view and never changes (`E3023`); `if ui exists as ref mut me:` may change
  what it names; `if scores.get(name) exists as copy of v:` is a copy of the program's own, which may
  be changed (`v += 1`; the map is untouched), the same as `if scores.get(name) exists as v:` then
  `mine = copy of v`; `if pair exists as (ref mut a, ref b):` opens an `Option` of a pair, each name
  with its own word (`[GRM-42]`). A name opened from a link is a view of an object reached through a
  link, which nothing that may end that object outlives (Part VIII). The value must be able to hold
  nothing, an `Option` or a `weakref` link: anything else would run the block every time, which is
  `E2036` ("`n` is an `int`, which always holds a value, so this `if` would always run; to name it
  write `v = n` in place of the `if` line, and use `v` in its block, moved out one level"; for a
  `Result`, "write `if Ok(v) = r:`"). The name is new: one already in use is `E2106` ("`exists as`
  declares a new name: write `if cam.target exists as t:`, and `t` for `target` in its block").
  `Some(...)` makes a value (`best = Some(5)`) and names a `match` branch (`Some(p):`), never an `if`
  or `while` header (`E0115`, `[GRM-19]`); a value's kind is tested with `is` (`[GRM-45]`).
  `Some(...)` in an `if` or `while` header is E0115 ("an `Option` or a `weakref` link is opened with `exists as`: write `if ui exists as ref mut me:`").
* `[GRM-45]` *(new in 0.9.15)* **The kind test.** In an `if` or `elif` header, `s is Circle as c`
  runs the block when `s`, a value of an interface or base-class type, is a `Circle`, and names it
  `c` there with its words (`if this.pools[k] is Pool[T] as ref mut pool:`); the type after `is` and
  the name after `as` end the header, so this `as` names and never converts. Other kinds are tested
  with `elif s is Square as q:` and `else:`, which is how a choice among kinds is written. `s is
  Circle`, without a name, is a `bool` wherever an expression may stand. The named form is legal only
  in an `if` or `elif` header (`E0100` elsewhere). What a test means and what it refuses is
  `[TYP-44]`.

## III.5 Expressions

Precedence, lowest first. Operators on one row associate left unless noted.

| Level | Operators | Notes |
|---|---|---|
| 1 | `return e`, `break`, `continue`, `yield e` | jump expressions; must be a whole expression statement or a whole arm/branch body (`[GRM-16]`) |
| 2 | `fn(…) => e`, `fn(…) copy of x: …`, `once fn …` | lambda, with its capture words after the brackets (`[GRM-42]`); body extends as far right as possible |
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
| 15 | `as` | postfix type conversion |
| 16 | `-` `~` · `ref` `ref mut` `const ref mut` `copy of` `owns` `shares` `shares mut` `thread_shared` `weakref` `sharedref mut` · `cast … as` (prefix) | `-x as u8` is `(-x) as u8`; `ref a.b[i]` names `a.b[i]`; `copy of xs[0]` copies `xs[0]` |
| 17 | `**` | right-associative; binds tighter than a prefix `-` on its left: `-2**2 == -4` |
| 18 | `using` | the instance a value is made in: `owns Mesh(…) using current` is `owns (Mesh(…) using current)` |
| 19 | call, index, field, method, `?.`, postfix `?` | |
| 20 | atoms | |

```text
expression    := jump_expr | lambda | ternary
jump_expr     := "return" [expr_list] | "break" [IDENT] | "continue" [IDENT] | "yield" [expression]
lambda        := ["@safety" "(" STRING ")"] ["once"] "fn" "(" [lambda_param {"," lambda_param} [","]] ")" [capture_list]
                 ["->" type] ("=>" expression | ":" block)
lambda_param  := [param_word] IDENT [":" type]
capture_list  := capture {"," capture}
capture       := capture_word (IDENT | "this")
capture_word  := "copy" "of" | "owns" | "shares" ["mut"] | "thread_shared" | "weakref" | "ref"
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
multiplicative:= conversion {("*" | "/" | "//" | "%") conversion}
conversion    := unary {"as" type}
unary         := ("-" | "~") unary | value_word unary | "cast" postfix_expr "as" type | power
value_word    := "ref" ["mut"] | "const" "ref" "mut" | "copy" "of" | "owns"
               | "shares" ["mut"] | "thread_shared" | "weakref" | "sharedref" "mut"
power         := made ["**" unary]
made          := postfix_expr ["using" postfix_expr]
postfix_expr  := atom {postfix}
postfix       := "(" [arg {"," arg} [","]] ")" | "(" expression comp_clauses ")"
               | "[" expr_list "]" | "." IDENT | "." INT
               | "?." IDENT | "?"
arg           := expression | IDENT "=" expression | IDENT "..."   -- a pack's values passed on
atom          := literal | IDENT | "this" | "Self" | "(" expression ")" | tuple_lit | list_lit
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
* `[GRM-8a]` *(changed in 0.9.15)* Inside `[ ]` in expression position the parser commits to a type argument when the next
  token is `ref`, `owns`, `shares`, `thread_shared`, `weakref`, `sharedref`, `*`, `dyn`, `some`, `fn`,
  `once`, `extern` or `void`; otherwise it parses an expression and name resolution reinterprets it
  as a type where `[GRM-8]` resolved an instantiation.
* `[GRM-8b]` *(changed in 0.9.10)* *(changed in 0.9.15)* An index whose argument is a type is `E2172`; an instantiation
  argument that is neither a type nor a constant, or an interface named bare as a type, which does not
  say who owns its values (`[TYP-45]`), is `E2173`.
* `[GRM-8c]` `IDENT = type` inside `[ ]` is an associated-type binding (`Iterator[Item = int]`).
* `[GRM-10]` A `match` whose arms use `pattern: block` is a statement; one whose arms use
  `pattern => expression` is an expression. Mixing the forms is `E0103`.
* `[GRM-11]` There are no block expressions. A value computed by several statements is written with a
  `match` expression, a conditional expression, a local function, or `comptime(…)`.
* `[GRM-16]` *(changed in 0.9.9)* `return`, `break` and `continue` are expressions of type `Never`;
  `yield e` is an expression of type `void` (`[CORO-3]`). A jump is either a whole expression statement or the whole
  body of a lambda, a match arm, or a conditional-expression branch; `a + return b` is `E0107`.
* `[GRM-15]` *(changed in 0.9.10)* *(changed in 0.9.15)* `owns e` hands `e` over. It is written, and required, on the line
  that makes a value for a place that owns it alone (`mesh=owns Mesh(points=[0.0])`,
  `left=Some(owns Tree(value=2))`, `e.mesh = owns Mesh(points=[1.0, 2.0])`; `[TYP-45]`, `E2123`);
  before the iterable of a `for` or of a comprehension's `for` clause (`[GRM-27]`) and before the
  scrutinee of a `match`, where the default is a view, so that the loop or the branches take the value
  over (`for x in owns xs:`, `match owns e:`); and in `b = owns n` for a number, which `=` would copy,
  so that `n` ends there. Before a name whose place already hands the value over with no word
  (`e.mesh = m`, `b = a` for a value that is not a number, an argument, `return a`), and before
  `fn`, it is `E0109`, whose message says to remove it (for `owns fn`, to name what the function
  takes on each captured name: `fn() owns report:`).
* `[GRM-17]` *(changed in 0.9.10)* A lambda with a `:` body inside brackets holds exactly one simple
  statement (`[LEX-6a]`). More is `E0106`, whose help is to declare a local function on the preceding
  line (`[GRM-28]`). Outside brackets, a `:` body on the lambda's own line is one simple statement,
  ended by the statement's `NEWLINE`: `h = fn(x): total += x` assigns the lambda to `h`.
* `[GRM-42]` *(new in 0.9.15)* **Capture words, and lists of names without types.** A lambda or a
  local function names what it keeps after its brackets, with one word before each name and nothing
  between the brackets and the first word: `fn() copy of score:`, `fn() owns players:`, `fn() shares
  mut count:`, `fn() thread_shared log:`, `fn() weakref this:`, `fn() ref t:`. `copy of`, `owns`,
  `shares`, `shares mut`, `thread_shared` and `weakref` mean what they mean before a value
  (`[GRM-43]`), and `ref t` is a read-only view of what `t` names at that moment; a name not listed is
  the real variable (`[CLO-2]`). `with` before the captures is `E0100` ("a function's captures follow
  its brackets with no `with`: write `fn() copy of score:`"), and `ref mut` is not a capture word
  (`E0120`: "remove `ref mut x`: without a word the function already uses the real `x`, and may change
  it: `fn(): x += 1`"). In a list of names declared without types of their own (a capture list, a
  loop's names, a pattern's names, the names after `exists as`, a lambda's parameters written without
  types) each word belongs to the one name after it, and a name with no word may not follow a name
  with a word, so that no word can be read as covering the next name (`E0116`: "`ref mut` here belongs
  to `pos` only; say what `vel` is: `for ref mut pos, ref vel in q:`, or `ref mut vel` to change it
  too"). `ref` states a name's default, a read-only view, and may be written on any declared name. A
  parameter list with types, and a call's arguments, are not such lists.
* `[GRM-43]` *(new in 0.9.15)* **Words before a value.** `copy of`, `owns`, `shares`, `shares mut`,
  `thread_shared`, `weakref`, `sharedref mut`, `ref`, `ref mut` and `const ref mut` stand before the
  value they act on and bind as a prefix `-` does: `snapshot = copy of items`, `other = shares mut
  items`, `cam.target = weakref player`, `Watcher(target=sharedref mut players[r])`,
  `boss = ref mut enemies[0]`, `bump(ref mut a)`, `blend(const ref mut d, s)`. What each one makes is
  Part VII's and Part VIII's; at a call they are the parameter's words (`[FN-1]`). `copy` is a
  keyword only directly before `of`, and `of` only directly after `copy`. `sharedref` is always
  followed by `mut`, before a value and in a type alike: alone it is `E0114` ("a link that only reads
  may be held by any number of names: write `target: ref Player`; `sharedref` is for a link that may
  change its target: `target: sharedref mut Player`"). `using` after a value names the instance of
  the memory manager it is made in (`owns Mesh(points=[0.0, 1.0]) using current`, `[0.0, 1.0] using
  current`, `Enemy(hp=500) using current`, `[TYP-46]`); it binds tighter than the words before a
  value, so the mesh is made in `current` and then handed to its place.
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
* `[GRM-27]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* A comprehension is shorthand for an iterator pipeline and has exactly its meaning and
  cost: `[f(x) for x in xs if p(x)]` is `xs.iter().filter(fn(x) => p(x)).map(fn(x) => f(x)).collect()`
  into an `Array`; `{…}` forms collect into a `Set` or a `Map`. Clauses nest left to right as in
  Python. The loop variables are scoped to the comprehension. A clause over `owns xs` takes the
  collection over, as `for` does (`[CTL-1]`): the items the comprehension keeps are handed over into
  the new collection, the rest end, and `xs` cannot be used afterwards (`[s for s in owns shapes if
  s.area() > 1.0]`). Over a collection read in place the items are read-only views, which `copy of`
  copies (`[copy of s for s in shapes if s.area() > 1.0]`), the collection staying as it was.
* `[GRM-38]` *(new in 0.9.9)* *(changed in 0.9.15)* A parenthesised comprehension `(e for x in it if c)` is a **generator
  expression**: the pipeline of `[GRM-27]` without the `collect`, a lazy `Iterator` that reads what it
  iterates in place and allocates nothing. As the only argument of a call it needs no extra
  parentheses: `sum(x * x for x in xs)`, `any(p.hp <= 0 for p in players)`. `(e)` without `for` is a
  parenthesised expression, and `(e,)` a tuple.
* `[GRM-39]` *(new in 0.9.9)* **Nesting depth.** Every compiler accepts expressions, statements,
  patterns and types nested 256 levels deep, counting each operand, argument, element, bracket, block
  and type argument as a level; an operator or method-call chain is as deep as it is long
  (`1 + 1 + … + 1` is left-nested). A compiler may accept more and states how much (this one, 1,024).
  A program nested more deeply than the compiler accepts is `E0112`, reported once where the limit is
  passed; the compiler never ends without a diagnostic (ODR-070).
* `[GRM-30]` *(new in 0.9.9)* *(changed in 0.9.15)* **Casts.** `cast s as Circle` gives the `Circle` that `s`, a value of an
  interface or base-class type, holds, and stops the program if it is not one (`[TYP-44]`); `cast` is
  a keyword only at the start of an operand directly before a name or `this` (`[LEX-15]`), and its
  operand is a postfix expression (`pool = ref mut cast this.pools[k] as Pool[T]`). `as?` and `as!`
  are lexed as single tokens (`[LEX-21]`) and are not Ember: either is `E1010`, whose message gives
  the kind test (`if s is Circle as c:`, `[GRM-45]`) for a downcast that may fail and `cast s as
  Circle` for one that stops the program; so are `cast s if Circle` and `cast s when Circle`.
* `[GRM-32]` *(new in 0.9.9)* `comptime(e)` evaluates the expression `e` at compile time (`[CT-6]`).
* `[GRM-36]` *(new in 0.9.9)* *(changed in 0.9.15)* `ref e`, `ref mut e` and `const ref mut e` name the place `e`: `ref e`
  as a read-only view of it, `ref mut e` as a name that may change it (`[TYP-14]`). The operand MUST be
  a place — a local, a field, an index, a dereferenced reference, a call whose result is `ref T` or
  `ref mut T` (`r = ref mut first_even(ref mut xs)`), a cast of a place, or, before a call that gives a
  `MutSpan`, that span's elements (`d = ref mut tracks[name].as_mut_span()`) — and `ref` of any other
  expression is `E0111`, whose help is to bind the value to a local first. `ref` binds like unary
  minus: `ref a.b[i]` names `a.b[i]`.

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
* `[GRM-13]` *(changed in 0.9.15)* A name a pattern binds in a place it does not take over is a read-only view of the
  part it names, as a name for a part is (Part VII): a number, a `bool` or a variant with no payload is
  copied, as Python binds a number, and nothing is changed through the name (`E3023`). `ref mut`
  before it makes a name for that part that may change it (`State.Walking(ref mut path):` then
  `path.pop()`); `mut` alone is `E0113` ("`mut path` says `path` may change but not whose list it is;
  to change that list through `path`, write `State.Walking(ref mut path):`"). `match owns e:` hands `e`
  over, and each name of the branch that runs owns its part.
* `[ENM-1]` *(changed in 0.9.9)* Inside a pattern whose scrutinee type is known, a variant may be
  written unqualified (`Circle(r)`); in an expression it is qualified (`Shape.Circle(1.0)`) unless
  imported.
* `[GRM-46]` *(new in 0.9.15)* **Clauses after a field's type.** `from` after a view field's type
  names what the view points into: a `let` field of the same object whose contents live in a block of
  their own (`words: Array[StrView] from text`), or `static`, text written in the program (`kind:
  StrView from static`) (`[TYP-15]`). After a function's result type it names the parameter, or the
  field of `this`, that the result points into (`-> StrView from src`, `[GRM-41]`). `guarded by` after
  the type of a field or a `static mut` names the `Lock` that must be held to read or change it
  outside `init` (`lines: Array[String] guarded by lock = []`; Part XI). `from`, `guarded` and `by`
  are keywords only in these places (`[LEX-15]`).
