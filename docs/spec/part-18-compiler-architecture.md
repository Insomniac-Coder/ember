# Part XVIII — Compiler Architecture

## XVIII.1 Overview

```
 .em files ─► Lexer ─► Parser ─► AST ─► Name resolution ─► Type checking/inference ─► HIR (typed, desugared)
                                                                                         │
                          ┌──────────────────────────────────────────────────────────────┘
                          ▼
   HIR ─► MIR lowering ─► Definite-init ─► Borrow check (NLL) ─► Drop elaboration
                                                                        │
                          ┌─────────────────────────────────────────────┘
                          ▼
   Monomorphisation ─► MIR optimisations (RC elision, inlining of tiny fns, SROA, const-prop, bounds-check elim,
                       exclusivity-check elision) ─► Effect analysis (under the contract profile, `[EFF-15]`)
                          │
            ┌─────────────┴─────────────┐
            ▼                           ▼
     C11 backend (v1)             LLVM backend (v2)
            │                           │
     project C compiler           LLVM opt + codegen
            └─────────────┬─────────────┘
                          ▼
                      linker (lld / link.exe / ld)  + ember_rt.lib
```

Effect analysis runs **after** monomorphisation and after the target-independent subset of MIR optimisation, because `[EFF-9]`'s `RuntimeCheck(k)` describes generated code: a check that elision removes must not appear in the effect set. It runs under the **contract profile** (`[EFF-15]`), so a contract has one verdict for a given source rather than one per build profile.

`[CMP-1]` The compiler is a Rust workspace (`emberc`). Each stage is a crate with a documented input/output type and a `--emit=<stage>` flag so intermediate representations can be dumped and snapshot-tested. `[CMP-2]` No stage after parsing may report a diagnostic without a source span.

### Crate layout

```
compiler/
  ember_span         FileId, Span, SourceMap (line/column mapping, UTF-8 aware)
  ember_diag         Diagnostic model, rendering (ariadne-style), error-code registry (Part XIX §6), JSON output
  ember_lexer        tokens, indentation algorithm, literal decoding
  ember_ast          AST types (§2), visitor, pretty-printer (used by the formatter)
  ember_parser       recursive descent + Pratt; error recovery; produces AST + parse diagnostics
  ember_resolve      module graph, scopes, DefIds, import resolution, IndexOrInstantiate disambiguation
  ember_types        Ty interner, TypeInfo table, layout computation, unification, interface obligations
  ember_typeck       bidirectional type checking of function bodies → HIR
  ember_hir          HIR types (§3)
  ember_mir          MIR types (§4), lowering from HIR, verifier
  ember_analysis     definite-init, NLL borrow checker, drop elaboration, effects, exclusivity analysis
  ember_mono         monomorphisation, instantiation cache, drop-glue synthesis
  ember_opt          MIR-level optimisations
  ember_interp       comptime MIR interpreter
  ember_ffi          libclang import (clang-sys), BIR, overlays, .embind read/write, C++ thunk generation
  ember_abi          C ABI classification for the LLVM backend; C-backend type mapping
  ember_codegen_c    MIR → C11
  ember_codegen_llvm MIR → LLVM IR (inkwell)             [v2]
  ember_build        ember.toml, lockfile, build graph, caching, CMake file-API reader, C/C++ toolchain driver
  ember_driver       `ember` CLI
tools/
  ember_fmt, ember_lint, ember_lsp (v2), ember_shader_bind
runtime/
  ember_rt/          C11 runtime (§9)
std/                 standard library in Ember
tests/               conformance suite (Part XIX §5)
```

## XVIII.2 AST

The AST is a faithful, span-carrying tree. Node kinds (Rust enum names given; fields abbreviated):

```
Item      = Fn(FnDecl) | Struct(StructDecl) | Class(ClassDecl) | Enum(EnumDecl) | Interface(IfaceDecl)
          | Extend(ExtendDecl) | Const | Static | TypeAlias | ExternBlock | Comptime(Block) | Import(ImportDecl)
FnDecl    { attrs, vis, is_unsafe, dispatch: None|Virtual|Override, name, generics, params: Vec<Param>, ret: Option<TypeExpr>,
            where_: Vec<Bound>, body: Option<Block>, doc }
Param     { mode: Borrow|Mut|Owned, pat: Ident|SelfKw, ty: TypeExpr, default: Option<Expr> }
TypeExpr  = Path{segments, generic_args} | Ref{mutable, inner} | Ptr{mutable, inner} | Tuple(Vec) | Fn{abi, params, ret}
          | Dyn(bounds) | Array{elem, len: Expr} | SelfTy | Void | Never | Infer
Stmt      = Decl{pat, ty, init} | Assign{targets, op, value} | Expr(Expr) | Return(Option<Expr>) | Break(label) | Continue(label)
          | Pass | If{...} | While{...} | For{pat, iter, body, else_} | Match{scrutinee, arms} | With{items, body}
          | Defer(Block) | Unsafe(Block) | Comptime(Block) | Labeled{label, stmt}
Expr      = Lit | Path | Field{base, name} | TupleField{base, idx} | Index{base, args} | IndexOrInstantiate{base, args}
          | Call{callee, args: Vec<Arg>} | MethodCall{recv, name, generic_args, args} | Unary | Binary | Logical
          | Ternary | Range{lo, hi, inclusive} | Cast{expr, ty} | TryOp(expr) | OptChain{base, name}
          | Lambda{owned, params, ret, body: Block|Expr} | MatchExpr | Tuple | ArrayLit | ArrayRepeat | FString{parts}
          | RefOf{mutable, place} | SelfExpr | Paren
Pattern   = Wild | Lit | Range | Bind{name, by_ref, mutable, sub: Option} | Path(path) | TupleStruct{path, fields}
          | Struct{path, named_fields, rest} | Tuple | Slice | Or(Vec)
```

`[AST-1]` Every node has `span: Span` and an `id: NodeId` (dense, per file) used by side tables (types, resolutions). `[AST-2]` The parser recovers at statement boundaries (skip to next `NEWLINE` at the current indentation) and at item boundaries; it never produces fewer than one diagnostic for a malformed region and never a cascade of more than 3 for one region (tested by the `parser/recovery` suite).

## XVIII.3 HIR

HIR is the AST after name resolution, type checking and desugaring. Differences from AST:

* All names are `DefId`s or `LocalId`s; paths are resolved.
* Every expression carries its `Ty`.
* Desugared: `for` → `while` + iterator calls (`[CTL-1]`); operators → interface method calls (scalars keep intrinsic ops); `?` → `match`; `?.` → `match`; f-strings → `Formatter` calls; `with` → block + explicit drops; augmented assignment → method call or binary op; `elif` → nested `if`; ternary → `if`; named/default arguments → positional with default expressions inserted; auto-ref/deref adjustments made explicit (`Adjust::Borrow`, `Adjust::Deref`, `Adjust::Coerce(widen)`); method calls resolved to `Callee::Static(DefId, generic_args)` or `Callee::Virtual(slot)` or `Callee::Dyn(iface, slot)` or `Callee::Closure`.
* Patterns are compiled to a **decision tree** (Maranget's algorithm) with exhaustiveness/redundancy results attached.
* Closures are lifted to synthetic struct types with a capture list `{local, mode: ByRef|ByMutRef|ByValue}`.

`[HIR-1]` HIR is the input to the comptime interpreter's MIR lowering and to the formatter's semantic lints. `[HIR-2]` `--emit=hir` prints a stable textual form used in snapshot tests.

## XVIII.4 MIR

MIR is a control-flow graph of basic blocks over **places** and **operands**, in the style of Rust MIR, with explicit borrows, moves, drops and RC operations.

### 4.1 Grammar

```
Body      := { locals: Vec<LocalDecl{ty, name, kind: Arg|Temp|User|Ret}>, blocks: Vec<BasicBlock>, arg_count }
BasicBlock:= { stmts: Vec<Stmt>, terminator: Term }
Place     := Local(LocalId) . Proj*           Proj := Field(i) | Index(LocalId) | ConstIndex(n) | Deref | Downcast(variant) | Column(field)   (Column: SoA)
Operand   := Copy(Place) | Move(Place) | Const(Const)
Rvalue    := Use(Operand) | Ref{mut, Place} | RawPtr{mut, Place} | BinaryOp(op, Operand, Operand) | CheckedBinaryOp(..)
           | UnaryOp | Cast(kind, Operand, Ty) | Aggregate(kind, Vec<Operand>)   kind = Tuple|Struct(def)|Enum(def, variant)|Array|Closure(def)
           | Len(Place) | Discriminant(Place) | NullaryOp(SizeOf|AlignOf, Ty) | ShallowInitBox
Stmt      := Assign(Place, Rvalue) | SetDiscriminant(Place, variant) | StorageLive(Local) | StorageDead(Local)
           | Retain(Operand) | Release(Place) | BeginAccess{place, kind: Read|Write, token: Local} | EndAccess(token)
           | FakeRead(Place) (* for borrowck of match scrutinees *) | Nop
Term      := Goto(bb) | SwitchInt{discr: Operand, targets: [(u128, bb)], otherwise: bb} | Return | Unreachable
           | Call{func: Operand, args: Vec<Operand>, dest: Place, next: bb, unwind: Option<bb>}
           | Drop{place, next: bb, unwind}      (* replaced by calls to drop glue during elaboration; kept in MIR for borrowck *)
           | Assert{cond: Operand, expected: bool, msg: AssertKind, next: bb}   (* bounds, overflow, div-by-zero, exclusivity *)
           | Panic{msg}
```

### 4.2 Invariants checked by the MIR verifier (`[MIR-*]`)

* `[MIR-1]` Every `Place` is well-typed; `Deref` is applied only to `ref`, `ref mut`, `Box`, raw pointers, or class handles (`Deref` of a handle yields the object type; field projection of an object requires the header offset added at codegen).
* `[MIR-2]` `Move(p)` appears at most once per path for `p` and no `Copy(p)` of a non-`Copy` type exists.
* `[MIR-3]` `Ref{mut}` of a place inside a class object is always bracketed by `BeginAccess/EndAccess` unless annotated `elided` by the exclusivity analysis.
* `[MIR-4]` Every block ends with exactly one terminator; every `Call` with an unwind edge is inside a function with `unwind` policy (v2).
* `[MIR-5]` `Retain`/`Release` appear only for class handles and `Shared`; MIR lowering inserts them at every handle copy and drop; `ember_opt` may remove pairs per `[RC-2]`/`[RC-3]`.

### 4.3 TypeInfo table

`ember_types` maintains, per interned `Ty`: `size`, `align`, `layout` (fields with offsets; enum tag placement and niche), `is_copy`, `needs_drop`, `is_view` (+ region parameter slot), `is_send`, `is_sync`, `is_zeroable`, `has_niche(Ty)`, `drop_glue: Option<DefId>`, `vtable(iface)`, `ffi_safe: bool`. Layout of `@layout(c)` structs is computed by the same algorithm the C ABI uses (natural alignment, trailing padding), and is cross-checked against libclang for imported types (`[FFI-5]`).

### 4.4 Type checking algorithm (`ember_typeck`)

1. **Collect** item signatures for the whole package (types, function signatures, interface impls, associated types) into `ember_types`. Cycles in struct definitions by value are `E2200 infinite size`.
2. **Per function**: allocate inference variables for each local declared without a type and for each generic argument at call sites; walk the HIR in **checking mode** where an expected type exists (`check(expr, ty)`) and **synthesis mode** otherwise (`synth(expr) -> ty`); unify with union–find; record coercions at coercion sites (`[TYP-5]`, `Array→Span`, `String→str`, handle upcast, `!`→any, closure→fn type); collect obligations `Ty: Interface`.
3. **Solve obligations** by searching impls (inherent + interface impls in scope, then blanket `extend[T: B]` impls) with unification; ambiguity is `E2062`; unsatisfied is `E2040`.
4. **Method resolution** per `[TYP-24]` with auto-ref/deref adjustments recorded.
5. **Default literals** (`[LEX-16/17]`), then **report** any unresolved variables (`E2060`).
6. **Pattern compilation** and exhaustiveness.
7. Emit HIR.

### 4.5 MIR lowering

Standard: expressions lowered to temporaries; places preserved; short-circuit ops to branches; `match` decision trees to `SwitchInt`; `with`/`defer` to explicit block structure; every local gets `StorageLive`/`StorageDead`; every non-`Copy` local gets a `Drop` terminator at scope exit on every path (including the `return` path); temporaries get drops at statement end; class-handle copies get `Retain`; class-handle drops get `Release`; class-field long-term accesses get `BeginAccess`/`EndAccess` (`[EXC-*]`); bounds checks become `Assert` before `Index` projections; overflow checks become `CheckedBinaryOp` + `Assert` when the profile/attribute asks for it.

### 4.6 Definite initialisation

Forward dataflow over MIR with a lattice per local (`Uninit | Init | Maybe`), also per field path for structs (partial moves/inits) and for `self` fields inside `init` methods (`[CLS-2]`). Reading `Uninit`/`Maybe` is `E3050`/`E2100`; `Maybe` at a drop point introduces a **drop flag**.

### 4.7 Borrow checking (NLL)

The algorithm is Rust's NLL (RFC 2094) restricted by the v1 lifetime rules:

1. **Regions.** Every reference/view-typed local, temporary and projection gets a region variable `'r`. A function's signature regions are per `[LT-1]`: `'self`, one `'p_i` per view-typed parameter, `'ret` = the elided/`@borrows` choice, plus `'static`. Struct view fields use the struct's single region.
2. **Constraints** are generated by walking MIR: assignment `a = b` of reference types yields `'b: 'a` (b outlives a, i.e. `points('a) ⊆ points('b)`); calls instantiate the callee's signature regions with fresh variables and add its constraints; `Ref{place}` creates a **loan** `L = (place, mut?, region)`; reborrows add constraints from the base reference's region.
3. **Liveness.** Compute the set of CFG points at which each local is live (used later on some path). A region `'r` includes every point at which any local with a type mentioning `'r` is live, and `[LT-*]` closure under constraints (fixpoint).
4. **Loan scope.** Loan `L` is **in scope** at point `P` iff `P ∈ points(region(L))` and `L` has not been **killed** (its base local was overwritten or went out of scope before `P` on that path).
5. **Access check.** At each statement, for each place `P'` accessed with kind `K ∈ {Read, Write, Move, ShallowWrite, Drop, BorrowShared, BorrowMut}`, for each loan `L` in scope with place `P` such that `P` and `P'` **overlap** (one is a prefix of the other, with `Index` projections assumed overlapping unless constant-disjoint, `Column` and `Field` projections disjoint when names differ, `Deref` of a class handle assumed overlapping with any other deref of the same class type — handled by exclusivity instead), report an error if `K` conflicts with `L.mut` per `[BRW-1]`. Two-phase borrows: a `Ref{mut}` marked `two_phase` is a shared loan until its **activation** point (the call).
6. **Region errors.** A loan whose region extends beyond the borrowed place's storage (`StorageDead`/`Drop` of a local while a loan on it is in scope) is `E3060 borrowed value does not live long enough`; a return of a reference whose region is not a subset of `'ret`'s allowed region is `E3062 returned reference does not derive from a parameter`.
7. **Diagnostics** name (a) the borrow site, (b) the conflicting access, (c) the later use that keeps the borrow alive ("borrow later used here"), and (d) a fix suggestion chosen from: shorten with a block, clone, use `split_at_mut`/`columns_mut`, use an index loop, use `Weak`.

The implementation is expected to be ~4–6k lines; the Rust compiler's `rustc_borrowck` is the reference for edge cases (drop-check, closures, two-phase, `match` fake reads). Polonius-style location-sensitive reasoning is not required for v1.

### 4.8 Exclusivity analysis

For class-object accesses, a lightweight pass over MIR: for each `BeginAccess(place=h.deref.f…)`, if all other `BeginAccess` on the *same handle local* (same `LocalId`, not reassigned in between) are statically ordered so that their intervals do not overlap conflictingly, mark it `elided`; overlapping accesses through the *same* local are `E3080` (compile-time, like a borrow error). Accesses through distinct locals keep the runtime check.

### 4.9 Drop elaboration

Replace `Drop{place}` terminators with: nothing (if `!needs_drop`), a call to the type's drop glue (a synthesised function that calls user `drop` then drops fields), a `Release` (handles/`Shared`), or a conditional on the drop flag. Partial moves produce per-field drops. This pass makes MIR ready for optimisation and codegen.

### 4.10 Effects

Per Part X: compute a bottom-up fixpoint over the call graph SCCs of the monomorphised program; store the effect set on each function instance; check contracts; produce chains for diagnostics. Before monomorphisation, generic functions are checked once with their bounds' declared effects.

### 4.11 Monomorphisation

Collect instantiation roots (`main`, `@export`s, `@test`s, statics); walk MIR bodies substituting generic arguments; instantiate on demand with a `(DefId, substs)` cache; synthesise drop glue, vtables (`dyn` and class), and closure bodies. `[MONO-1]` Instantiations are named deterministically (§8) so that separate compilation units dedupe at link time.

### 4.12 MIR optimisations (v1 set)

`RC pair elision` (`[RC-2]`), `SROA` (scalar replacement of `Copy` struct temporaries), `const propagation`, `copy propagation`, `dead-store/dead-code`, `bounds-check elimination` (range analysis for `for i in 0..len(a)` patterns — guaranteed by `[CTL-3a]`-style tests), `inline` of functions ≤ 8 MIR statements or marked `@inline`, `drop-flag elimination`, `tail-temporary merging`. All are optional for correctness; the C compiler/LLVM does the heavy lifting.

## XVIII.5 Comptime interpreter (`ember_interp`)

Executes MIR directly over an interpreter heap with typed allocations (each allocation knows its `Ty` and layout). Supports every MIR construct except `Call` into `extern` functions and raw-pointer deref outside interpreter allocations (`E6010`). Provides intrinsics: `size_of`, `align_of`, `offset_of`, `reflect`, `read_file`, `env`, `target()`. Results are converted back to `Const` values (including aggregate constants and byte strings) for embedding as statics. Step and memory limits per `[CT-3]`.

## XVIII.6 C backend (`ember_codegen_c`)

Emits one `.c` file per Ember module plus `ember_types.h` (all struct/enum/vtable definitions, topologically sorted) and `ember_decls.h` (prototypes).

**Type mapping** (`[CG-C-*]`):

| Ember | C |
|---|---|
| scalars | `int8_t … uint64_t`, `__int128`/`_BitInt(128)` (fallback struct on MSVC: `ember_i128` with helper ops), `float`, `double`, `_Float16`/`uint16_t` bit-pattern, `bool`, `uint32_t` (char), `size_t`/`ptrdiff_t` |
| `void` | `void` for returns; `ember_unit` (empty struct) as a value |
| `!` | `void` + `__builtin_unreachable()`/`__assume(0)` |
| struct/tuple/closure env | `struct em_<mangled> { ... }` with `_Alignas`; `@packed` → `#pragma pack(push,1)`/`__attribute__((packed))` |
| enum (unit) | `typedef <repr> em_<name>;` + `enum` constants |
| enum (payload) | `struct { tag; union { struct variant0; ... } u; }` (niche-optimised forms special-cased: `Option[ptr-like]` → the pointer) |
| `ref T` / `ref mut T` | `const T*` / `T*`; with `restrict` on locals proven disjoint (`[SIMD-3]`) |
| `Span`/`MutSpan` | `struct { const T* ptr; size_t len; }` / non-const |
| class handle | `struct em_obj_<Class>*` (object struct begins with `ember_obj_header`) |
| `Box[T]` | `T*` |
| `dyn I` ref | `struct { void* data; const em_vt_<I>* vt; }` |
| `[T; N]` | `struct { T a[N]; }` (so it is a value) |
| `fn` / `extern fn` | function pointer typedefs |
| generic instance | mangled distinct C type/function |

**Code shapes**: each MIR basic block is a C label; terminators become `goto`/`switch`/`return`/calls; `Assert` becomes `if (unlikely(!cond)) ember_panic_<kind>(file, line, ...)`; `Retain/Release` become inline functions from `ember_rt.h`; `BeginAccess/EndAccess` likewise; checked arithmetic uses `__builtin_*_overflow` on Clang/GCC and `ember_ck_*` helpers on MSVC; `@inline` → `static inline __attribute__((always_inline))`/`__forceinline`; `@cold` → `__attribute__((cold))`/`__declspec(noinline)`; SIMD via `ember_simd.h`.

**Debug info**: `#line` directives mapping every emitted statement to the Ember source; locals keep their Ember names where legal. `[CG-C-1]` The emitted C MUST compile warning-free under `-std=c11 -Wall -Wextra` (Clang/GCC) and `/W3` (MSVC), be free of UB by construction (no signed-overflow arithmetic without checks: wrapping ops use unsigned arithmetic and cast back), and not depend on compiler extensions except through `ember_rt.h` macros that have portable fallbacks.

`[CG-C-2]` The generated C is deterministic for identical input (stable ordering, no pointer-based hashing), so the build cache and `diff`-based review work.
* `[CG-C-4]` **Aliasing facts in emitted C.** `restrict` in C qualifies a pointer *object*; the pointer inside a `Span`/`MutSpan` is not such an object at the point of use. For every loop body and every `@simd` region, the C backend MUST hoist the base pointer of each view accessed in the region **whose base and length are loop-invariant across the region** into a local of type `T* restrict` / `const T* restrict`, and its length into a `size_t` local, before the region, and index those locals in the body. A view reassigned inside the body is not hoisted and receives no annotation. Two such locals MUST be declared `restrict` together only where `[SIMD-3]` establishes them disjoint.
* `[CG-C-5]` Every `ember_panic_*` declaration MUST carry `_Noreturn` and a cold marker (`__attribute__((cold))` on Clang/GCC; on MSVC, `__declspec(noreturn)` with the call placed in a basic block outside the loop body), and the C backend MUST emit the panic call in its own block reached by a forward branch, so the host compiler can sink it away from the fast path.
* `[CG-C-6]` For every loop in vectorisable form the C backend MUST emit the host compiler's vectorisation pragma immediately before it: `#pragma clang loop vectorize(enable)`, `#pragma GCC ivdep`, or `#pragma loop(ivdep)` (MSVC). These pragmas assert the absence of a loop-carried dependence, which is why `[SIMD-5]`'s single-constant-offset clause is a precondition rather than an optimisation.
* `[CG-C-3]` **Cross-translation-unit inlining.** Because `[BLD-1]` emits one translation unit per module, a call to a function defined in another module is opaque to the host C compiler unless the callee's definition is visible in the caller's translation unit. Extending `[BLD-1]`'s existing COMDAT-style mechanism from monomorphised instantiations to non-generic definitions, the C backend MUST emit, per package, an **inline header** `target/<profile>/c/<package>_inline.h`, included by every emitted `.c` file of that package and of every package that depends on it, holding a `static inline` definition of every function that is: (a) annotated `@inline`; (b) an impl of an operator interface (`Add`, `Sub`, `Mul`, `Div`, `Neg`, `Index`, `IndexMut`, `Eq`, `Ord`, `*Assign`, …) on a type whose `size_of` ≤ 64 bytes; (c) `len`, `is_empty`, `as_span`, `as_mut_span`, `iter`, `iter_mut`, `next`, a field accessor, or a `Deref`/`Iterator` method of a `std` view or container type; or (d) any other function whose MIR body after §4.12 is ≤ 40 statements and whose effect set does not contain `FFI`. A function emitted this way MUST NOT also be emitted with external linkage in its defining module unless it is `@export`ed or its address is taken, so `[MONO-1]`'s link-time dedup is unaffected.
* `[CG-C-3a]` `@inline` is **binding on the C backend**, not a hint: such a function MUST be emitted per `[CG-C-3]` and MUST carry `__forceinline` (MSVC) or `__attribute__((always_inline))` (Clang/GCC). Part III §7's table and the glossary entry for "Contract" are amended accordingly, resolving the existing contradiction with the `[CG-C-*]` code shapes. `@noinline` remains a hint.
* `[CG-C-3b]` A package MUST publish the MIR bodies of every function selected by `[CG-C-3]` in its build artefact, and `[BLD-2]`'s interface hash MUST cover them — `[BLD-2]` already names "inline bodies"; this rule fixes which bodies those are. A change to such a body invalidates dependent modules' codegen.
* `[CG-C-8]` **Step fidelity.** A `#line` directive MUST precede every emitted statement and MUST name the span of the *source construct that produced it*, never the declaration span of a place it mentions. All C statements lowered from one Ember statement MUST carry the same `#line`, so "step over" advances exactly one Ember statement. Desugared constructs (`for`, `?`, `?.`, `with`, f-strings, operator calls — XVIII §3) MUST attribute to the source syntax, not to the desugaring. **This requires MIR `Stmt` and `Term` to carry a source span**, and the MIR verifier checks that every statement has one.
* `[CG-C-7]` **Names.** Every MIR local carrying a user name MUST be emitted with that name as its C identifier, transliterated per `[MNG-3]`, suffixed `_<n>` only on collision with a C keyword, a reserved identifier, or another local in the same C scope. Parameters keep their Ember names; compiler temporaries keep `_<index>`. A profile MAY set `debug_names = false`; no default profile does. Deterministic naming keeps `[CG-C-2]`'s diff-based review intact.
* `[CG-C-9]` **Debugger visualisers.** `ember build` MUST emit, beside the binary, `target/<profile>/<package>.natvis` (passed with `/NATVIS:` on MSVC) and `<package>-gdb.py` / `<package>-lldb.py`, generated from the same `TypeInfo` table (§4.3) the backend already walks. They MUST render at minimum: `Option[T]` as `None`/`Some(v)` including every niche form; `Result[T,E]`; each payload enum as `Variant(fields)`; `String`/`str` as text including the SSO form; `Array`/`Span`/ `MutSpan` as `len` elements; `Box`, `Shared`, `Weak` as their pointee plus counts; a class handle as `Class { fields }` with the header hidden; `Handle[Tag]` as `index:generation`; `SoA[T]` as reconstructed `T` values.
* `[CG-C-10]` **Stacks.** `[RT-4]`'s panic output and `ember_backtrace_print` MUST print Ember function paths and Ember `file:line:col`, demangled from `[MNG-1]`'s scheme, never raw C symbols. The toolchain ships `ember demangle` (a stdin/stdout filter) so MSVC and GDB stacks can be read.

## XVIII.7 LLVM backend (v2)

Maps MIR to LLVM IR through `inkwell`: the same type mapping; `noalias`/`readonly`/`dereferenceable(N)`/`nonnull` attributes from the borrow checker's facts; `!nontemporal`, `!alias.scope` for `@simd` loops; DWARF/CodeView debug info with Ember type names; PGO via LLVM instrumentation; ThinLTO. It becomes the default when it passes the full conformance suite plus the performance suite (Part XX §4).

## XVIII.8 Name mangling

```
em_<pkg>_<module path with '_'>_<item>[__g<hash of generic args>][__v<vtable>]      e.g. em_std_math_Vec3_length, em_game_ecs_integrate__g3f2a1c
```

`[MNG-1]` Hash = first 12 hex digits of BLAKE3 of the canonical type string of the generic arguments. `[MNG-2]` `@export("name")` overrides the symbol entirely. `[MNG-3]` Identifiers are transliterated to ASCII (`_uXXXX_` for non-ASCII). `[MNG-4]` Class object structs are `em_obj_<mangled class>`; vtables `em_vt_<mangled>`; type infos `em_ti_<mangled>`.
* `[MNG-5]` The mangled prefix `em_` of `[MNG-1]`/`[MNG-4]` is derived from `EMBER_SYMBOL_PREFIX` and constructed in exactly one function.

## XVIII.9 Runtime ABI (`ember_rt`, C11)

Header `ember_rt.h`, ABI version macro `EMBER_RT_ABI = 1`. Everything below is `[RT-*]` normative.

```c
/* memory */
void* ember_alloc(size_t size, size_t align);            /* never returns NULL: panics on OOM */
void* ember_realloc(void* p, size_t old_size, size_t new_size, size_t align);
void  ember_free(void* p, size_t size, size_t align);
void* ember_try_alloc(size_t size, size_t align);        /* NULL on failure */

/* objects */
typedef struct ember_obj_header { uint32_t strong; uint32_t weak; uint32_t access; uint32_t flags; const ember_type_info* ti; } ember_obj_header;
static inline void ember_retain(ember_obj_header* o);               /* non-atomic or atomic per ti->flags & EMBER_TI_SYNC */
static inline void ember_release(ember_obj_header* o);              /* calls ember_rt_deinit when strong hits 0 */
void  ember_rt_deinit(ember_obj_header* o);                         /* runs ti->drop chain, drops fields (ti->drop_fields), then weak-release */
static inline void ember_weak_retain/release(...);
static inline ember_obj_header* ember_weak_upgrade(ember_obj_header* o);   /* NULL if strong == 0 */
static inline void ember_access_begin_read(ember_obj_header*, const char* what, ember_loc);  /* panics on conflict */
static inline void ember_access_begin_write(...);
static inline void ember_access_end_read/write(...);
void* ember_downcast(ember_obj_header* o, const ember_type_info* target);   /* NULL if not a subclass */

/* panics & diagnostics */
_Noreturn void ember_panic(const char* msg, size_t len, ember_loc loc);
_Noreturn void ember_panic_bounds(size_t index, size_t len, ember_loc);
_Noreturn void ember_panic_overflow(const char* op, ember_loc);
_Noreturn void ember_panic_div_zero(ember_loc);
_Noreturn void ember_panic_exclusivity(const char* what, ember_loc);
_Noreturn void ember_panic_unwrap(const char* what, ember_loc);
void ember_backtrace_print(void);

/* threads */
void ember_rt_thread_attach(void);   void ember_rt_thread_detach(void);
ember_thread* ember_thread_spawn(void (*f)(void*), void* arg, size_t stack_size);
int ember_thread_join(ember_thread*);
/* mutex, rwlock, condvar, atomics wrappers (C11 atomics / Win32 SRWLOCK), tls key API */

/* arenas, jobs (jobs in v1.1), time, env, fs, io: thin wrappers over the OS with UTF-8 paths on Windows */

/* runtime lifecycle & embedding */
typedef struct ember_rt_config { void*(*alloc)(size_t,size_t); void(*free)(void*,size_t,size_t); void(*log)(int,const char*,size_t);
                                 void(*on_panic)(const char*,size_t); uint32_t flags; } ember_rt_config;
ember_rt_config ember_rt_config_default(void);
int  ember_rt_init(const ember_rt_config*);   void ember_rt_shutdown(void);
uint32_t ember_rt_abi_version(void);

/* debug facilities (compiled out in shipping) */
void ember_debug_leak_report(FILE*);          /* live objects + cycles (intrusive live list enabled by EMBER_DEBUG_OBJECTS) */
void ember_debug_alloc_stats(ember_alloc_stats*);
```

* `[RT-1]` `ember_rt` has no dependencies beyond libc and the OS; `mimalloc` is vendored and used as the default allocator unless `cfg.alloc` is set.
* `[RT-2]` No global constructors; `ember_rt_init` is explicit (the generated `main` calls it) and idempotent.
* `[RT-3]` `ember_type_info` layout: `{ uint32_t size, align; uint32_t flags; const char* name; const ember_type_info* base; void (*drop)(void*); void (*drop_fields)(void*); const ember_vtable* vtable; const ember_itable_entry* itables; uint32_t itable_count; const ember_field_desc* fields; uint32_t field_count; }` — the reflection fields are present only for `@reflect` types.
* `[RT-4]` Panics print `panic at <file>:<line>:<col>: <message>` followed by a backtrace in debug/release, then call `cfg.on_panic` (if set) and `abort()`.
* `[RT-5]` Every runtime symbol, macro and header name is **generated** from a single build constant `EMBER_SYMBOL_PREFIX` (default `ember`), defined in exactly one place in `ember_rt` and one in the compiler. `ember_rt.h` is a **generation output** carrying literal identifiers, not a header of macro concatenations — it is the interface document C embedders read, and it must stay readable. No file in the compiler, runtime, CMake module, examples or test corpus may hard-code the symbol prefix, the CLI name, the manifest file name, or the source and binding-cache extensions; each is read from a single `branding` module. `tools/check_branding.py` fails CI on any hard-coded occurrence.

## XVIII.10 Compiler correctness strategy

* Snapshot tests per stage (`--emit=tokens|ast|hir|mir|mir-opt|c`) with `insta`.
* The MIR verifier runs after every pass in debug builds of the compiler.
* Differential testing: every `run-pass` test is executed through both backends (once LLVM exists) and through the comptime interpreter where applicable; outputs must match.
* Fuzzing: `cargo-fuzz` targets for the lexer, parser, type checker (well-typed program generator), and borrow checker (random mutation of accepted programs must either still pass or produce an error with a span).
* FFI layout tests: a generated C program asserts `sizeof`/`offsetof` for every type crossing the boundary and is compiled with each supported compiler in CI.
* Performance regression suite (Part XX §4) gates releases.

---
