# Helper AJ — every function and every lambda has a type of its own (ODR-278)

Branch `worktree-agent-aa2ff85845051a332`, from `main` at `3f2fce5b`.

## ADR draft (written before building) — function items

**Rules:** `[CLO-1]`, `[FN-6]`, `[CLO-3]`, `[CLO-10]`, `[EFF-1]`, `[EFF-2]`, `[TYP-5]`, `[TYP-23]`,
`[MNG-1]`, `[BLD-1]`, `[COST-3]`, ODR-102, ODR-278, D-877, D-901.

**Context.** A function or a capture-free lambda used as a value has today the shape type
`fn(A) -> R` (`stateless`), which says only "some function from `A` to `R`". So the compiler cannot
tell `add_one` from `make_list` (D-901: `@noalloc fast` refused because `make_list`, another
`fn(int) -> int` used as a value elsewhere, allocates), an adapter's instance is shared by every
lambda of one shape and calls through a 40-byte callable value (`struct ember_fn_value`: entry,
descriptor, three words), and the call goes through a thunk to the lambda. A capturing lambda
already has a type of its own (its environment struct, `closure_calls`), called directly.

**Decision.**

1. *The type.* `TyKind::FnItem { symbol, name, shape }`: the type of exactly one function — a
   function named as a value (`add_one`), an instance (`identity[int]`), a capture-free lambda or
   capture-free local function. `symbol` is the function's C symbol (an instance's carries its
   arguments' hash), which names it across builds as kept MIR names functions (`[BLD-1]`), so a
   kept body's item type is found again with no definition to look up; `shape` is the stateless
   `fn(A) -> R` it converts to (its `unsafe` too); `name` is how a diagnostic shows it.
   It holds nothing: size 0, `Copy`, nothing to drop, `Send` and `Sync`, not a view.
   Shown `fn(int) -> int {add_one}` (Rust's spelling: shape and identity), a lambda
   `fn(int) -> int {lambda}`. Spelled for `[MNG-1]` by its symbol (unique, stable).
2. *Where it stays itself:* a local declared from it, a generic argument inferred from it (so a
   generic is instantiated per function: `map(f)` is `Map[I, R, <f's item>]`), a lambda's
   inferred result, a tuple or `Option` built from it.
3. *Where it converts to its shape* (`[TYP-5]`, a new conversion): every coercion site whose
   expected type is a callable type — `fn(A) -> R` (ODR-102's owned callable value: a field, an
   annotated local, a return type, a collection's written element type, a `static`), `unsafe fn`,
   `@latebound fn`, `once fn`, `fn(…) + Send`, `extern "C" fn`. The conversion is the value
   today's function value is (`Const::Fn`, its native descriptor), so everything downstream of it
   is unchanged.
4. *When two meet* the result is the shape (a join), and the same item met twice stays the item:
   the branches of a conditional expression, the arms of a `match`, a generic parameter solved by
   two arguments (`pick(add_one, sub_one)`), and a local the body assigns again (`if c: f = a
   else: f = b`, or `f = a` … `f = b`: found by the same scan of the body's assignments `[CLO-13]`
   uses). A list, set or map written out holds the shape: a collection is for holding any
   function of its shape (as `[TYP-38]` makes a literal's text owned). Effects of a value of the
   shape are D-877's (the functions converted to that shape).
5. *Calls.* A call through a value of item type is a direct call to its function
   (`FuncRef::Direct`; the zero-sized value is still evaluated): its effects are exactly that
   function's (`[EFF-1]`, `[EFF-2]`), an analysis of it is exact, and the C compiler can inline
   it. A call through the shape stays indirect, and D-877's resolution by shape stays for it only
   — now more precise too, since an item never converted is not counted for its shape.
6. *Capturing lambdas:* unchanged. Their type is already their own (the environment struct), and
   a call through one is already direct; the environment is the value, so an "item plus
   environment" would add nothing.
7. *Backends.* An item is laid out and passed as `void` is: the C backend's `void` (one byte as a
   struct member, as a `void` field is: C has no zero-sized member), the LLVM backend's `Zst`, the
   evaluator's empty value. MIR: an item value is `Const::Void`; its conversion to the shape
   `Const::Fn(symbol)` (as `FnValue` is today); a call through it `FuncRef::Direct { symbol }`
   (the rewrite `call_known_callables` (ADR-169) already proves sound: same arguments, same
   function).
8. *Effect analysis.* Exact for item calls by construction (direct calls); D-877's
   `plain_function_values` unchanged, serving only values typed as the shape.
9. *Incremental builds.* `KindRecord::FnItem { symbol, name, shape }`; found again by interning.
10. *Hot reload (`[HR-*]`).* A call through an item is a direct call, which a reloadable image
    sends through the function's slot like any direct call; an item value holds no address, so a
    reload has nothing to re-point. A conversion to the shape is `Const::Fn` as today.
11. *Code size.* A generic is instantiated per function given to it (one `Map` instance per
    lambda an adapter holds, was one per shape), each instance's call direct and inlinable — the
    C++/Rust trade. Measured below.

**Spec text the lead writes** (proposals): `[FN-6]` "A function is a value of a type of its own,
naming exactly that function (zero-sized), which converts to any callable type it satisfies";
`[CLO-1]` "a lambda … has a unique anonymous type" (now built for capture-free lambdas: a function
item); `[CLO-3]` (a local declared from a function holds that function's type unless assigned
again); `[TYP-5]` a new rule: a function item converts to its callable type where one is expected;
`[TYP-23]` the joins of decision 4 and the collection rule; `[EFF-2]` "a call through a value of a
function's own type contributes that function's effects"; `[MNG-1]` an item's spelling;
`[COST-3]` a call through a function item is a direct call; `[CLO-10]` a function item holds
nothing (zero bytes).

## Done

## Not done

## Compiler changes

* `ember_types`: `TyKind::FnItem { symbol, name, shape }` and its facts (layout zero, `Copy`, no
  drop, `Send` and `Sync`, not generic unless its shape is, display `fn(A) -> R {name}`, spelling
  and canonical name `fn{symbol}`); `TypeTable::item_shape`, `TypeTable::join_items`; the unifier
  joins two items of one shape at a parameter and solves a written `fn(…)` from an item's shape;
  `callable_sources` reads an item as its shape. `codec`: `KindRecord::FnItem`.
* `ember_typeck`:
  * the item made where a function becomes a value (`fn_value_of`: a free function, an instance
    `f[T]`) and where a capture-free lambda or local function is made (`synth_lambda_fresh`),
    named by its symbol (`item_symbol`: a lambda's `closure_symbols`; a free function's
    `function_symbols`, now computed from its declaration when its signature is collected by
    `free_function_symbol`, which the body's check uses too; an instance's from
    `generic_paths` and its arguments' spelling); a foreign declaration keeps its C pointer type;
    `item_defs` maps each item back to its function (debug-asserted equal to the emitted symbol);
  * conversion to the shape (`item_to_shape`: the `FnValue` itself, or a `Cast` of any other
    expression) at every coercion to a callable type (`coerce`), in `[TYP-5]` rule 11
    (`Option[fn]`), an export table's fields, a comparison (`==` beside a callable value);
  * joins: a conditional expression, `match` arms (their shape), a list literal and a collection
    literal's elements, an open `[]` fixed by a push, a local declared from a function and assigned
    again (`reassigned`, from `[CLO-13]`'s assignment scan, which now also sees destructuring
    targets), a destructuring declaration likewise, an open `None` fixed by a function;
  * calls through an item: `synth_indirect_call` takes the shape's signature, every callable-value
    test (`is_callable_value`, `through_callable_ref`, `callable_signature_of_value`, the bound
    unifier) sees items;
  * a closure that names an enclosing variable holding an item captures nothing for it: a local of
    its own is set to the function where its body begins (`CaptureWatch::items`);
  * diagnostics: two items that differ (`E2020`, "each function has a type of its own", help: the
    shape); messages about a callable's signature show an item's shape (`callable_shown`);
  * thread, job, scope and FFI-callback checks treat an item as holding nothing; an FFI callback
    given an item calls its function from the trampoline (`FfiCallback::closure`).
* `ember_mir`: an item value is `Const::Void`; its conversion `Const::Fn(symbol)`; a call through
  it `FuncRef::Direct { symbol }` (the item still read: `keep_item_read`).
* `ember_codegen_c`: an item is `void` (`c_type`, `is_void`; a member is one byte, as a `void`
  member is); a callback record's member; the trampoline calls an item's function with no state.
* `ember_codegen_llvm`: an item is `Repr::Zst`; the callback trampoline as the C backend's.
* `ember_interp`: an item is void (`is_void`).
* `ember_analysis`: the cycle analysis reads an item as owning nothing. Effects need no change:
  item calls are direct calls.

## Measurements

## Defects found

## Spec questions

1. *A local assigned two functions* (`if c: f = a else: f = b`, `f = a` … `f = b`). ODR-278 names
   the conditional expression (the shape); a local assigned again is not named. Options: (a) the
   local holds the shape, decided from the body's assignments as `[CLO-13]` decides "assigned
   after" (built: Python-like, and every program that compiled before compiles); (b) the first
   assignment fixes the item and the second is `E2020` (Rust's rule, and D-555's for capturing
   lambdas, whose join would need an owned callable value). Recommend (a): the join costs nothing
   (a function pointer, as before), (b) breaks working programs for no gain.
2. *A collection written out* (`[add_one, sub_one]`, `{"a": f}`, `xs = []; xs.push(f)`): built as
   the shape always, even for one function (`[add_one]` is `Array[fn(int) -> int]`), since a
   collection is for holding any function of its shape (a list of one function's values could
   hold no other: `push(sub_one)` would be `E2020`). Option: the item when every element is the
   same function. Recommend as built.
3. *`match` arms giving functions* are built as the shape (the first arm settles the type and the
   others are checked against it, so a mismatch points at the arm that differs); a conditional
   expression keeps the item when both branches are the same function. Option: join arms as the
   conditional does (two passes over the arms). Recommend as built (rare; no cost).
4. *How a diagnostic shows a function's type*: built `fn(int) -> int {add_one}` (Rust's: the
   shape and the one function), a lambda `fn(int) -> int {lambda}`; messages about a callable's
   signature (`cannot take`, `takes a function of no arguments`, `calling a value of type`) show
   the shape only. The owner's words were "is of type `add_shield`". Recommend as built.
5. *Foreign declarations* (`extern "C"` functions declared, not defined) keep their C function
   pointer type as their value's type (no item): their symbol is decided by their contract (a
   wrapper's, or C's), known only once the declaration's wrapper is planned, and their value is
   only ever given to C. Every function Ember defines (`extern "C" fn` definitions and `@export`
   functions too) has its item. Options: (a) as built; (b) items for them as well, the contract's
   symbol planned with the signature. Recommend (a) for now: no program gains from (b).
6. *A variable holding a function item, assigned inside a local function or a lambda* (the
   body's assignment scan does not look inside closures): `E2020`, "each function has a type of
   its own", whose help names the shape (`CLO-1/reject_a_place_holding_one_function_holds_no_
   other.em`). Option: extend the scan into closure bodies (the variable then holds the shape).
   Recommend leaving it (rare, and the help gives the fix).
