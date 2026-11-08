# Round 2: what was applied, and how each result was judged

Inputs: `round2/inputs/` (76 clusters K01–K76, 47 new gaps NG-01–NG-47, five design questions).
Results: `round2/results/workflow-result.json` (44 agents: 10 readers, 10 refuters, 14 designers,
9 judges, 1 critic), digested per group in `round2/results/digest-*.md`, `ruling-*.md`, `critic.md`.
Scripts: `scratchpad/edit20.py` onward (asserting, whitespace-insensitive anchors against the parts
at HEAD, not the stale `CURRENT-SPEC.md` snapshot the agents read; see critic missing-1).

Legend: **applied** (the refuter's better text where it gave one, else the reader's); **dup** (already
at HEAD from Hardened_2 or the lead fixes); **skipped** (refuted, or not a defect); **owner** (recorded
for the owner, text left safe as it is or made safe).

## R1 (Parts I–IV) — edit20.py

| Cluster | Reader / refuter | Decision |
|---|---|---|
| K18 string `+` | answered_differently / narrowed | **applied** (better edits): `a + b` on a `String` reads both operands and makes a new `String` (TXT-11, TYP-21 twice, the §X.4 Allocation row); every operator now reads its operands, as GRM-47 and the E0119 row say. Lead ruling, no verdict touched (JOB.md has none on text `+`). |
| K28 untyped list of text | answered_differently / confirmed | **applied**: TYP-38 says a context-free list keeps the elements' own type (`Array[StrView]`). |
| K30 owners' equality and `is` | not_answered / confirmed | **applied** (6 edits): TYP-14, TYP-5 rule 7 (owners read through), TYP-36 (owners have the value's `Eq`/`Ord`/`Hash`/`Display`/`Debug`), EXP-9 and the VI.3 row (`is` on owners and links), E2150 row (registry). |
| K41 `Some` after conversions | answered_differently / narrowed | **applied** (better edit): TYP-5's composition sentence: rule 7 before any rule, rule 11 after any one rule. Reverses the old "rule 11 after rules 1–4" list (predates rules 8–12); no verdict. |
| K45 `dyn` with `Self` parameter | answered / confirmed | nothing to change; the missed `owns dyn I` clause applied (below). |
| K59 `runtime` placement | answered_differently / confirmed | **dup**: the `lambda_param` grammar line was already at HEAD (edit19, J1-17). |
| K62 `copy of` on loop names | answered_differently / confirmed | **skipped**: decided by the grammar (two-line form, verdict 66). |
| K64 list pattern arms | answered_differently / narrowed | **applied** (better edits): GRM-13 gains the list pattern's meaning; grammar `primary_pattern` gains `"[" ".." [IDENT] "]"`. Top-level arm `["quit"]:` stays `E0100` (verdict 155); owner question recorded (whether an arm's list pattern may open a line). |
| K65 several interfaces after `dyn` | not_answered / confirmed | **applied** (better edits): `bound_list` takes round brackets as `needs` does (`dyn (A, B)`), TYP-17 refuses `+`, TYP-22 defines the set's table and layout. Lead ruling extending verdict 109's sentence to `dyn`/`some` (reported). |
| K68 pack tuple sizes | not_answered / confirmed | **applied**: grammar `tuple_type`/`tuple_elem`/`fn_type_param` take a spelled-out pack; TYP-47 says what a tuple over a pack is at sizes 0, 1 and n. |
| K70 raw identifier | answered / confirmed | **skipped** (not a defect). |
| K71 nested tuple access | answered / confirmed | **skipped** (LEX-18 decides it). |
| missed: literal element hands over | — | **applied**: OWN-9. |
| missed: `owns dyn I`'s interfaces | — | **applied**: TYP-36 (`Display`, `Debug`, `Hash` through `dyn`; never `Ord`). |
| missed: `Option[Cs]...` expansion | — | **applied**: TYP-47 refuses C++'s pattern expansion (`E0100`). |

Registry rows to update (p17a_0915.py): E2150 (owners added).

## R2 (Parts V–VI) — edit21.py

| Cluster | Reader / refuter | Decision |
|---|---|---|
| K12 function lent to std and kept | answered_differently / narrowed | **applied** (better edits): CLO-12 and GRM-28 spell `thread.spawn(owns push_a)`. The narrowing of verdicts 150 and 51 is already at HEAD (FN-2a, Hardened_2); reported. |
| K19 written `init` call words | not_answered / confirmed | **applied**: FN-2a and OWN-9: a memberwise constructor's or an `owns` `init` parameter's value is handed over with no `owns`; a plain `init` parameter reads; `ref mut` is written. Touches verdict 159 in a case it did not consider (a plain `init` parameter lends instead of handing over); reported, alternative text kept in the digest. |
| K22 opened name owns a call result | answered_differently / confirmed | **applied** the help improvement both agreed on (OWN-11's `E3013` help offers `match owns jobs.pop():`); the view meaning of an opened name stays. |
| K27 `for` over a named iterator | answered / confirmed | nothing; the missed BRW-15 clause applied. |
| K32 `super.init` on a memberwise base | not_answered / narrowed | **applied** (better edits): CLS-3 (the memberwise constructor is the base's `init` for every purpose; `super.init(…)`'s argument words), CLS-10. |
| K33 `init` and field defaults | answered_differently / confirmed | nothing (STR-6 decides it); the missed STR-2 bracket applied. |
| K40 function value's inline state | not_answered / narrowed | **applied** (better edits): CLO-10 (one frame address in the state word; a block where a word-capture meets real variables) and the §X.4 row. |
| K44 `drop` changes a `let` field | answered_differently / confirmed | **applied**: CLS-9a (the `let` check holds in `drop`); the missed DRP-1 cross-reference applied. |
| K66 constructor visibility | not_answered / narrowed | **applied** (better edit): STR-1 visibility across the three levels. |
| missed: one-word owner capture changed through | — | **applied**: CLO-10 (a change of the target through a link or under a lock writes the target, not the word; no block). |
| missed: BRW-15 "plain local that holds a view" | — | **applied**. |
| missed: DRP-1 `let` | — | **applied**. |
| missed: STR-2 written `init` | — | **applied**. |

## R3 (Parts VII–VIII) — edit22.py (K05 in the rulings' script)

| Cluster | Reader / refuter | Decision |
|---|---|---|
| K04 weakref outlives arena | answered / confirmed | nothing (ALC-5a decides it). |
| K05 `thread_shared … using` | not_answered / narrowed | **superseded by the judged K05 ruling** (shape B: allowed, counted away from the making line, `E7015` only where the value would hold a link to an instance not every thread may call). The reader's shape A (`E2114` for every manager) is not applied. Applied with the rulings; recorded for the owner (OPEN ODR). |
| K06 sharing a local | answered / confirmed | nothing. |
| K11 `unsafe fn drop` / override | not_answered / confirmed (hole) | **applied**: UNS-12 (a function safe code runs without naming it is never an `unsafe fn`; `E2020`); E2020 registry row. |
| K17 read-only parameter promise | answered_differently / confirmed | **applied**: FN-1 and BRW-21 state the across-a-call promise (end-or-move only; the lead's area-2 Q4 ruling). Narrows verdict 5 across a call; reported. |
| K23 `match` subject temporaries | not_answered / confirmed (hole) | **applied**: EXP-4 (a `match` subject's temporaries follow the condition rule), THR-19 (a temporary guard holds the lock), E7018 row. |
| K24 owning names changeable | not_answered / confirmed | **applied**: BRW-14 (two edits), BRW-15, CTL-1: a name handed its value owns it. |
| K25 `owns` takes a place out | not_answered / confirmed | **applied**: GRM-15, OWN-11 (`body = owns req.body`), TYP-44 (the words before a cast), DSP-6. |
| K26 copying a view | not_answered / narrowed (hole in proposed) | **applied** (better edits, option B): BRW-12 (a `Copy` part is copied by `=`), OWN-8 (E2116 only for `ref mut`/`MutSpan`; a read-only view is `Copy`, `copy of w` is one more view), TYP-36, E2116 row. Brings the text to verdict 68. The reader's PHIL-2 edit (an allocation) not applied. |
| K35 `weakref dyn` | answered_differently / confirmed | **applied**: DSP-4's list gains `shares mut dyn I`; missed TYP-45 (`sharedref mut dyn I` and the views of a `dyn` value) applied. |
| K42 split MutSpan halves | not_answered / confirmed | **applied**: SPN-3, BRW-12 (`left, right = ref mut xs.split_at(mid)`). |
| K43 weakref into `@cached` | not_answered / narrowed (hole in proposed) | **applied** (better edits): CLS-12 (a `@cached` body reads no field of a weak-counted class object reached through `this`'s fields), EFF-27, E2117 row; missed CLS-12 `static mut` clause applied, E2117 row. |
| K63 copied array/tuple changed | answered_differently / confirmed | **applied**: OWN-7 covers every `Copy` value with parts. |
| missed: part of a value with several owners | — | **applied**: OWN-11, EXP-6 (`E3013`). |

## R4 (Parts IX, X, XVIII, annexes) — edit23.py (K34 in the rulings' script)

| Cluster | Reader / refuter | Decision |
|---|---|---|
| K34 which types take `using` | answered_differently / narrowed | **applied with the K05 ruling** (merged text: the property, `Deque`, `SoA`, `SmallArray` named; `thread_shared` back in the list). |
| K54 Nondet contested lock | answered_differently / confirmed | nothing; missed E4070 row (registry). |
| K58 `runtime` check's effect kind | not_answered / narrowed | **applied** (better edits, option B: `RuntimeCheck(Aliasing)` only): EFF-9, EFF-12, DSJ-6, LT-48, COST-6 row, PRF-4, HR-35; E4030 row (registry). The reader's `Panic(Explicit)` half not applied. |
| K61 arena scope views | answered_differently / confirmed | nothing; missed E3096 row (registry). |
| K14, K48, K51, K60 | answered / confirmed | nothing. |
| K67 bool/char casts | answered_differently / confirmed | nothing; missed TYP-6 help applied. |

Registry rows so far: E2150 (K30), E2020 (K11), E7018 (K23), E2116 (K26), E2117 (K43 + static mut), E4070 (K54 missed), E3096 (K61 missed), E4030 (K58), E3013 (several owners, check row).

## R5 (Parts XI, XVII) — edit24.py (K10 in the rulings' script)

| Cluster | Reader / refuter | Decision |
|---|---|---|
| K01 `sharedref mut` and `Send` | answered_differently / confirmed | **applied**: THR-15 refuses a channel of a non-`Send` `T` where it is made (verdict 164's letter); E7015 row. |
| K02 `thread_shared` not `ThreadShared` | not_answered / narrowed | **applied** (better edits merged with the verdict-165 bullet at HEAD): THR-1 (type `Send` and `ThreadShared` judged by components; a `Receiver` is not; `thread_shared fn()`'s state is `ThreadShared` unless `once fn`), THR-9 (`thread_shared fn()`, `once fn`, `Receiver`), marker table row; E7015 row. |
| K03 task ends `shares` owners | answered / narrowed | E7015 row only (registry). |
| K10 queued drops and check points | not_answered / narrowed | **superseded by the judged K10 ruling** (applied with the rulings). |
| K36 `unsafe extend` vouching | not_answered / confirmed | **applied**: TYP-43 (two edits: a class may vouch `Send`; `ThreadShared` is vouched on a foreign type only); missed FFI-8 (`threads(creator)` never vouched `ThreadShared`, `E7008`). |
| K37 parallel array elements | not_answered / narrowed | **applied** (better edits): PAR-2 (c) admits an `Array`/`[T; N]` the body neither grows nor shrinks; PAR-2b. |
| K38 parallel reductions | answered_differently / narrowed | **applied** (better edits): PAR-2 (two), PAR-3 (two). |
| K39 reduction start values | not_answered / narrowed | **applied** (better edits): THR-21 (two). |

## R6 (Parts XII–XV) — edit25.py

| Cluster | Reader / refuter | Decision |
|---|---|---|
| K13 weakref to a static | not_answered / narrowed (hole) | **applied** (better edits): THR-2 (a link never names what every thread may reach), CT-5 (a `thread_shared` value in a static is placed writable, marked handed over); OBJ-1 clause with the K05 ruling; E7015 row. |
| K20 `AnyError` and `Send` (cluster) | answered_differently / confirmed | **held for the K20 ruling**, which changes `Error`'s requirements. |
| K29 text view in a list | answered_differently / narrowed→not_answered | **applied**: STD-8 (literal needle typing, `Contains[StrView]` for collections of `String`, help that compiles). |
| K49 `@ser(skip)` field value | not_answered / narrowed | **applied** (better edits): SER-1, §XIV.2 paragraph. |
| K55 Nondet math functions | not_answered / narrowed (hole) | **applied** (better edits): DET-2, STD-27; missed DET-4 (a `det` twin for every `Nondet` function, under `std.math`'s names; folds K73's `ln`). `std/src/math/det.em`'s `pub fn log` becomes a defect of the file (reported). |
| K56 `unsafe` at compile time | not_answered / confirmed | **applied**: CT-1, CT-2. |
| K69 SoA flattened field | not_answered / narrowed | **applied** (better edits): SOA-1, SOA-6. |
| K72 `comptime.read_file` | not_answered / confirmed | **applied**: CT-2. |
| K73 `det.ln` | answered_differently / confirmed | **applied** through the DET-4 edit. |
| K74 `AnyError` core layer | answered / confirmed | nothing. |
| missed: `dyn I` `ThreadShared` | — | with the K20 ruling (THR-9). |
| missed: `thread_shared T using A` | — | with the K05 ruling. |

## R7 and R8 (Part XVI) — edit26.py (K07 and K21 in the rulings' scripts)

| Cluster | Reader / refuter | Decision |
|---|---|---|
| K07 raw pointer in a struct | not_answered / confirmed | **superseded by the judged K07 ruling** (edit28.py). |
| K08 export arguments from C | not_answered / narrowed | **applied** (better edits): FFI-31b (a function pointer C hands an export is `unsafe extern "C" fn`), FFI-56; E5015 row. |
| K09 C++ reference result | not_answered / narrowed | **applied** (better edits): FFI-2a (a result that points somewhere needs a lifetime word, views included), FFI-17a rows, FFI-11a. |
| K15 foreign `const` globals | not_answered / narrowed | **applied** (better edits): FFI-2a (derived `const`), FFI-41, FFI-8 static row, STA-1; missed UNS-1 (read of a foreign static). |
| K16 `unsafe import cpp` | not_answered / narrowed | **applied** (better edits): FFI-51 (the property: code that runs uncalled), TCB-4, LEX-23, TIER-1. |
| K21 C++ overloads | needs_owner / narrowed | **superseded by the judged K21 ruling** (edit30.py); recorded for the owner. |
| K31 kept callback declaration | not_answered / narrowed | **applied** (better edits): FFI-21 (`owns cb: fn()` under `retained`/`once`), FFI-10, FFI-52 example, two messages. |
| K46 `@ffi(handle)` | not_answered / narrowed | **applied** (better edits): FFI-8 (a handle's representation and words), FFI-11, the Vulkan example (`device`, `commandBuffer`). |
| K47 overlay operator names | not_answered / narrowed (hole) | **applied** (better edit): FFI-42 (an operator is named by its interface method; unsafe until its entry); missed free-operator clause, W5033 row. |
| K50 `adopt` ends by own destructor | not_answered / narrowed | **applied** (better edits): FFI-36, FFI-36a; E5002 row. |
| K52 C keeps an address Ember owns | answered_differently / confirmed | missed THR-2 message fix applied. |
| K53 once-callback token | answered_differently / narrowed | **applied** (better edit): FFI-21 (a registering call returns the C result then the token, `[FFI-11b]`); the R7 missed variant not applied. |
| K57 export threads main/creator | answered / confirmed | nothing. |
| K75 overlay header mismatch | not_answered / narrowed | **applied** (better edits): GRM-35, FFI-55; E5010 row. |
| K76 callback table full | not_answered / confirmed | **applied**: FFI-53 (the table grows by chunks; a 32-bit target's 65,536 slots full stops the program), COST-6 row. The owner's rule on panics for a limit of the implementation is read as not applying (the limit is C's 32-bit pointer); reported. |
| missed (R7): `@ffi(std_function)` raw pointer | — | with the K07 ruling (FFI-17a row). |
| missed (R7): `@ffi(newtype)` over a pointer typedef | — | **applied**: FFI-8 typedef row. |
| missed (R7): hand-declared `type` takes the overlay type words | — | **applied**: FFI-10. |
| missed (R8): FFI-10 "64 bits" | — | **applied**. |
| missed (R8): ODR-166 row says `ForeignBox[T]` | — | in `tools/appx_h.py` (the row is generated). |

## R9 and R10 (the 47 new gaps) — edit27–30.py and the registry

| Item | Reader / refuter | Decision |
|---|---|---|
| NG-01, 02, 04, 05, 07, 09, 22, 42, 43, 44 | answered / confirmed | nothing. |
| NG-03 weakref's give-back | not_answered / confirmed | **applied**: TYP-46 (`weakref T using A`), ALC-5a, OWN-13, GRM-3; missed ALC-1 clause (a class made `using A` is a type of its own; `this` carries the kind). |
| NG-06 `thread_shared … using` | not_answered / narrowed | (2) superseded by the K05 ruling; (1) refuted. |
| NG-08 `using` binding | needs_owner / refuted | **skipped** (not a defect). |
| NG-10 return to C | needs_owner / narrowed | **applied** with the K10 ruling (a check point at a foreign call and at the return of every export and trampoline); recorded for the owner. |
| NG-11..13, 15..17, 19..21, 32..38, 40, 41 (the FFI direction) | not_answered / confirmed or narrowed | **applied** in edit28.py: the K07 limit (declarations) plus one [UNS-1] entry (R10's better form: giving foreign code a raw pointer is the unsafe operation), FFI-39, FFI-31b, FFI-8 struct row, FFI-8 typedef row, FFI-17a `std_function` row. NG-11's FFI-21a rewrite (provenance) not applied. |
| NG-14, NG-18 | not_answered / narrowed | **applied** (better): FFI-8 struct row, UNS-1. |
| NG-23 `dyn A + Send` | not_answered / narrowed | **applied under K65's spelling**: TYP-22 (a marker interface in a `dyn` list gives the marker); the `+` form is `E0100` (TYP-17). |
| NG-24 THR-1 and verdict 165 | answered / narrowed | covered by K02's merged THR-1 text. |
| NG-25 open class values and `Send` | not_answered / narrowed | **applied by the K20 ruling** (THR-8's class clause, THR-9, the marker rows); E7015 row. |
| NG-26 const/non-const pair | needs_owner / confirmed | **applied by the K21 ruling** ([FFI-40b]: two names, `at` and `at_mut`); recorded for the owner. |
| NG-27, NG-31 overloaded virtuals | answered_differently / narrowed | **applied through [FFI-57]/[FFI-39]** (one `override fn` per member). |
| NG-28 C aliases | not_answered / refuted | **skipped** (aliases, as the text says). |
| NG-29 `T&&` | not_answered / confirmed | **applied**: FFI-40; missed FFI-17f. |
| NG-30 override's first line | not_answered / narrowed | **applied** (better): CLS-4, new code `E2113`. |
| NG-39 forged C function pointer | not_answered / refuted | **skipped**; the K07 limit still counts a C function pointer the caller supplies in a struct (the judges' design 2). |
| NG-45 std message text | not_a_defect | nothing. |
| NG-46, NG-47 waking a waiting maker | answered_differently / narrowed | not applied alone; the K10 ruling decides the job-worker and return-to-C cases. |
| missed: `extern "C" fn` type's words | — | **applied**: FN-6. |
| missed: imported struct's function-pointer field | — | **applied**: FFI-56 (always `Option`). |
| missed: interface implementation's first line | — | **applied**: IFC-3 (`E2040`). |
| missed: `some I + Send` | — | **applied** under K65's spelling: TYP-32. |

## The five judged rulings — edit27.py (K05, K10), edit28.py (K07), edit29.py (K20), edit30.py (K21)

Each applied as its judge(s) merged it, reconciled with the readers' edits on the same rules (RC-4 point 4 carries K05's `using` path, K10's two marks and the `thread.scope` pin; OBJ-1 carries K10's chunk sentence, K05's `using` sentence and K13's static clause; TYP-46/ALC-1 carry K05, K34 and NG-03; THR-8/THR-9/TYP-43 carry K20 over K02 and K36; FFI-39 carries K21 and NG-19; the Vulkan example carries K46 and K07). Owner items recorded in `docs/OWNER-QUEUE.md` (ODR-260 onward) and in `decisions.md` §14.

Registry (p17a_0915.py, `code_registry.py --write`): rows E1010, E2020, E2040, E2080, E2114, E2116, E2117, E2150, E3013, E3096, E3100, E4030, E4070, E5002, E5010, E5015, E5056, E6005, E6010, E7007, E7008, E7010, E7015, E7018, E9030, W5033; new E2113 (CLS-4), E5035 (FFI-57); codes.rs gains E2113 and E5035. 271 live, 36 retired, 0 problems.
