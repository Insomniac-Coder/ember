# Gap map 01: ownership and hand-over (OWN, FN, MONO, ALC, ARN, HEAP, DRP, EXP, STA)

Agent 01, 2026-10-27. Read-only: nothing was built or run. The compiler is at `ab2b7b76` ("Move the
development target to spec 0.9.15_Hardened_1"); it still implements 0.9.10_Hardened_35.

**How this was made.** The 52 rules of these families that the H.41 table lists were extracted from both
spec trees with a script (`target/gap-0.9.15/01/extract.py`) and compared by hand. Every claim about the
compiler was checked with `rg` on the source; file:line is given for each. **All 201 old tests** in the
40 folders of these rules were **read by hand** (none classified by script). Design-test counts come from
`conformance-rule-map.md` (script `target/gap-0.9.15/01/map.py`), which maps 347 of 841 tests only by
their "main" spec ID and so names most of these rules zero times; where that is misleading, a content
count over `run4/v5/conformance/` (by `rg -l`) is given too.

**Overall.** Almost nothing of the new text is built: the compiler has the old model complete
(`owned` mode, `Box[T]`, Copy class handles, `Shared`/`Weak` with run-time access checks, `mem.forget`,
`alloc_nodrop`, no `static mut`), and none of the new words (`owns` as a type or before a value,
`copy of`, `using`, `this`, `static fn`, `ref mut` at a call). What *is* reusable is the machinery under
the words: the move/drop-flag dataflow (`ember_analysis/src/drops.rs`), the `Box` representation and its
codegen, the arena builtins and their region tracking, the first-access static initialiser, and the drop
sites of the effect pass.

## 1. Rule table

Status: `built`, `partly`, `not built`, `remove`, `text only`. Old tests: k = keep, s = respell,
w = rewrite, r = retire (file names of every non-keep in section 1b). Design tests: by the rule map;
`+n by content` where the map undercounts.

| rule | H.41 kind | what the spec now requires (one line) | today in the compiler | status | what must change | old tests | design tests |
|---|---|---|---|---|---|---|---|
| OWN-1 | changed | one owner for every type, a class object included, unless a line says `shares`/`thread_shared` | `ember_types/src/lib.rs:2122` and `:2658`: `TyKind::Class`/`ClassInterface` are `Copy` (a copy retains, `[OWN-7]` old); `ember_mir/src/lower.rs:7372-7375` moves only non-Copy | partly | class objects not `Copy`; `=` of a class name hands over (A1-9) | 2: k1 s1 | 26 (R1:17 R2:2 R3:2 R4:2 R11:1 R15:2) |
| OWN-2 | changed | drop at scope end/overwrite/statement end; nothing ends an owner without its clean-up; no `mem.forget` | drops built: `lower.rs:871-885` (`emit_drops_from`), `drops.rs:238` (`elaborate`); `mem.forget` exists: `ember_hir/src/lib.rs:2063`, `ember_typeck/src/lib.rs:16380`, `std/src/mem.em:97` | partly | remove `mem.forget` (A1-10); `fn drop():` spelling (A1-7) | 3: s3 | 0 (+32 with `fn drop` by content) |
| OWN-3 | changed | `b = a` of a whole owning name hands over; of a view makes a read-only view; E3040 message names `copy of` and `shares` | move dataflow and E3040: `drops.rs:1592-1745`; message "has been moved out of" (`drops.rs:1685`, `:1739`); help says `.clone()` | partly | view case of `=` (with BRW-12, borrow area); new E3040 text (A1-3) | 7: k1 s6 | 18 (R1:6 R2:3 R3:3 R4:2 R5:1 R12:1 R15:1 R16:1) (+17 with E3040 by content) |
| OWN-4 | changed | a hand-over in a loop not given back before the next round is `E3040` (E3041 retired) | `drops.rs:1643-1665` emits `E3041`; `codes.rs:302` | partly | fold E3041 into E3040, keep its loop wording (A1-3) | 3: s1 w2 | 0 |
| OWN-6 | changed | `mem.take/replace/swap(ref mut …)`, `mem.drop(owns x)` ends the name for every type, numbers included; `mem.forget` is E1010 with help | mem builtins in typeck (`lib.rs:16380` area); `mem.drop` of an int leaves it usable (vocabulary row `mem.drop`, probe `k3_mem_drop_int.em`); forget built | partly | `ref mut` at the call (A1-8); numbers end (A1-2); forget removed with E1010 text (A1-10) | 9: s6 w1 r2 | 0 (+9 `mem.drop`, +4 `mem.take` by content) |
| OWN-7 | changed | `Copy` = bit copy, holds no owner or link; class objects, `shares`, `weakref`, `ref mut`, `MutSpan` not Copy; `=`-copied Copy struct changed through the new name while the old is used is E3023 | `ember_types/src/lib.rs:2084-2140` `copy_with`: class handles Copy (2122), retain on copy | partly | class not Copy (A1-9); `@derive(Copy)` with a class field E2080; E3023 for a changed copy (borrow area, BRW-14) | 4: k2 w1 r1 | 0 |
| OWN-8 | changed | `copy of x` runs `Clone`; class object copied as a new object; links copied as links; every Copy type has Clone; E2116 for weakref, view type, no Clone, self-referential view | no `copy of`; `x.clone()` is an ordinary method; class `clone` copies the handle (vocabulary row `.clone()`); derives in `ember_typeck/src/derives.rs` | not built | `copy of` expression, E2116, `.clone()` at a use is E1010 (A1-5) | 14: k2 s11 w1 | 65 (R3:41 …) (+130 with `copy of` by content) |
| OWN-9 | added | `owns` hands over every type, numbers included: `f(n)` ends `n`; `b = owns n`; `return a`; E0109 narrowed to `owns` before a name whose place already hands over, or before `fn` | `owned` param of a Copy type copies (`lower.rs:7374`); E0109 is "`owned` in expression position" (`ember_parser/src/body.rs:1193-1207`, `codes.rs:174`) | not built | A1-1 (word), A1-2 (numbers), new E0109 | none | 4 (R3:3 R4:1) (+6 with E0109 by content) |
| OWN-10 | added | `owns` on the making line for an `owns` field, `Option[owns T]`, a list of `owns` (at first and later): E2123; `for x in owns xs`, `match owns e`; `owns` on a class type E2114 | `owned e` only as `for`/`match` source: `body.rs:511-521`, `:1583-1587`; no `owns` type, no E2123/E2114 (absent from `codes.rs`) | not built | A1-6 | none | 4 (R3:4) (+4 E2123, +2 E2114 by content) |
| OWN-11 | added | a part stored elsewhere (`push`, `return`, a field, an `owns` arg) is handed over on its own; part then whole is E3040; a class field or a non-owning place is E3013 | field-sensitive move paths `drops.rs:84-140`; partial move E3042 `drops.rs:1631`; class field E3012 `drops.rs:992-1001`; borrowed-param E3013 only when the value needs drop (`drops.rs:1005-1012`) | partly | `n = p.name` becomes a view (BRW-12, borrow area) so only stores hand over; fold E3042→E3040, E3012→E3013; E3013 for dropless values too (A1-3, A1-4) | none | 8 (R1:2 R2:1 R3:1 R11:2 R12:1 R15:1) |
| OWN-12 | added | hand-over to a C parameter declared as keeping it; `x.into_raw()` only inside `unsafe:` (E3100), made by the compiler | only `ForeignBox.into_raw` in Ember source, which calls `mem.forget`: `std/src/ffi.em:177-181` | not built | built-in `into_raw()` on `owns`/class objects (A1-11) | none | 0 (+3 `into_raw` by content: R1:1 R16:2) |
| OWN-13 | added | words before a value act on its object: `copy of`, `owns`, `shares`, `shares mut`, `thread_shared` (E7017), `weakref`, `sharedref mut`, `ref`, `ref mut`; an owner made from another does no more than it (E3023); each word in an untyped name list covers one name (E0116) | `ref`/`ref mut` before a place exist (`ExprKind::RefOf`, `ember_ast/src/lib.rs` ~750); `owned e` (`ExprKind::Owned`, `lib.rs:760`); nothing else | not built | `owns e`, `copy of e` here (A1-5, A1-6); `shares`/`thread_shared`/`weakref`/`sharedref mut` are the shares area's; E0116 the grammar area's | none | 0 (most R3 tests use these words) |
| FN-1 | changed | parameter words: plain reads; `ref mut` in-out (place, may be re-pointed); `const ref mut`; `owns` hands over, numbers included; bare `mut` E0113; `ref` in a parameter's type E0117 | `ast::Mode {Borrow, Mut, Owned}` `ember_ast/src/lib.rs:303-312`; parsed `ember_parser/src/decls.rs:1015-1046`, `:1580-1586`; lambdas `body.rs:1650-1660`; `Mode::Owned` used 107 times, `FnParamMode::Owned` 21 times outside tests | partly | rename and extend modes, E0113/E0117 (A1-1, A1-8); read-only plain params are BRW-15 (borrow area) | 2: s1 w1 | 62 (R4:35 R5:6 …) |
| FN-1a | changed | a `MutSpan` parameter that writes says `ref mut`; the call says `ref mut` before the span, even one made in the call | `mut s: MutSpan` accepts `buf.as_mut_span()`: `ember_typeck/src/lib.rs:56109-56160` | partly | A1-8 | 1: s1 | 0 |
| FN-2 | changed | no word reads; a callee that needs its own takes `owns`; caller passes `copy of x` or hands `x` over | plain = borrow (built); no `copy of` | partly | A1-5 | none | 0 |
| FN-2a | changed | a call writes `ref mut`/`const ref mut` before an argument for such a parameter (E3028 missing or extra; E3027 before what may not change); `owns` args take no word; lock/Atomic/channel args take none | call never writes a mode (`ast::Arg` has no word, `ember_ast/src/lib.rs:828-833`); E3027 for a non-place `mut` argument `lib.rs:56146-56160`; no E3028 (`codes.rs` lacks it) | not built | A1-8 | 3: s3 | 14 (R4:11 R14:2 R15:1) (+7 E3028, +7 E3027 by content) |
| FN-3 | changed | a function hands its result over; a view result points into view args (`-> T from name`); a view of locals/temporaries returned is E3063 | returns by move (built); returned view of a local is E3060 (`ember_analysis/src/borrows.rs:3890`); `@borrows(p)` instead of `from` | partly | E3060→E3063 at return (lifetime area owns `from`, LT-1a) | 2: s1 w1 | 0 |
| FN-4 | changed | the object is `this`, not listed; `owns this` after the parameters; `static fn` (E2115 outside a type); `this` in a static/free fn and `self` are E1010 | receiver `self`/`mut self`/`owned self`: `ast::ParamKind::Receiver` `ember_ast/src/lib.rs:326`; a type-body fn without receiver is associated | not built | A1-7 (with the grammar area for GRM-41) | 1: w1 | 80 (R5:24 R15:12 R12:11 …) |
| FN-6 | changed | callable types `fn(A, ref mut B, owns C)`, `thread_shared fn`, no change list; word mismatch is E2020 (E2228 retired) | `fn(mut B, owned C)` types; `E2228` mode mismatch `ember_typeck/src/lib.rs:36061` | partly | words (A1-1, A1-8); E2228→E2020; `thread_shared fn` is the threads area's | 13: k6 s4 w3 | 0 |
| FN-8 | changed | only the E1010 help for a script local used in a function adds "a `static mut` for a module variable that functions change" | `main` forms built (`lib.rs:48722-48806`, E2183); help text for the script-local case not located | partly | help text, after A1-17 | 12: k10 s2 | 0 |
| FN-9 | changed | a class parameter takes its word as any type: plain reads (object neither changes nor ends while used), `ref mut` may change and re-point, `owns` hands the object over | plain handle param writes fields and calls `mut self` methods; `owned` handle from a variable is a retained copy the caller keeps; `mut` re-points; RC-5 store-back (ODR-065, `lower.rs:7350-7360` `keep_handle_alive`) | partly | A1-9 with A1-8; read-only enforcement is BRW-15 | 4: s1 w3 | 0 |
| MONO-10 | added | a call proved to name one whole variable twice (two `ref mut` args, or a `ref mut` arg and a module variable the callee uses by name) compiles against its own copy of the callee where the two are one place | nothing; `mem.swap(value, value)` is E3022 (test OWN-6) | not built | A1-22 | none | 0 (+1 `add_both`, R4) |
| ALC-1 | changed | `T using A` names the manager kind on `owns`/`shares`/class/`Array`/`Map`/`Set`; the making line names the instance (`… using current`); E2020 wrong kind, E2114 on other types, E3063 outliving the instance; one word for multi-instance managers; `Box[T, A]`/`Array[T, A]` are E1010 | no allocator parameter at all: `Array[T, A]`/`Array.new_in` never existed (no `new_in` in the compiler or std) | not built | A1-14 | none | 4 (R3:3 R4:1) (+11 R3 tests with `using … LevelArena`) |
| ALC-2 | changed | `fn allocate(layout)`, `unsafe fn deallocate/reallocate`; a program's own call inside `unsafe:` (E3100); `Layout(size, align)` checks power of two and isize fit, panics; fields `pub(read)` | `std/src/mem.em:110-200`: `self` methods, safe deallocate/reallocate, `Layout` a plain `pub` struct; `unsafe interface` parsing `ember_parser/src/decls.rs:369` | partly | A1-13 | 2: s1 w1 | 0 |
| ALC-5 | added | while a value made in an instance lives, the instance changes only through allocation/`Allocator` calls and `alloc*`; other changes E3021; reads allowed | nothing | not built | A1-15 | none | 0 |
| ARN-1 | changed | `Arena` not Copy; `alloc*` through a name that may change the arena; views point into memory, not the cursor, and do not conflict; `reset`/`scope`/end end every view (later use E3060) | `alloc` takes a shared borrow: `ember_typeck/src/lib.rs:54994-55010`; arena region source `ember_analysis/src/borrows.rs:2903`, `:3552`, `:3902-3921`; `reset` takes `mut` (`lib.rs:54925`), conflict reported E3021 | partly | A1-16 | 4: k1 s2 w1 | 0 |
| ARN-2 | changed | `alloc*` of a type needing drop is E3090 whose help gives `using`; `Arena`/`FixedArena` implement `Allocator`; no `alloc_nodrop` | E3090 `lib.rs:54980-54991` with help naming `alloc_nodrop`; `alloc_nodrop` accepted `lib.rs:54933`, `ember_hir/src/lib.rs:2262` | partly | remove `alloc_nodrop`; arenas implement `Allocator` (A1-14, A1-16) | 1: r1 | 0 |
| ARN-3 | changed | `alloc(owns v: T) -> ref mut T` hands `v` over | built: the value is moved in (`lib.rs:54994-55010`) | text only | spelling only, through A1-1 | 6: k2 s4 | 0 |
| ARN-4 | changed | `Arena.fixed(ref mut buffer)` from a `MutSpan[u8]`, held as a kept `ref mut` view | `Arena.fixed(bytes)` borrows mutably (test ARN-4 `reject_using_fixed_backing…` gives E3021) | partly | call word `ref mut` (A1-8, A1-16); kept view is LT-45 (lifetime area) | 7: k1 s6 | 0 |
| ARN-6 | changed | `scope()` through a changeable name; `ScopedArena` holds a `ref mut` view (LT-46) | built: `borrows.rs:74-75`, `:4829`, `:4937` (E3096) | text only | none beyond A1-16's "changeable name" | 3: k3 | 0 |
| ARN-7 | changed | no reuse while a view is used (every such operation ends views); while a `using` value lives, only `Allocator` methods change the arena | `reset` conflicts with live views as E3021 | partly | A1-15, A1-16 | 2: k1 w1 | 0 |
| ARN-8 | changed | `MaybeUninit`: `write(owns v)`, `unsafe fn assume_init() owns this`; for `T` needing drop, `uninit()`, `write`, `write_at` are unsafe (E3100) | `write`/`write_at` safe for every `T`: `lib.rs:49897-49945`, `:55670`; `uninit()` `lib.rs:38823` | partly | A1-12 | 5: k5 | 0 |
| DRP-1 | changed | `fn drop():` changes its own object as a given; explicit call E3070 | `fn drop(mut self)`; E3070 built `lib.rs:46587-46601` | partly | spelling (A1-7) | 5: s4 w1 | 0 |
| DRP-5 | changed | no field taken out of `this` in `drop` (E3010); `mem.take(ref mut this.f)`; `drop` makes no owner of its own object (OBJ-5) | E3010/E3013 in drop `drops.rs:735-760`; self escape check `drops.rs:477-690` | partly | spelling (A1-7, A1-8) | 3: s3 | 0 |
| DRP-6 | changed | ending `owns T` ends `T` and frees; class owner ends object unless weakrefs hold the block; `shares` subtracts; `weakref` subtracts weak count; views nothing; guards unlock | Box drop glue (`ember_codegen_c/src/lib.rs:2490`, `:5408`, `:12032`); class release and Shared release by count; `Ref`/`RefMut` guards | partly | `owns` (A1-6); class end without retain (A1-9); `shares`/`weakref` (shares area) | 10: s8 w1 r1 | 0 |
| DRP-7 | changed | "reference" → "view" | built: `lower.rs:878-882` storage end for DRP-7; E3060 shape B7 | text only | none | 11: k2 s8 w1 | 0 |
| DRP-8 | added | ending a value counts as a call of its `drop` for every rule (E3020 inside a loop over what `drop` changes; a function that may end it changes what `drop` changes) | effect pass records drops as call sites: `ember_analysis/src/effects.rs:1202-1250` (`drop_sites`, `drop_of`), but for effect bits only (Alloc, Panic …), not changes | partly | A1-21 (needs the effects area's change summary) | none | 0 |
| EXP-4 | changed | a temporary lives until the last use of every view made from it (`word = line.to_lower().trim()`); a condition temporary lives into the block when the condition names what it holds | temporaries end at the statement: `lower.rs:1020-1045` (`statement_temp`, `end_plain_temps`); E3060 "dropped at the end of its statement while it is still borrowed" `borrows.rs:4760` | not built | A1-18 | 15: k3 s7 w5 | 0 |
| EXP-5 | changed | a changing method on a temporary whose only effect is that change is E2140 | E2140 for non-place assignment only: `lib.rs:16395`, `:16449`, `:26002` | partly | A1-19 | 2: k2 | 0 (+3 E2140 by content, R8) |
| EXP-6 | changed | a whole non-Copy name used as a value hands over; `owns` hands over every type; parts handed over on their own; E3010, E3011; class field and non-owning place E3013 | `drops.rs:984-1065` (E3011, E3012, E3013), partial moves E3042 | partly | A1-3, A1-4 | 15: s4 w11 | 4 (R1:1 R11:2 R15:1) (+9 E3013 by content) |
| EXP-9 | changed | `a is b` compares class objects/views by address; `x is T` kind test and `if x is T as name:` (one type-id compare); E2036 always passes, E2037 never passes | `is` identity/`None` only: parser `body.rs:948-951`, typeck `lib.rs:58395-58402` (E2150) | partly | A1-20 (with the dyn/cast area, DSP-4, GRM-45) | 2: s1 w1 | 0 |
| EXP-10 | changed | wording only ("reference" → "view", "move" → "hand over") | built `lib.rs:16154-16171`, `:37009` | text only | none | 2: k1 s1 | 0 |
| HEAP-1 | changed | heap types allocate through `ember_alloc…` or the manager named with `using` | runtime allocator only | partly | A1-14 | none | 0 |
| HEAP-3 | changed | a `shares`/`thread_shared` value lives in one block with its count (and weak count where weakrefs may reach it); a class object only adds one; count compared with its maximum before +1 | `Shared(v)` builds a counted block: `ember_typeck/src/lib.rs:19462-…` (`counted_of`) | partly | shares area | none (folder absent) | 0 |
| HEAP-4 | removed | (`Shared.get` with a checked read access) | `get` with `ember_access_begin_read` | remove | remove with `Shared` (shares area) | 1: r1 | 0 |
| HEAP-5 | removed | (`Shared.get_mut` with a checked write access) | `get_mut` with `ember_access_begin_write` | remove | remove (shares area; run-time exclusivity checks go) | 11: r11 | 0 |
| HEAP-6 | changed | `shares`/`shares mut`/`thread_shared`/`weakref` not Copy; another owner only by a line that says so; `=` hands over | `Shared[T]` is Copy (retain on copy) | not built | shares area | 1: w1 | 0 |
| HEAP-7 | changed | `weakref T` to a class object or a `shares`/`thread_shared` value; of a part E2119 | `Weak[O]` for handles/Shared/SyncShared | partly | shares area | 1: w1 | 0 |
| HEAP-9 | changed | `owns T` replaces `Box[T]`: own block, reads as `T`, `owns dyn I`, making line says `owns` (E2123), E2114 on a class; not Copy; Clone when `T` is; foreign type held `owns` ended by its contract's destructor | `Box` built as a compiler struct `{ value: *mut T }` or `{ data, vtable }` for `dyn`: `ember_typeck/src/lib.rs:19389-19455` (`box_of`), auto-deref (35 `box_inner` sites), codegen `ember_codegen_c/src/lib.rs:2490`, `:5408`, `:10692` (`BoxNew`), `:9630` (`DynBoxNew`) | partly | A1-6 (representation kept, surface replaced) | 2: s2 | 0 |
| HEAP-10 | changed | `shares` and its weakref never reach another thread (E7015); `thread_shared T` atomic counts; changed only through Atomic/Mutex/RwLock/channel/`guarded by` (E7014) | `SyncShared[T]`, `@sync` (`lib.rs:19462` `counted_of(inner, sync)`) | partly | shares/threads areas | 3: w3 | 0 |
| STA-1 | changed | `static` never changes, `T: ThreadShared` (E7002); `static mut` is a module variable changed by name with no `unsafe`, main thread and scoped tasks only unless `guarded by` (E7012); `global` E0100; a module-level assignment outside the entry file E0100 with the declaration to write; foreign `static mut` only in `unsafe` | native `static mut` refused: `ember_typeck/src/lib.rs:10258-10264` ("`static mut` needs `unsafe`, which is not in this phase", E1010); E7002 "not Sync" `lib.rs:10192-10201`; file-scope statements outside entry E0100 with generic text: `ember_driver/src/lib.rs:3749-3763`; `global` E0100 with a `static`/class-field help: `ember_parser/src/python.rs:156-163` | partly | A1-17 (E7012/`guarded by` threads area; ThreadShared rename threads area) | 4: k1 s3 | 0 (+90 tests use `static mut`: R1:4 R2:2 R4:12 R5:10 R8:16 R12:21 R14:4 R15:16 R17:5) |
| STA-3 | changed | a `static mut` whose value owns heap memory is built on first access, never in read-only memory (CT-5) | first-access accessor exists: `ember_codegen_c/src/lib.rs:7225` (`…_static(void)` with `rt_static_begin`), hot reload `reload.rs:337`; pending statics `ember_typeck/src/lib.rs:10265-10285`, `:7863` | partly | A1-17 | 3: k2 s1 | 0 |

**Features the compiler implements under a removed name** (no H.41 row of their own beyond the above):

| removed thing | where today | handled by |
|---|---|---|
| the `owned` word (parameters, receivers, `owned fn`, `for x in owned xs`, `match owned e`, callable types, lambda parameter inference) | lexer `ember_lexer/src/token.rs:231`; parser `decls.rs:835`, `:1021`, `:1046` (a `once fn` parameter forced `Owned`), `:1584`; `body.rs:511`, `:1173-1207`, `:1583`, `:1650-1660`; `ast` `lib.rs:311`, `:760`, `print.rs:222`, `:658`; 227 uses in `std/src`, 319 `.em` test files, 54 `.rs` files | A1-1 (E1010 "`owned` is now `owns`"; see question Q1 for `@ffi(param(w, owned, one))`) |
| `Box[T]`, `Box[dyn I]`, `Box(v)`, `b.get()` | `ember_typeck/src/lib.rs:15084-15093`, `:19389-19460`, `:38210-…`, `:48133`; `ember_types/src/lib.rs:1831`, `:2012`, `:2321`, `:3518`; analysis `access.rs:504`, `borrows.rs:5201`, `cycles.rs:257`, `exit_drops.rs:80`, `effects.rs:449`; 18 uses in std (`core.em:92` `AnyError.error: Box[dyn Error]`, `coroutine.em:53-63`, `ecs.em:1208`); 138 `.em` test files | A1-6 |
| `mem.forget` | `Builtin::MemForget`: `ember_hir/src/lib.rs:2063`, `:2467`; typeck `:16380`, `:49565`; `drops.rs:607`; `effects.rs:1538`; `ember_codegen_c/src/lib.rs:8795`, `:10136`; `ember_codegen_llvm/src/builtins.rs:313`; `ember_interp/src/builtin.rs:638`; `std/src/mem.em:97`, `ffi.em:180` | A1-10 |
| `alloc_nodrop` | `ember_hir/src/lib.rs:2262`; typeck `:54933`, `:54990`, `:55047` | A1-16 |
| Copy class handles with retain on copy, retained `owned` handle args, RC-5 store-back | `ember_types/src/lib.rs:2122`, `:2658`; `ember_mir/src/lower.rs:7340-7375`; `ember_analysis/src/uncounted_handles.rs`, `temporary_owners.rs` | A1-9 (with the shares area) |
| retired codes E3012, E3041, E3042, E2228 | `drops.rs:995`, `:1661`, `:1631`; typeck `:36061` | A1-3, A1-8 |

### 1b. Old tests by rule (every non-keep named)

Spelling changes that recur, so they are not repeated per file: `fn drop(mut self)` → `fn drop():`;
`self` → `this` and the receiver dropped from the parameter list; `owned x` → `owns x`; `mut x: T`
parameter → `ref mut x: T` and `ref mut` at the call; `x.clone()` → `copy of x`; `str` → `StrView`;
`Box(v)`/`Box[T]` → `owns v`/`owns T` and `.get()` removed; `mem.take(x)` → `mem.take(ref mut x)`;
`ref T` in a parameter type → a plain parameter; the old E3040 text "has been moved out of" → the new
hand-over text (every E3040 annotation is a respell for this reason).

* **OWN-1** k1 s1. s: `reject_two_owners_of_one_value` (E3040 text).
* **OWN-2** s3: `accept_a_drop_method_runs_at_scope_end`, `accept_drops_run_in_reverse_declaration_order`, `accept_owned_parameters_drop_in_the_callee`.
* **OWN-3** k1 s6: `accept_one_move_per_value_in_one_expression` (owns, copy of), `accept_replacing_a_field_keeps_the_handle` (drop), `reject_a_field_write_through_a_moved_handle` (`Box[dyn Speak]` → `owns dyn Speak`, this, E3040 text), `reject_a_list_used_after_assignment_moved_it`, `reject_a_value_moved_twice_in_one_expression`, `reject_use_after_move` (E3040 text).
* **OWN-4** s1 w2. s: `accept_a_move_the_loop_puts_back`. w: `reject_a_move_in_a_loop`, `reject_a_move_missing_on_some_path` (E3041 retired → E3040).
* **OWN-6** s6 w1 r2. s: `accept_span_replace_result_refreshes_index_pointer`, `accept_take_replace_and_swap_preserve_ownership`, `reject_a_handle_used_after_mem_drop`, `reject_replace_while_shared_borrow_lives` (E3021 text), `reject_take_without_default`, `reject_use_after_mem_drop_of_noncopy_value`. w: `reject_swap_of_the_same_place` (`mem.swap(ref mut value, ref mut value)` names one whole variable twice: MONO-10 and the E3022 exception make it legal). r: `accept_forget_and_align_of_use_ownership_and_layout`, `reject_use_after_forget` (`mem.forget`).
* **OWN-7** k2 w1 r1. w: `accept_a_copy_holding_handles_retains_them` (`@derive(Copy)` with a class field is now E2080). r: `accept_copying_nested_enums_of_handles` (retain of Copy handles, and `return w` from a plain parameter is now E3013).
* **OWN-8** k2 s11 w1. s: `accept_derived_clone_satisfies_clone_bound`, `accept_derived_clone_with_handwritten_field_clone`, `accept_derived_copy_field_struct_clone`, `accept_derived_enum_clone` (its `assert-c` names `(*self)`: check the C name of `this`), `accept_generic_derived_clone`, `accept_generic_derived_enum_clone`, `accept_handwritten_clone_interface`, `accept_nested_derived_struct_clone`, `reject_derive_clone_for_nonclone_enum_payload`, `reject_derive_clone_for_noncopy_field`, `reject_derive_clone_settled_before_the_drops_are_known`. w: `accept_cloning_arrays_and_strings` (`copy of tokens` copies `Token` objects, and `Token` has no `Clone`: E2116 unless it derives it).
* **FN-1** s1 w1. s: `accept_a_mut_bool_argument_keeps_its_write`. w: `accept_cell_and_refcell_through_borrowed_parameters` (`RefCell` removed; `Cell.set` through a plain parameter is E3023).
* **FN-1a** s1: `accept_a_mut_view_parameter_takes_a_view_value`.
* **FN-2a** s3: `accept_callee_mode_forms_a_mutable_borrow` (`double(ref mut value)`), `reject_a_lambda_given_to_a_mut_callable_parameter`, `reject_mut_argument_without_a_place` (E3027 with `ref mut` before a temporary; message text).
* **FN-3** s1 w1. s: `accept_returning_by_move_and_a_borrowed_parameter` (E0117). w: `reject_returning_a_reference_to_a_local` (E3060 → E3063).
* **FN-4** w1: `accept_receiver_modes_and_associated_functions` (`fn zero()` in a type body is now a method; needs `static fn`; `owns this`).
* **FN-6** k6 s4 w3. s: `accept_callable_type_parameter_modes`, `accept_mut_lambda_for_callable_type`, `accept_owned_callable_type_parameter`, `accept_owned_indirect_parameter_is_transferred`. w: `accept_a_static_function_value_is_called_by_name` (`HOOK = fn…` inside a function now assigns to the module's `HOOK` under GRM-4, refused; see Q3), `reject_callable_type_parameter_mode_mismatch`, `reject_function_value_mode_mismatch` (E2228 → E2020). The folder also holds a stray build directory `tests/conformance/FN-6/target/debug`.
* **FN-8** k10 s2: `accept_main_owns_its_args` (owns), `accept_main_takes_its_args_by_any_name` (bare `mut` is E0113).
* **FN-9** s1 w3. s: `reject_a_borrowed_handle_parameter_repointed` (E3023 text). w: `accept_a_borrowed_handle_parameter_writes_the_object` (writes through a plain parameter are E3023), `accept_a_mut_handle_parameter_repoints_a_field_safely` (RC-5 store-back gone; `swap_in(ref mut p.child, p)` overlaps), `accept_an_owned_handle_argument_the_caller_keeps` (`take(a)` now ends `a`).
* **DRP-1** s4 w1. s: `accept_a_drop_written_in_an_extend_block`, `accept_drop_runs_once_at_the_end_of_life`, `accept_mem_drop_ends_a_value_once`, `reject_calling_drop_explicitly`. w: `accept_mem_drop_ends_a_guard` (`RefCell` removed; a `MutexGuard` can carry the same check).
* **DRP-5** s3: `accept_drop_may_read_its_fields`, `reject_move_field_into_owned_call`, `reject_move_field_out_of_drop` (E3010 text names `mut self`).
* **DRP-6** s8 w1 r1. s: `accept_box_drops_payload_before_free`, `accept_box_method_through_shared_ref`, `accept_box_owns_dereferences_and_frees` (its `assert-c` names the `…BoxG…` typedef), `accept_class_handle_and_shared_release` (`shares Token()`, `_alias = shares owner`; `assert-c` names runtime symbols the shares area may rename), `accept_explicit_box_type_argument`, `accept_nested_boxes_drop_inside_out`, `accept_overwrite_drops_old_box`, `reject_use_after_moving_box_of_copy_payload`. w: `reject_moving_box_while_payload_is_borrowed` (the view used after the hand-over is E3060 at the use, not E3021 at the move). r: `reject_get_from_temporary_box` (`get` is gone).
* **DRP-7** k2 s8 w1. s: `accept_generic_string_view_without_drop`, `accept_heap_view_source_outlives_holder`, `reject_break_exits_with_holder_borrow`, `reject_break_exits_with_holder_borrow_of_counter`, `reject_continue_exits_with_holder_borrow` (E3021 text), `reject_generic_drop_parameter_carrying_borrow`, `reject_generic_drop_reads_expired_borrow`, `reject_generic_string_drop_reads_expired_borrow`. w: `reject_return_holder_borrowing_local` (returned: E3063).
* **EXP-4** k3 s7 w5. s: `accept_a_loop_over_a_temporary_collection`, `accept_a_loop_over_an_iterator_of_a_temporary_array` (`for ref mut x in noisy()`), `accept_an_early_exit_drops_the_temporaries_of_what_it_leaves`, `accept_condition_temporaries_drop_before_each_body`, `accept_nested_view_producers_in_for_keep_their_owner`, `accept_synthesized_binding_temporaries_drop_in_source_order`, `accept_temporaries_drop_in_reverse_order_of_creation`. w: `accept_temporary_text_producers_inside_synthesized_evaluation` (`text.split_once(separator(ref mut text))` overlaps a read and a `ref mut` argument: E3021), and the four rejects that the extension now accepts: `reject_a_borrow_of_a_plain_temporary_past_its_statement`, `reject_a_synthesized_get_result_outliving_its_temporary`, `reject_a_synthesized_split_result_outliving_its_temporary`, `reject_an_iterator_of_a_temporary_array_past_its_statement`.
* **EXP-5** k2.
* **EXP-6** s4 w11. s: `accept_partial_move_into_a_call_clears_the_field_flag`, `reject_a_moved_field_is_the_span`, `reject_borrowed_field_move_into_owned_call`, `reject_borrowed_move_to_return` (E3013 text). w (BRW-12: `x = p.f`, `x = xs[0]`, and `x = o` for a plain parameter now make a view, not a move, so drop order or the verdict changes): `accept_borrowed_copies_and_dropless_moves` (`q.w = t` through a view: E3023), `accept_conditional_partial_move_uses_field_drop_flags`, `accept_nested_partial_move_preserves_disjoint_paths`, `accept_owned_partial_move_drops_each_field_once`, `accept_reinitialising_a_moved_field_restores_the_whole`, `reject_move_field_out_of_borrowed_param`, `reject_move_field_out_of_shared_ref`, `reject_move_of_borrowed_param`, `reject_moving_out_of_a_class_field` (E3012 retired; the `x =` half is now legal), `reject_moving_out_of_an_element`, `reject_whole_use_after_owned_partial_move` (E3042 retired).
* **EXP-9** s1 w1. s: `reject_is_on_values` (E0117, E2150 text says "class handles"). w: `accept_is_tests_options_and_identity` (`alias = first` hands the object over, so `first is alias` is E3040).
* **EXP-10** k1 s1: `accept_optional_chaining` (`this`, E0117).
* **HEAP-4** r1: `accept_shared_get_in_a_function_taking_text`.
* **HEAP-5** r11: `accept_shared_get_mut_nll`, `accept_shared_get_mut_returned`, `accept_shared_get_reader_nll`, `accept_unique_shared_get_mut_elides_access`, `run_fail_a_shared_in_a_class_field_conflicts`, `run_fail_aliased_shared_get_mut`, `run_fail_branch_selected_shared_get_mut_conflicts`, `run_fail_nested_shared_get_mut_return_conflicts`, `run_fail_returned_shared_get_conflicts_with_get_mut`, `run_fail_returned_shared_get_mut_conflicts`, `run_fail_shared_get_conflicts_with_get_mut`.
* **HEAP-6** w1: `accept_shared_copy_final_release_with_weak`.
* **HEAP-7** w1: `accept_shared_weak_upgrade_fails_during_drop`.
* **HEAP-9** s2: `accept_a_list_of_wide_vectors_is_aligned_to_their_size`, `accept_array_storage_is_aligned_for_its_elements` (`copy of xs`).
* **HEAP-10** w3: `accept_a_sync_shared_shares_one_value`, `reject_a_shared_in_a_sync_class` (E7001 retired), `reject_get_mut_on_a_sync_shared` (becomes E7014).
* **ALC-2** s1 w1. s: `reject_implementing_allocator_without_unsafe`. w: `accept_an_unsafe_extend_implements_allocator` (its `reallocate`/`deallocate` calls now need `unsafe:`).
* **ARN-1** k1 s2 w1. s: `accept_allocating_the_unit_value` (`Arena.fixed(ref mut bytes)`), `reject_using_an_arena_after_move` (E3040 text). w: `reject_reset_while_an_allocation_is_live` (E3021 at `reset` → E3060 at the later use).
* **ARN-2** r1: `accept_alloc_nodrop_skips_individual_destruction`.
* **ARN-3** k2 s4: `accept_alloc_array_calls_default_and_alloc_zeroed_fills_zeros`, `accept_generic_allocation_when_the_concrete_type_needs_no_drop` (`@borrows` removed), `reject_alloc_array_without_default_and_alloc_zeroed_without_zeroable`, `reject_generic_allocation_when_the_concrete_type_needs_drop` (a plain `frame: Arena` may not allocate: `ref mut frame`).
* **ARN-4** k1 s6: `accept_a_fixed_arena_in_noalloc_code`, `accept_fixed_arena_uses_caller_storage_without_growth`, `reject_a_growing_arena_in_noalloc_code`, `reject_using_fixed_backing_while_the_arena_is_live`, `run_fail_empty_fixed_arena_is_exhausted`, `run_fail_fixed_arena_panics_on_exhaustion`.
* **ARN-6** k3. **ARN-7** k1 w1: `reject_reset_while_an_arena_view_is_live` (E3060 at the use). **ARN-8** k5.
* **STA-1** k1 s3: `accept_a_static_mutated_through_a_mutex` (StrView), `reject_foreign_static_without_sync`, `reject_local_static_without_sync` (E7002 text says `ThreadShared`).
* **STA-3** k2 s1: `accept_a_static_initialises_on_first_access` (`Array[StrView]`).

Totals over the 201 files: **keep 46, respell 96, rewrite 42, retire 17.**

Outside these 40 folders, the unchanged rules of the same families (OWN-5, DRP-2..4, EXP-1..3, FN-5,
FN-7, FN-10, ARN-5a..g, ARN-8a, ARN-9..11, HEAP-8, MONO-6, MONO-7) hold 80 files, of which 31 contain an
old spelling by a script scan (not read by hand); they need respelling though their rules did not
change.

## 2. Work items

Ordered; each slice leaves the compiler consistent (builds, its tests green, nothing half-removed).
"Old tests" means the files of section 1b.

### Slice 1: the plain renames and removals

**A1-1 `owns` replaces `owned` everywhere** (M)
- Changes: `ember_lexer/src/token.rs:231` (`owned` stays lexed as a keyword, its use E1010 with the
  help of LEX's text); parser `decls.rs:835`, `:1015-1046`, `:1580-1586`, `body.rs:480-521`,
  `:1165-1210`, `:1583-1587`, `:1645-1662`; `ember_ast/src/lib.rs:303-312` (`Mode {Read, RefMut {
  is_const }, Owns}`), `:760` (`ExprKind::Owns`), `print.rs`; `ember_types` `FnParamMode` (21 uses);
  typeck `Mode::Owned` (107 uses); `std/src/*.em` (227 uses); `ember_fmt`.
- Completes: ARN-3 (text), the `owns` half of FN-1, FN-2, FN-6; the spelling half of OWN-10's
  `for x in owns xs` / `match owns e`.
- Tests: old respells for the `owned` files of OWN-2, OWN-3, OWN-4, FN-6, FN-8, DRP-5, EXP-6, ARN-3.
  Design: none pass alone (every design test also needs `this`/`ref mut`).
- Depends: the grammar/keyword area's decision on how `owns` is lexed (reserved or contextual); Q1.

**A1-10 `mem.forget` removed; `mem` signatures say `ref mut`/`owns`** (S)
- Changes: drop `Builtin::MemForget` from `ember_hir/src/lib.rs:2063`, `:2467`, typeck `:16380`,
  `:49565`, `drops.rs:607`, `effects.rs:1538`, `ember_codegen_c/src/lib.rs:8795`, `:10136`,
  `ember_codegen_llvm/src/builtins.rs:313`, `ember_interp/src/builtin.rs:638`; `std/src/mem.em:61`,
  `:97`; E1010 with OWN-6's message; `ForeignBox.into_raw` (`std/src/ffi.em:177-181`) moves to A1-11.
- Completes: OWN-2's "no `mem.forget`", OWN-6's forget half.
- Tests: retires OWN-6 `accept_forget_and_align_of_use_ownership_and_layout`, `reject_use_after_forget`
  (5 `.em` and 3 `.rs` files use `mem.forget` in all). Design: R1 `mem.forget` test (1).
- Depends: A1-11 must land in the same slice or `ForeignBox.into_raw` loses its body (FFI area owns
  `ForeignBox`'s removal).

**A1-12 `MaybeUninit` writes unsafe for a `T` that needs drop** (S)
- Changes: typeck `lib.rs:38823` (`uninit`), `:49897-49945` (`write`), `:55670` (`write_at`): E3100 when
  `needs_drop(T)`; `assume_init` spelled `owns this`.
- Completes: ARN-8. Tests: ARN-8 keeps 5 green. Depends: A1-7 for the `owns this` spelling only.

**A1-13 The `Allocator` interface of 0.9.15** (S)
- Changes: `std/src/mem.em:109-200` (`fn allocate(layout)`, `unsafe fn deallocate/reallocate`,
  `Layout` constructor checking a power-of-two `align` and `size` rounded to `align` fitting `isize`,
  panicking `invalid Layout`, fields `pub(read)`); typeck: calls of the two `unsafe fn`s outside
  `unsafe:` are E3100 (already general for `unsafe fn`); the global-allocator glue
  `ember_driver/src/lib.rs:4839-4891` calls them as compiler calls.
- Completes: ALC-2. Tests: ALC-2 s1 w1. Design: none mapped; the 11 R3 `LevelArena` tests need it, and
  contradict it (risk R6).
- Depends: A1-7 (`this`-style signatures), the visibility area for `pub(read)`.

### Slice 2: hand-over semantics

**A1-2 `owns` hands numbers over** (S)
- Changes: `ember_mir/src/lower.rs:7372-7375`: an argument for an `owns` parameter, `b = owns n`, an
  `owns` capture and `mem.drop(n)` lower to `Operand::Move` whatever the type; `drops.rs` then reports
  E3040 with no new analysis (all locals are already move paths, `drops.rs:96-108`); drop elaboration
  emits nothing for a Copy type. E0109 rewritten to its new case (`owns` before an existing name whose
  place already hands over, except `b = owns n` for a number, and `owns fn`), `body.rs:1193-1207`,
  typeck `:31735`.
- Completes: OWN-9, OWN-6's "numbers included". Design: R3 (3), R4 (1), and the 6 E0109 tests.
- Depends: A1-1. Cost: none at run time (a Move of a scalar is the same load).

**A1-3 The hand-over messages and folded codes** (M)
- Changes: `drops.rs:1592-1745`: the E3040 text of OWN-3 (naming `copy of` and `shares`), the loop case
  (`:1643-1665`, E3041 → E3040 with the "next round" wording), part-then-whole (`:1631`, E3042 → E3040,
  OWN-11's text), class field (`:992-1001`, E3012 → E3013 with `mem.take(ref mut q.name)`); E3013 for a
  hand-over out of a non-owning name whatever the type (drop the `needs_drop` exemption at
  `drops.rs:1005-1012`); `codes.rs:285`, `:302`, `:306` retired.
- Completes: OWN-4; with A1-4, OWN-3, OWN-11, EXP-6.
- Tests: OWN-4 w2; the E3040-text respells of OWN-1, OWN-3, OWN-6, DRP-6, ARN-1; EXP-6's E3013
  respells. Design: the 17 E3040 and 9 E3013 tests, once their other words exist.
- Depends: A1-1.

**A1-4 Only stores hand a part over; `x = p.f` is a view** (M)
- Changes: MIR lowering of a binding to a part (`lower.rs`, the `Let` path at `:1020-1035`) produces a
  view instead of a move when the borrow area's BRW-12 says so; `drops.rs` partial-move paths stay for
  `push`/`return`/field/`owns`-argument stores.
- Completes: OWN-3 (view case), OWN-11, EXP-6. Tests: EXP-6 w11, OWN-7 r1, EXP-9 w1. Design: OWN-11 (8),
  EXP-6 (4).
- Depends: **the borrow area's BRW-12/BRW-15** (what a part name is and what it may do). Lands with it.

### Slice 3: `copy of`

**A1-5 `copy of x`** (M)
- Changes: parser (contextual `copy` before `of`, `ExprKind::CopyOf`; also `exists as copy of v` and
  capture lists, which the closure and `exists as` areas parse); typeck: requires `Clone`, runs `clone`;
  `@derive(Copy)` implies `Clone` (`ember_typeck/src/derives.rs`); E2116 for a `weakref`, a view type,
  a type without `Clone`, a value whose views point into its own `let` field; `x.clone()` at a use is
  E1010 "write `copy of x`" (a type's own `fn clone()` stays); a class object's `copy of` makes a new
  object; a struct holding `shares`/`weakref` fields copies them as links (+1 each).
- Completes: OWN-8, FN-2, OWN-13's `copy of`. Tests: OWN-8 s11 w1, HEAP-9 s2, and every `.clone()`
  respell (89 `.em`, 123 `.rs` files contain `.clone()`, many of them Rust's own). Design: OWN-8 (65),
  and 130 tests use `copy of`.
- Depends: A1-9 for the class-object case; the shares area for link fields.

### Slice 4: `owns T` (Box removed)

**A1-6 `owns T`, `owns dyn I`, the making-line `owns`** (L)
- Changes: parser: `owns T` in type position, `owns e` before a value anywhere (OWN-13); typeck: keep
  `box_of` (`lib.rs:19389-19455`) as the representation with origin `owns`; `Box`, `Box[…]`, `Box(…)`
  are E1010 with the vocabulary's help (`lib.rs:14630-14655`, `:15084`, `:38210`, `:48133`); `.get()` on
  an `owns` value refused with "remove `.get()`" (`:45876-45910`, `:50142`); E2114 on a class type;
  E2123 when a value made for an `owns` place (struct-literal field, `Some(…)` for `Option[owns T]`,
  `push`/`insert` argument of an `Array[owns …]`, field assignment `e.mesh = …`) is not written
  `owns …`; codegen typedef names (`ember_codegen_c/src/lib.rs:2490`, `:5408`, `:10692`, `:9630`) and
  the LLVM backend's equivalents; analysis sites that test `name.is("Box")` (`access.rs:504`,
  `borrows.rs:5201`, `cycles.rs:257`, `exit_drops.rs:80`, `effects.rs:449`); `std` (18 uses).
- Completes: HEAP-9, OWN-10, DRP-6's `owns` half, OWN-13's `owns`.
- Tests: DRP-6 s8 r1, OWN-3 `reject_a_field_write_through_a_moved_handle`; 138 `.em` files use `Box`.
  Design: OWN-10 (4), the 4 E2123 and 2 E2114 tests, all `owns dyn` tests (R14:19 use `owns`).
- Depends: A1-1. Cost: identical to today's `Box` (one pointer, or two for `dyn`), as C's `T*`.

### Slice 5: parameter words, call words and `this`

**A1-8 `ref mut`/`const ref mut` parameters and words at the call** (M)
- Changes: parser: `ref mut`/`const ref mut` before a parameter name (also in callable types and lambda
  parameters); bare `mut` E0113; `ref`/`ref mut` inside a parameter's type E0117; `ast::Arg` keeps the
  argument expression, whose `RefOf` form carries the word. Typeck `pass_argument*`
  (`lib.rs:56109-56160`): E3028 when the word is missing or extra (also `const`), E3027 when `ref mut`
  stands before something not changeable, no word for `Mutex`/`RwLock`/`Atomic`/channel parameters;
  E2228 → E2020 (`:36061`).
- Completes: FN-1, FN-1a, FN-2a, FN-6 (words), ARN-4 (`Arena.fixed(ref mut b)`).
- Tests: FN-1 s1, FN-1a s1, FN-2a s3, FN-6 s4 w2, OWN-6 s4, ARN-1/3/4 respells; `mut self` and `mut`
  parameters are in 546 `.em` files. Design: FN-1 (62), FN-2a (14), E3028 (7), E3027 (7).
- Depends: A1-1; the borrow area for the read-only meaning of a plain parameter (BRW-15) and for
  `ref mut` exclusivity (BRW-22).

**A1-7 `this`, `owns this`, `static fn`, `fn drop():`** (L)
- Changes: parser removes `ParamKind::Receiver` (`ember_ast/src/lib.rs:326`), `self` E1010, `this`
  outside a method E1010, `static fn` (E2115 outside a type body), `owns this` after the parameters;
  typeck: a type-body function is a method unless `static fn`; `drop` has no receiver word; DRP-5's
  check (`drops.rs:735-760`) keyed to `this`. Every `std` method.
- Completes: FN-4, DRP-1, DRP-5 (spelling), ARN-8's `owns this`.
- Tests: FN-4 w1 and the `drop`/`self` respells across 1b. Design: FN-4 (80).
- Depends: the grammar area (GRM-41) and the effects area (what a method does to `this` is inferred,
  EXC-15).

### Slice 6: class objects have one owner

**A1-9 Class objects are not `Copy`** (L)
- Changes: `ember_types/src/lib.rs:2122`, `:2658` → not Copy; `@derive(Copy)` with a class field E2080;
  MIR: no retain on copy, `=`/`owns` hand the object over (`lower.rs:7340-7375`, `counted_handle`,
  `keep_handle_alive`, the ODR-065 store-back); remove or rework `uncounted_handles.rs` and
  `temporary_owners.rs` (they exist to elide retains that no longer happen); `is` compares objects.
- Completes: OWN-1, OWN-7, FN-9, EXP-9 (identity half), DRP-6 (class half).
- Tests: OWN-7 w1, FN-9 s1 w3, EXP-9 w1, OWN-3 `accept_replacing_a_field_keeps_the_handle`.
  Design: OWN-1 (26).
- Depends: **the shares area** (another owner only through `shares`; the class object's count and weak
  count, OBJ-1/OBJ-3), A1-8 (FN-9's words). Possibly owned by a classes area: coordinate.

### Slice 7: module variables

**A1-17 `static mut`** (L)
- Changes: typeck `lib.rs:10258-10264` accepts native `static mut`; a function's `X = v` / `X.push(…)`
  on a module variable assigns it by name (GRM-4); a `static` (not `mut`) assigned by name is refused
  (Q3); `driver/src/lib.rs:3749-3763`: a module-level assignment outside the entry file gives OWN's
  E0100 help ("`static mut FRAME: int = 0`"); `parser/src/python.rs:156-163` `global` help text;
  codegen: a `static mut` with no heap memory is a plain mutable C global (no accessor); one whose value
  owns heap memory uses the existing first-access accessor (`ember_codegen_c/src/lib.rs:7225`,
  `reload.rs:337`) on writable storage, never constant data (CT-5); FN-8's help.
- Completes: STA-1 (except E7012/`guarded by`/`ThreadShared`), STA-3, FN-8.
- Tests: STA-1 s3, STA-3 s1, FN-8 (help), FN-6 `accept_a_static_function_value_is_called_by_name`.
  Design: **90 tests use `static mut`** (R8:16, R12:21, R15:16 …); this item gates a large share of
  the design suite.
- Depends: the effects area (module-variable changes inferred, EFF-24); the threads area (E7012,
  `guarded by`, THR-20, `ThreadShared` for E7002).

### Slice 8: memory managers and arenas

**A1-14 `using A`** (L)
- Changes: parser: `T using A` after a type, `e using inst` after a making expression; typeck: E2114 on a
  type without heap memory; `Array[T, A]`/`Box[T, A]` E1010; E2020 when the instance's type is not the
  place's kind or the value came from another manager; types: `owns T using A`, `Array[T] using A`,
  `Map`/`Set` likewise, as distinct instances of the type; one instance word when `A` may have several
  instances (shared decision D3); codegen: blocks from `A.allocate`, growth through `A.reallocate`,
  end through `A.deallocate`, monomorphised direct calls; `Array.new_in(inst)`; `Arena` and
  `FixedArena` implement `Allocator` (runtime: a block given back to an arena is not freed one by one).
- Completes: ALC-1, HEAP-1, ARN-2 (with A1-16).
- Tests: design R3 `LevelArena` tests (11, after fixing risk R6), ALC-1 (4).
- Depends: A1-6, A1-13; the shares area for `shares T using A`; the lifetime area for E3063 (a value
  outliving its instance, LT-4); the effects area (ending such a value changes its instance).

**A1-15 An instance stays as its values need it** (M)
- Changes: `ember_analysis/src/borrows.rs`: a value made `using inst` holds a region on `inst`, as a
  view does, under which reads of `inst` are allowed and the compiler's own `Allocator` calls and
  `alloc*` are exempt; any other change of `inst` is E3021 with ALC-5's message.
- Completes: ALC-5, ARN-7. Tests: ARN-7 w1. Depends: A1-14; the borrow area's region model.

**A1-16 The arena borrow model of 0.9.15** (M)
- Changes: typeck `lib.rs:54900-55060`: `alloc*`/`scope()` called through a name that may change the
  arena (a plain `frame: Arena` parameter refused, E3023); remove `alloc_nodrop` (`:54933`, `:54990`,
  `:55047`, `ember_hir/src/lib.rs:2262`); E3090's help names `using`; borrows: `alloc*` results carry
  the arena's region but do not conflict with each other or with the call (`borrows.rs:2903`,
  `:3552`, `:3850-3921`); `reset`/`scope`/the arena's end end every view, and a later use is E3060
  (today E3021 at `reset`).
- Completes: ARN-1, ARN-2, ARN-4, ARN-6, ARN-7 (with A1-15).
- Tests: ARN-1 s2 w1, ARN-2 r1, ARN-3 s4, ARN-4 s6, ARN-7 w1. Depends: A1-8, the borrow area.

### Slice 9: expressions

**A1-18 A temporary lives as long as the views made from it** (L)
- Changes: `ember_mir/src/lower.rs:1020-1045` (`statement_temp`, `end_plain_temps`): a temporary from
  which a view is bound to a name is promoted to a hidden local of the enclosing block, ended right
  after the last use of every view made from it (a drop flag where paths differ); a condition's
  temporary lives into its block when the condition names what it holds (`exists as`, a kind test, a
  pattern); E3063 for a view of it that would outlive it; `borrows.rs:4760` text goes.
- Completes: EXP-4. Tests: EXP-4 w5. Design: none mapped (R2's `word = line.to_lower().trim()` cases).
- Depends: the borrow area's liveness of views (regions.rs), `exists as` (opening area).
  Cost: none at run time; the drop moves earlier or later, never doubles.

**A1-19 A lost change on a temporary is E2140** (S)
- Changes: typeck at the method-call site (`lib.rs:16395-16449` family): a method that may change its
  object, called on a temporary ending with the line, whose result is unused, is E2140.
- Completes: EXP-5. Design: the 3 E2140 tests (R8). Depends: the effects area ("may change its object").

**A1-20 Kind tests `x is T`, `if x is T as name:`** (M)
- Changes: parser `body.rs:948-951` (`is` followed by a type), typeck `lib.rs:58395-58402`: one type-id
  compare on `dyn I`, `owns dyn I`, `shares dyn I` or a base class; E2036, E2037.
- Completes: EXP-9. Depends: the dyn/cast area (DSP-4, GRM-45), which may own this item.

### Slice 10: needs the effect summaries

**A1-21 Ending a value counts as a call of its `drop`** (M)
- Changes: `ember_analysis/src/effects.rs:1202-1250` (`drop_sites`/`drop_of`) feed the new change/read
  summary, not only effect bits: every place a value of a type with `drop` may end (scope end, `clear`,
  replacement, an `owns dyn I` or function value replaced) adds that `drop`'s changes.
- Completes: DRP-8. Design: tests in R1/R8 with `fn drop` pushing to a module variable.
- Depends: the effects area's summary (shared decision of that area), A1-17.

**A1-22 One whole variable under two names: a copy of the callee** (L)
- Changes: a MIR pass after the borrow check finds calls proved to name one whole variable twice (two
  `ref mut` arguments of one local or static, or a `ref mut` argument naming a module variable that the
  callee uses by name, transitively); clones the callee's `Body` with the two places merged (shared
  decision D5); mangles and deduplicates it; re-runs the pass on the copy so its own calls are covered;
  the borrow area's E3022 exempts these calls; `ember inspect` lists each copy.
- Completes: MONO-10. Tests: OWN-6 w1 (`mem.swap(ref mut v, ref mut v)`: a builtin, so the "copy" is
  the self-swap no-op). Design: R4 `add_both` (1), the `rename(ref mut LOG)` cases.
- Depends: the effects area (which module variables a callee uses), A1-17, A1-8.

### Not in this area's items

HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, HEAP-10 (and their 18 old tests) belong to the shares/threads
work; OWN-12's hand-over to a C parameter is the FFI area's (A1-11 below is only `into_raw`).

**A1-11 `into_raw()` built in, only inside `unsafe:`** (M)
- Changes: typeck: `x.into_raw()` on an `owns` value or a class object, E3100 outside `unsafe:`, ends
  `x` and gives `*mut T`; codegen: no drop for it; replaces `ForeignBox.into_raw` (`std/src/ffi.em:177`).
- Completes: OWN-12 (the `into_raw` half). Design: 3 tests. Depends: A1-6, A1-9, A1-10; the FFI area.

## 3. Shared decisions

* **D1. `owns T` is today's `Box` representation.** One pointer to a block holding `T` (`{data,
  vtable}` for `owns dyn I`), built by the existing `BoxNew`/`DynBoxNew` builtins, freed by the existing
  glue. Only its name (and the C typedef) changes. Cost: as C's `T*` plus `malloc`; no header, no count.
  Other areas (shares, FFI's foreign `owns`) build on this, not a new struct.
* **D2. A number handed over is a `Move` of a Copy type.** `Operand::Move` on a Copy type copies bits
  exactly as `Copy` does; only the move analysis treats the source as ended. No new MIR form; zero cost.
* **D3. Which managers carry an instance word.** Proposal: a manager type with no fields (zero-sized:
  `Global`, `System`) has one instance; the compiler makes `A()` where it needs one and the value holds
  nothing. Any other manager type may have several instances, and a value made in one holds a pointer to
  it (one word, verdict 47), placed last in the value. Cost: zero for the default; 8 bytes per value or
  list header otherwise, as C++ `std::pmr` (see Q2).
* **D4. Allocation through a named manager is monomorphised.** `owns T using A` and `Array[T] using A`
  call `A.allocate`/`reallocate`/`deallocate` as direct calls on the stored instance pointer (no
  vtable, no function pointer); the runtime `ember_vec_*` paths stay for `Global`. Cost: a direct call
  per allocation, as a C program passing its arena; the default path unchanged.
* **D5. A MONO-10 copy is keyed by (callee instance, alias partition).** The partition says which `ref
  mut` parameters are one place and which parameter is which module variable; the copy is a clone of
  the callee's MIR `Body` with those locals merged (a merged module variable replaced by the static
  itself), mangled with the partition (MNG-1/MNG-6) and deduplicated. Cost: code size only.
* **D6. A `static mut` without heap memory is a plain C global; one with heap memory goes through the
  first-access accessor** that `static` already uses (writable storage). Its once-guard must stay
  atomic because scoped tasks may make the first access. An as-if refinement, for a const-evaluable
  initialiser in a program with an entry: build it before `main`'s first statement (no observable
  difference, no per-access branch). Statics are never dropped (CT-5).
* **D7. A value made `using inst` holds a region on `inst`** in the borrow checker (the same kind of
  region a view holds), with an exemption for the compiler's own `Allocator` calls and `alloc*`. ALC-5,
  ARN-7 and LT-4's "lives no longer than the instance" then fall out of the borrow area's model rather
  than a new mechanism.
* **D8. Promoted temporaries (EXP-4) are hidden block locals** ended after the last use of every view
  made from them, using the borrow area's view liveness; a drop flag only where paths differ.

## 4. Questions for the owner

* **Q1. Is `owned` still the contract word inside `@ffi(param(…))`?** `[LEX-…]` (Part II, the list of
  removed words, `p02-lexical.md:107-109`): "**Reserved and not words of Ember** (… any use is `E1010`
  …): `owned` (write `owns` …)". But `[FFI-11]`'s table (`p16-ffi.md:272`) keeps "`owned` (callee takes
  it)" as an ownership axis word, and `p16-ffi.md:501`, `:519` write `@ffi(param(widget, owned, one))`
  (as verdict 91 does). Blocks A1-1: the lexer must either refuse `owned` everywhere or keep today's
  exception inside `@ffi(…)` (`ember_parser/src/body.rs:1173-1182`). Proposal: keep the exception inside
  `@ffi(…)` contract lists only.
* **Q2. Which memory managers "may have several instances"?** `[ALC-1]`: "A manager with one instance
  (`Global`, `System`) needs none on the making line … A value whose manager may have several instances
  keeps its instance's address, one word". The rule names two examples and no test for a user's type.
  Blocks A1-14 (layout and the making-line check). Proposal: D3, a manager type with no fields has one
  instance.
* **Q3. What is `HOOK = v` in a function when `HOOK` is a plain `static`?** `[GRM-4]`: "`x = e` where no
  `x` is in scope declares `x` … where `x` is in scope it assigns". `[STA-1]`: a `static` "never
  changes", and a `static mut` is changed "by its name, with no word". So a function's `HOOK = …` assigns
  the module's `HOOK` and must be refused, but no code is named for a change of a plain `static`
  (today the line declares a local that shadows it, D-692). Proposal: E3023 ("`HOOK` is a `static`,
  which never changes; for a module variable that functions change write `static mut HOOK …`; for a
  local of your own use another name").

## 5. Risks

* **R1. Class objects losing `Copy` (A1-9) is the largest cross-cutting change.** Retain-on-copy,
  ODR-065's store-back, `uncounted_handles.rs` and `temporary_owners.rs` exist only because handles are
  Copy; every class test, std's class-based modules and the ECS rely on it. The ownership of this item
  between this area, a classes area and the shares area must be settled before work starts.
* **R2. Silent behaviour changes, not just new errors.** BRW-12 (`x = p.f` is a view) and EXP-4 (a
  temporary lives to its views' last use) change drop order in programs that compile under both models.
  Eleven EXP-6 and five EXP-4 old tests flip; std and examples may change output without failing to
  compile. Mitigation: run the old run-pass suite with drop-printing types after A1-4 and A1-18, compare
  outputs, and read each difference.
* **R3. Respelling volume.** `owned` in 319 `.em` test files and 54 `.rs` files, `mut self` in 546
  `.em` and 89 `.rs`, `Box` in 138 `.em` and 48 `.rs`, `Shared[` in 120 `.em`; `std` has 227 `owned`,
  18 `Box` and about 1,900 lines with `self`/`owned`/`mut self`. A mechanical rewriter (in `ember_fmt`,
  driven by the parser) is cheaper than hand edits; it is not in these items' sizes.
* **R4. MONO-10 interacts with generics and recursion.** A copy's own calls may need copies; recursion
  through a merged parameter needs the dedup key to be reached before the body is cloned. Code size grows
  per pattern; nothing measured.
* **R5. Static mut and hot reload.** `reload.rs:337` keeps static storage in a table across reloads;
  writable heap-owning statics add state that a reload must keep or migrate. Not examined further.
* **R6. Design tests that contradict the spec.** The 11 R3 `LevelArena` tests call
  `g.deallocate(p, layout)` and `g.reallocate(…)` outside `unsafe:`, but `[ALC-2]` now makes both
  `unsafe fn` and a program's own call "is made inside `unsafe:` (E3100)". The tests need an `unsafe:`
  block (the spec is clear; not a question).
* **R7. Unverified.** (a) Whether a field taken out of a non-`drop` body of a value whose type has its
  own `drop` is refused anywhere today: E3010 is only emitted inside `drop` (`drops.rs:746`), and
  `MovePaths::add` (`drops.rs:111-138`) keeps such types atomic, so such a move may be treated as a
  whole-value move; could not run it. (b) The location of FN-8's script-local E1010 help. (c) The C
  backend's handling of `Operand::Move` on a Copy type (D2 assumes a plain copy). (d) The LLVM backend's
  `Box` sites were not listed one by one.
* **R8. The rule map undercounts.** It maps most of these rules to zero design tests; the content counts
  (90 `static mut`, 130 `copy of`, 239 `owns`, 32 `fn drop`) show how much of the design suite each item
  gates.
* **R9. Stray build output in the test tree:** `tests/conformance/FN-6/target/debug` (disk footprint).

Confidence: **medium**. The per-file test classification is by hand and should hold; the compiler
citations were checked by search, not by running; sizes are estimates against a 68k-line checker and a
38k-line analysis crate.
