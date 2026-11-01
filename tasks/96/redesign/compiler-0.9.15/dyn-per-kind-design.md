# A `dyn` or `virtual` call checked by the code of each kind (verdict 151)

Spec: `[LT-1]` ("A `dyn` or `virtual` method's call is decided by the code of each kind that may reach it,
the types the program gives to the function, followed through its names and calls") and `[LT-48]`
(`tasks/spec-0.9.9/parts/p07-ownership.md`); verdicts 151 and 152 in `tasks/96/redesign/JOB.md`.

## Today

`compiler/ember_analysis/src/borrows.rs`, `legacy_elision`: `FuncRef::Virtual { .. } => Elision::Everything`
and `FuncRef::Interface { .. } => Elision::Everything`. The result of every `dyn` or virtual call points
into every argument, and the call may touch everything each argument reaches (`CallAccessContract::All`).
So the owner's `show(Mine(items=[9]))` is refused at `xs.push(owns 3)`, as `show(Yours(items=[9]))` is.

## What is built

### 1. Which kinds reach a value (new file `compiler/ember_analysis/src/dyn_kinds.rs`)

Inside each function, every local whose type carries a kind (a `dyn I`, a `Box[dyn I]`, an interface
handle of a class, a class handle, or a `ref` to one of these) gets the set of places its value may come
from:

* `Kind(k)`: where the value is made. A `ref dyn I` made from a concrete value
  (`CastKind::InterfaceUpcast`), an interface handle made from a class (`ClassInterfaceUpcast`), a
  `Box[dyn I]` made from a value (`FuncRef::DynBoxNew`), a class object made by its constructor
  (`Builtin::ClassNew`, the class it was made as);
* `Param(j)`: the function's parameter `j`;
* `Unknown`: anything not followed: a field (`h.p`), a list's element (`ps[0]`), the result of a call
  through a value or through `dyn`, a value with no body (C).

Names are followed: a copy, a move, a `ref` of the value or of the box holding it (`ref b` of a
`Box[dyn I]` is `&(*b.0)` in MIR), a class upcast. Calls are followed both ways: what a direct call
returns is what its callee's body returns, with the callee's `Param(j)` replaced by what the call passes
it (a fixpoint over all bodies, for `fn make() -> Box[dyn Picker]: return Box(Mine(items=[9]))`).

```ember
fn main():
    m = Mine(items=[9])
    show(ref m)                                  # show's p: {Mine}
    q: Box[dyn Picker] = Box(Mine(items=[9])) if input() == "m" else Box(Yours(items=[9]))
    show(q)                                      # show's p: {Mine, Yours}
```

The analysis is flow-insensitive inside one function (a local holds the union of everything assigned to
it anywhere in the function).

### 2. Each function checked once for each set of kinds its callers give it

Like a generic checked per instance: an **instance** of a function is the set of kinds of each of its
`dyn`/class parameters. Instances are found from the top down:

* a function no direct call reaches (`main`, a function only called from outside this build) and a
  function called where the call is not followed (a method in an interface table, a virtual method, a
  lambda, a function used as a value: `stores::dynamic_bodies`) has the instance "every parameter
  `Unknown`";
* each direct call inside an instance of the caller gives the callee the instance made of what each
  argument holds there (`Param(j)` of the caller replaced by the caller instance's set).

So `fn outer(p: dyn Picker): show(p)` called as `outer(ref m)` and `outer(ref y)` gives `outer` and
`show` the instances `{Mine}` and `{Yours}`, each known when compiling; `show(q)` above gives `show` the
instance `{Mine, Yours}`, known only while running.

For each instance, each `dyn` call `p.pick(xs)` is resolved to the methods it may run: for each kind of
the receiver, the method its table holds in the slot (for a class concrete, also every override of that
method below the class). For a virtual call `a.first(xs)` on a class value: the body the object's class
runs (the nearest definition at or above the class it was made as); where the class value is `Unknown`,
the body of every class at or below the call's class in the build (`[DSP-5]`: the whole program is in
the build).

The function is then checked once for each different resolution (instances that resolve every call the
same way are one check): a copy of its MIR body in which each resolved call is a direct call to a
compiler-made symbol whose contract is the **join** of the methods' summaries (`[LT-22]`): what the result
points into is the union of what each method's result points into, and what the call touches is the
union of what each touches. A call whose receiver is `Unknown` keeps today's worst case. The diagnostics
of the checks are reported once each (same code, place and message).

Where the methods' results have different shapes (one exact summary, one with none), the joined result
is the worst case, every argument, as today.

With one kind (`{Mine}`), the join is `Mine.pick`'s summary: `result <- arg0` (`this`), so `xs` is not
kept viewed and `show(ref m)` compiles. With `{Yours}`, it is `result <- arg1` (`other`): `xs` stays viewed
while `r` is used, and `xs.push(3)` is `E3021`.

The receiver's part of a method's summary (argument 0) is widened to the whole receiver: the paths in a
method's summary are paths in its own concrete type (`this.items` of `Mine`), which the `ref dyn Picker`
receiver of the call does not have. Today's worst case is the same for argument 0.

### 3. `E3021` where the kind is known only while running

When a check reports `E3021` for a change (a write, or a `ref mut` view taken, as `xs.push` takes one)
of a place whose read-only loan is held by an argument of a resolved call with more than one kind, and
some of those kinds' methods' results point into that argument and others' do not, the message names the
kinds that make the line unsafe, as `[LT-1]` gives it (`borrows.rs`, `check_point`; the loan's argument is
found by the local the loan is held in, which the call is given):

```
error[E3021]: `xs` changes while `r` may still look at it: `p` may be a `Yours`, and which kind it is is only known while running
  = help: to check it while running, write `runtime` on the parameter: `fn show(p: runtime dyn Picker):`
```

The help is given where the call's receiver is a parameter whose type is `dyn I` (or `ref dyn I`) and
whose kinds are more than one, or not followed. Where every method makes the line unsafe, the message is
today's: the line is unsafe whatever the kind.

Where the receiver is not followed (the worst case), the message is today's and the help above is given
when the receiver is a `dyn` parameter:

```ember
fn main():
    ps: Array[Box[dyn Picker]] = [Box(Mine(items=[9]))]
    show(ps[0])          # show's p: Unknown (a list's element is not followed)
```

### 4. Callers read each instance's own summary (`[LT-22]`)

A function whose result comes from a `dyn` call has, in each instance, the summary of its form there:

```ember
fn via(p: dyn Picker, xs: Array[int]) -> Span[int]:
    return p.pick(xs)

fn main():
    m = Mine(items=[9])
    xs = [1, 2]
    r = via(ref m, xs)
    xs.push(3)        # compiles: via, given a Mine, gives a result pointing into p only
    println(r)
```

Each form of a body is a copy with a symbol of its own (`<symbol>\u{1}kinds <digest of what its calls are
made direct to>`), its summary inferred by `infer_callable_summaries` from the summaries of what it calls
(the bodies' installed ones, the joined contracts, and the other forms'). A call in a caller's form is
made direct to the form of the instance it gives its callee where that form's summary is not the callee's
own; the caller then has a form of its own, whose summary may differ from its body's, and so up the call
chain (`dyn_kinds::expand`, a fixed point over the set of calls made direct, which only grows; at most
`MOST_ROUNDS` = 16 rounds, past which a caller reads the body's summary, the worst case). A form stores
what its body does (its store summary is the body's). A form's summary is inferred from the forms alone,
so a closure or a receiver's fields read by a form are the worst case there.

### 5. Incremental builds (`[BLD-1]`)

A body's forms read what their calls are made direct to: for each `dyn` or virtual call, each method it
may run with a digest of that method's own summary (which decides an error or a run-time check, 6), and
for each call made direct to a callee's form, the form with a digest of its summary. That list is
`Expansion::checked_under`, returned as `BorrowFacts::kinds` for each body with forms; the module record
keeps it (`compiler/ember_driver/src/incremental.rs`, `Form::kinds`, schema 5). A kept body is checked
again when the list the program now gives it is not the list its record kept; only the errors of that
check are new (the build that kept it reported no error, and its warnings are its record's). Tests:
`compiler/ember_driver/tests/incremental.rs`, `a_kept_function_is_checked_again_for_a_kind_a_changed_module_gives_it`
and `a_kept_function_is_checked_again_when_a_kinds_method_changes`.

### 6. `runtime` on a parameter (`[LT-48]`)

`fn show(p: runtime dyn Picker):`. The type checker gives `hir::Program::runtime_params` (function,
position); lowering puts each function's positions on `Body::runtime_params`.

Where a form's `dyn` call's receiver is a `runtime` parameter's value and nothing else (its only origin
is that parameter), whose kinds in that instance are more than one (or not followed: then every kind the
program makes of the table), and a change of a place only the call's result keeps viewed (the loan's
holder is a temporary nothing but the call reads) is unsafe for some of them (`E3021`, as in 3), the
borrow check reports no error and asks for a check (`dyn_kinds::Guard`) instead: one for each kind that
makes the line unsafe, at each point the line is lowered at (a `defer`'s body, at each exit), with the
message

```
`xs` changes while `r` looks at it: `p` is a `Yours`, whose `pick` gives back the list it was handed
```

(what the method gives back is named by the argument's type: a list (`Array`, `Span`), text (`str`), or
the type's name; "what it holds" for the receiver). Where one of those kinds cannot be told by its value's
table (a class's override), or no name holds the result, the error stays. Where the kind is known when
compiling (`show(ref y)` with `y` a `Yours`), the word changes nothing: `E3021`.

A value is followed only while nothing may replace its kind through a view of it: a local a `ref mut` is
taken of (`mem.swap(ref mut a, ref mut b)`, a `ref mut` parameter's argument, a closure's capture), or
one written through a reference or its box (`*q = Box(…)`), is not followed (1). A `ref mut` of the `dyn`
value itself (a `ref mut this` method) cannot replace it.

The checks go in after the borrow checks, before the coroutine transform (`coroutines::transform` calls
`insert_kind_checks_all` first, so a generator's body has them before it becomes its frame's state
machine): the borrow checks of the bodies with a `runtime` parameter are made again (an error there is an
internal compiler error: they must agree with the program's), their checks gathered (once per point and
kind), and before each guarded point:

```
_g = DynKindIs { concrete: Yours, interfaces: [Picker], .. }(copy _p) as bool
assert(!_g, "`xs` changes while `r` looks at it: ...") -> next
```

`CastKind::DynKindIs` (ember_mir) is one compare of the kind the value's table holds: every `dyn` table
ends with a word, `kind_identity` (the 64-bit FNV-1a hash of the concrete type's canonical name), the same
in every unit, since each unit has its own copy of a table (C: `->kind == UINT64_C(…)`; LLVM: an `i64`
load and `icmp eq`; the evaluator: the table's id, its tables being one program's). The panic is
`panic at main.em:17:5: <message>`, at the guarded line.

### Where it lives

* `compiler/ember_analysis/src/dyn_kinds.rs`: the kinds, the instances, the resolutions, the joined
  contracts, and the text of the help.
* `compiler/ember_analysis/src/borrows.rs`, `run_checks`: each body is checked in each of its forms; the
  joined contracts and the forms' summaries are added to the summaries the checks read; `check_point`
  gives `E3021` the message and help above, or asks for a `runtime` check; `insert_kind_checks_all`
  makes the checks.
* `compiler/ember_mir`: `CastKind::DynKindIs`, `Body::runtime_params`; `compiler/ember_codegen_c`,
  `compiler/ember_codegen_llvm`, `compiler/ember_interp`: `DynKindIs`.

Not changed: the summaries installed on MIR for the interface artifacts (`install_callable_regions_kept`)
stay as today (the worst case for a `dyn` call); `regions_all` and the effect summaries are unchanged.

### Cost to compile time and memory

* One pass over each body's statements per round of the return fixpoint (usually 2 rounds), only for
  bodies with a local of a kind-carrying type; one walk of the direct calls per instance.
* One extra borrow check per extra resolution of a body with a `dyn` or virtual call, and one copy of
  that body's MIR while it is checked. A body is checked for at most 8 instances; past that, the rest are
  checked as one, with the worst case for parameters (`MOST_INSTANCES`).
* No cost for a program with no `dyn` or virtual call (the pass returns at once).

## Not built (limits)

1. **Callers in another package** (the interface artifacts) read a function's installed summary, the
   worst case for its `dyn` calls.
2. **Kinds through a `dyn` call's arguments**: a method reached through a table (`fn take(q: dyn Q)` as
   an interface method) is checked with its `dyn` parameters `Unknown`.
3. **Lambdas** are checked with their `dyn` values' calls resolved only from their own locals and the
   worst case for their parameters (they are called where nothing is followed).
4. **`runtime` on a class's override**: a kind that runs a class's override cannot be told by the value's
   table; such a line stays `E3021`.
5. **A view named and used again** (`v = ref xs`, `r = p.pick(v)`) is no run-time check and gets no
   "may be a `Yours`" message, even where only `r` is used after the change: the call's result is taken
   to keep the loan only where its holder is a temporary nothing else reads.

## Open questions (the spec does not settle them)

1. **A function no call reaches**, in a program with `main`:
   ```ember
   fn show(p: dyn Picker):
       xs = [1, 2]
       r = p.pick(xs)
       xs.push(owns 3)
       println(r)

   fn main():
       println("hi")
   ```
   No kind reaches `show`. Built: its parameter is `Unknown`, the worst case (refused at `xs.push`, with
   the `runtime` help), as for a `pub` function called from another package.
2. **A value not followed** (a list's element, a field): the lead's direction is the worst case with the
   `runtime` help. `[EFF-2]` (verdicts 122, 128, 129) follows function values through fields and lists,
   and a `dyn` call's effects are those of every implementation the program's casts put in the table.
   Should `[LT-1]` follow kinds the same way (`ps: Array[Box[dyn Picker]] = [Box(Mine(items=[9]))]`,
   `show(ps[0])` would then compile)?
3. **The message where the receiver is not followed**: `[LT-1]`'s message names a kind ("`p` may be a
   `Yours`"); with no kind followed there is none to name. Built: today's message
   ("`xs` is borrowed here and mutably borrowed elsewhere") with the `runtime` help.
4. **Several kinds make the line unsafe**: built "`p` may be a `Yours` or a `Theirs`". The spec shows one.
5. **The help's first line for a function with more parameters** (`fn show(p: dyn Picker, n: int):`):
   built `fn show(p: runtime dyn Picker):`, naming the one parameter, as the spec's example does.
6. **A virtual call on a class value whose class is known only while running**:
   ```ember
   open class A:
       let items: Array[int] = [7]
       virtual fn first(other: Array[int]) -> Span[int]:
           return this.items

   class B(A):
       override fn first(other: Array[int]) -> Span[int]:
           return other

   fn show(a: A):
       xs = [1]
       r = a.first(xs)
       xs.push(owns 2)
       println(r)
   ```
   with `show(B())`: refused (`E3021`), known when compiling. With an `a: A` whose class is chosen while
   running, built: "`a` may be a `B`, and which kind it is is only known while running", with no help:
   `runtime` is a word only before `dyn` (`[LEX-15]`), and `a: A` has none.
7. **A local whose kind is chosen while running inside the function** (no parameter):
   ```ember
   fn main():
       q: Box[dyn Picker] = Box(Mine(items=[9])) if input() == "m" else Box(Yours(items=[9]))
       xs = [1, 2]
       r = q.pick(xs)
       xs.push(owns 3)
       println(r)
   ```
   Built: "`q` may be a `Yours`, ..." with no help (`runtime` goes on a parameter).
8. **A library's open class** subclassed by a package outside the build: a virtual call whose class
   value is not followed takes every class at or below the call's class in the build (`[DSP-5]`).
