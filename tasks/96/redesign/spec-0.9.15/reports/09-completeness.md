# Report 09: completeness check (phase 2)

Checker 09. Read in full: `JOB.md` (verdicts 1 to 99 and the sections around them), `V5/rules.md` (R1 to
R18, sections 0 to 4), the lead's changes at the end of `V5/changes.md`, `V5/unsafe-inventory.md`, the
crosswalk, the eight writers' reports, and every part `PARTS/p00` to `p24` as the writers left them.
Nothing was fixed. Nothing was compiled or run; `check_citations.py` was run once (its only undefined
IDs are in `p27`, the lead's). Scratch: `target/redesign9/09/` (`files.txt`, the list of parts checked;
`blocks.py`, a scan of every ` ```ember ` block for replaced spellings).

Line numbers are those of the parts at the time of this check. "Missing" means no part says it; "not the
same" means a part says something else.

## 1. Result in one paragraph

The rework is almost complete. Every rule R1 to R18 has a home, every verdict that still has a meaning
is applied, and every inventory fix C01 to C34 and U1 to U3 is written. No `Box`, `Shared`, `Weak`,
`RefCell`, `SyncShared`, `ForeignBox`, `@sync`, `@borrows`, mode `owned`, `mem.forget`, `upgrade()`,
`cast ... if`, effect clause, `with` before captures or run-time index compare is left, except in the
lines that refuse them or migrate them. The failures are 28 (F-1 to F-18; F-3 has ten items and F-4
three), listed in section 6: one design rule with no rule text (E2200), one cost row not carried, ten
refusal messages of section 3 not carried, the spelling of the unsafe export attribute disagreeing
between four parts, one run-time exclusivity check left in the ECS library (raised for the owner), and
several pairs of parts that say different things about one case (the bounds stop text, the
`thread_shared` count limit, four registry rows, module-level statements, the enforcement ladder).

## 2. Check 1: every rule of `V5/rules.md` is in the spec

| V5 | where the spec says it | verdict of this check |
|---|---|---|
| section 0 terms | VII.1 (p07:12-53), Appendix F, `[PHIL-16]`, `[GRM-40]`, `[GRM-41]` | present |
| R1 | `[OWN-1]`-`[OWN-3]`, `[OWN-6]`, `[OWN-7]`, `[OWN-12]`, `[FFI-54]` | present |
| R2 | `[BRW-12]`-`[BRW-15]`, `[OWN-11]`, `[GRM-4]`, `[TYP-14]` | present |
| R3 | `[OWN-8]`-`[OWN-10]`, `[OWN-13]`, `[TYP-45]`, `[TYP-46]`, `[ALC-1]`, `[GRM-42]`, `[GRM-43]`, `[CLO-2]` | present |
| R4 | `[FN-1]`, `[FN-1a]`, `[FN-2a]`, `[BRW-11]`, `[BRW-21]`, `[BRW-22]`, `[LT-47]`, `[MONO-10]` | present (one message missing, F-3a) |
| R5 | `[FN-4]`, `[GRM-41]`, `[EFF-24]`, `[EFF-25]`, `[EXC-15]`, `[BRW-10]`, `[BRW-15]`, `[CLS-2]`, `[CLS-12]`, `[DRP-8]` | present |
| R6 | `[LT-1]`, `[LT-1a]`, `[CORO-14]` | present |
| R7 | `[LT-45]`, `[LT-46]`, `[TYP-15]`, `[TYP-34]` | present (one message missing, F-3f) |
| R8 (a)-(g) | `[BRW-16]`, `[BRW-17]`, `[BRW-18]`, `[BRW-19]`, `[BRW-20]`, `[BRW-1]`, `[CTL-2]` | present (messages missing, F-3c, F-3d, F-3e) |
| R9 | `[RC-1]`, `[RC-4]`, `[RC-7]`, `[OBJ-1]`, `[THR-13]`, `[CELL-13]` | present |
| R10 | `[RC-5]`, `[BCK-7]` | present |
| R11 | `[WK-11]`, `[WK-12]`, `[WK-16]`, `[WK-2]`, `[WK-3]`, `[WK-17]`, `[WK-18]`; L3001 replaced by verdict 99 (`[WK-6]`, E3032) | present |
| R12 | `[CLO-1]`-`[CLO-19]`, `[EFF-2]`, `[EFF-26]`, `[FFI-21]`-`[FFI-23]`, `[FFI-52]`, `[FFI-53]`, `[THR-2]` | present (messages missing, F-3b, F-3g) |
| R13 | `[GRM-44]`, `[GRM-19]`, `[CTL-11]`, `[BRW-14]` | present (E0115's message not given, F-3j) |
| R14 | `[TYP-44]`, `[DSP-4]`, `[DSP-6]`, `[GRM-30]`, `[GRM-45]`, `[TYP-17]`, `[TYP-22]`, `[IFC-5]`, `[IFC-6]` | present |
| R15 | `[THR-1]`-`[THR-21]`, `[PAR-1]`, `[PAR-2]`, `[JOB-2]`, `[CORO-6]`, `[FFI-53]` | present (one message missing, F-3h) |
| R16 | `[TIER-1]`, `[UNS-1]`, `[UNS-7]`, `[UNS-8]`, `[UNS-11]`, `[UNS-12]`, `[OWN-12]`, `[FFI-54]`, `[EFF-12]`, `[DSJ-6]` | present |
| R17 | `[BLD-14]`, `[EFF-4]`, `[MOD-9]` | present |
| R18 | `[DIA-2]`, `[DIA-25]`-`[DIA-28]`, `[PHIL-16]` | present |
| section 2, costs | `[COST-3]` | one row missing (F-2) |
| section 2, stop texts | `[COST-6]`, `[PRF-4]`, `[DSJ-5]`, `[DSP-6]`, `[RC-1]`, `[WK-11]`, `[FFI-53]`, `[TST-31]`, `[RT-7]` | present, but the bounds text and the `thread_shared` limit disagree between parts (F-7, F-8) |
| section 3, refusals | each rule's own part; registry `p17a` | E2200 missing (F-1); ten messages missing (F-3) |
| section 4, codes | `p17a` | every design code present with its meaning; three rows disagree with their rules and one code is missing (F-9 to F-12); E0100's help disagrees with `[GRM-2]` (F-14) |

## 3. Check 2: every verdict 1 to 99

Applied, with the spec ID that carries each: 2 (`[BRW-1]`), 3 (`[FN-2a]`, `[THR-3]`), 4 (`[RC-7]`,
`[OWN-13]`), 5 (`[BRW-1]`), 7 (`[FN-4]`), 8 (`[STA-1]`), 9 to 11 (`[WK-11]`, `[OWN-8]`), 12 (`[OWN-8]`,
`[LEX-15]`), 13 (`[BRW-19]`), 14 (`[WK-17]`), 15 (`[WK-18]`), 16 (`[BRW-18]`, `[CORO-14]`), 18
(`[EXC-15]`, `[BRW-22]`), 19 (`[TYP-45]`, `[TYP-46]`), 20, 21 (`[STD-19]`, `[EFF-26]`), 23 (`[RC-7]`), 24
(`[ALC-1]`), 26 (`[GRM-44]`), 27 (`[TYP-44]`), 28 (`[WK-18]`), 30 and 52 (`[THR-10]`, `[THR-19]`), 31 and
39 (`[FFI-52]`), 32 (`[FN-1]`), 33 (`[STD-19]`), 34 (`[FN-1a]`), 38 (`[CLO-14]`), 42 and 70 (`[BRW-13]`,
`[FN-2a]`), 43 (`[THR-19]`), 44 (`[THR-18]`), 47 (`[LT-4]`), 48, 49 and 51 (`[CLO-3]`, `[GRM-28]`), 50
(`[GRM-40]`), 53, 54, 87 (`[BRW-21]`), 55 and 82 (`[RC-5]`), 56 (`[WK-3]`), 57 (`[DRP-8]`), 58 (`[OWN-6]`,
`[THR-6]`), 59 (`[RC-1]`), 60 (`[TIER-1]`, `[UNS-11]`), 61 (`[CLO-16]`), 62 (`[FFI-53]`, `[CLO-18]`), 65
(`[THR-20]`'s example), 66 and 67 (`[GRM-44]`, `[BRW-14]`), 68 (`[STR-3]`), 71 (`[CLO-19]`), 72 and 95
(`[BRW-18]`), 73 (`[DIA-28]`), 74, 85 and 97 (`[OWN-10]`), 77 and 92 (`[THR-21]`), 78 (`[OWN-11]`), 79
(`[OWN-8]`), 80 (`[CLO-4]`), 81 (`[BRW-17]`), 83 (`[GRM-42]`), 84 (`[OWN-9]`), 86 (`[EFF-24]`, `[GRM-41]`),
88 (`[BRW-11]`, `[MONO-10]`), 89 (`[BRW-11]`), 91 (`[OWN-12]`, `[FFI-54]`), 93 (`[THR-17]`), 94
(`[THR-10]`), 96 (`[TST-31]`), 98 (`[BRW-5]`), 99 (`[WK-6]`, E3032); rows 3 to 9, the `self`/`this`
verdict and "error messages are part of readability" (`[BRW-10]`, `[BRW-19]`, `[BRW-20]`, `[CLO-2]`,
`[CLS-2]`, `[GRM-13]`, `[OWN-3]`, `[DIA-25]`).

Replaced or void, and correctly absent: 1 (by 26), 25 (by 27), 90 (by 95); the spellings voided by 86
(6, 17, 22, 29, 35 to 37, 40, 41, 45, 46, 63, 64, 69, 75, 76): no part writes `changes`, `reads`,
`captured`, a change list or `changelist`, and their meanings are carried by the worked-out summary.

Not fully applied: verdict 60's rule for C12 is written in Part XVI but contradicted by Parts III, V and
XI (F-4). No other verdict fails.

## 4. Check 3: every inventory fix

| fix | where | check |
|---|---|---|
| C01 | `[ALC-2]`, `[UNS-1]` | present |
| C02 | `[FFI-52]`, `[FFI-39d]` | present |
| C03 | `[THR-2]`, `[WK-11]`, `[THR-11]` | present |
| C04 | `[OBJ-5]`, `[CLS-7a]`, `[DRP-5]`, E3016 | present |
| C05 | `[CLS-2]` | present |
| C06 | `[TYP-43]` | present |
| C07 | `[FFI-56]`, `[FFI-8]` | present |
| C08 | `[HR-44]` | present |
| C09 | `[ECS-8]` | present |
| C10 | `[FFI-22]` | present |
| C11 | `[JOB-2]`, `[CLO-7]` | present |
| C12 | `[FFI-26]`, `[TIER-1]`, `[MNG-2]` | written, but contradicted elsewhere (F-4) |
| C13 | `[WK-18]` | present |
| C14 | `[ALC-5]`, `[ARN-7]` | present |
| C15 | `[EFF-27]`, `[CLS-12]`, `[HR-44]` | present |
| C16 | `[FFI-23]`, `[FFI-39c]`, `[HR-20]` | present; E5060's registry row still has the old meaning (F-10) |
| C17 | `[CT-5]`, `[STA-3]` | present |
| C18 | `[HR-46]`, `[HR-15]` | present |
| C19 | `[WK-1]`, `[WK-5]`-`[WK-7]`, E3032, L3001 retired | present |
| C20 | `[ARN-2]`, `[ARN-8]` | present |
| C21 | `[THR-15]` | present |
| C22 | `[RC-4]`, `[RT-8]`, `[OBJ-1]` | present |
| C23 | `[RC-4]`, `[RT-7]`, `[RT-10]`, `[COST-6]` | present; `[PRF-4]`, the glossary and `[TST-31]` still say the old rule (F-8) |
| C24 | `[THR-18]`, `[THR-19]` | present |
| C25 | `[FFI-51]`, III.1 grammar | present; its safety note is not recognised by `[LEX-23]` (F-5) |
| C26 | `[FFI-55]`, `[GRM-35]` | present |
| C27 | `[GPU-12]` | present |
| C28 | `[FFI-31b]`, E5054 | present |
| C29 | `[RT-12]`, `[CLI-2]` | present; its code E9014 is not in the registry (F-11) |
| C30 | `[FFI-53]` | present |
| C31 | `[FFI-33a]`, `[FFI-33b]`, `[PRF-4]` | present |
| C32 | `[STD-29]` | present |
| C33 | `[HND-3]` | present |
| C34 | `[BLD-FFI-5a]` | present |
| U1 | `[HR-45]` | present |
| U2 | `[HR-47]` | present |
| U3 | `[THR-20]` | present |

## 5. Check 4: nothing of the replaced model left

Searched every part `p00` to `p24` for `Box`, `Shared`, `Weak`, `RefCell`, `SyncShared`, `ForeignBox`,
`@sync`, `@borrows`, `@static_safe`, `@no_runtime_checks`, `@fp(contract)`, `owned`, `self`, `mem.forget`,
`upgrade`, `cast ... if|when`, `as?`, `as!`, `holds`, `copied`, `.clone()`, `Mut[`, `str`/`cstr` types,
`set by`, `borrow`, `loan`, exclusivity and access words, `i == j`, `) changes`, `) reads`, `changes this`,
`captured`, `changelist`, and `with` before a capture word; and scanned every ` ```ember ` block
(`blocks.py`). Every hit is a line that refuses or migrates the old form (E1010, E0104, E0100 messages,
`[CLI-20]`, the retired-code table), Python code in Appendix E, the type name `Self`, or the FFI contract
word `owned` (kept by verdict 91, `[FFI-11]`). Left over: the ECS run-time conflict check (F-6), the old
`@export(...)` spelling (F-4c), `cstr` in E5064's row (F-12), `@fp(contract)` in E4072's row (F-9), and two
examples that cannot pass `ember check` (F-13).

`Cell` is kept on purpose: the brief and the crosswalk list it among the replaced words, but `V5/rules.md`
keeps it (R5 counts "a `Cell` field" among what a plain name never changes; E2117 refuses a `Cell`
field) and the vocabulary's row says "keep", set only through a name that may change its holder. By the
brief's order of truth the spec is right to keep it (`[CELL-1]`); it is not counted as a failure.

## 6. Failures (part, line, why)

**F-1** p04:286-295 (`[TYP-14]`, which p17a:105 cites for E2200). V5 section 3's row "a type that contains
itself, or a view of its own type, with no block of its own between" (E2200, with its two messages:
"`Tree` contains itself through `left` ... `left: Option[owns Tree]`" and "`Node.parent` would be a view of
the `Node` that holds this one ... `parent: weakref Node = None`") has no rule text in any part; only the
registry row names it.

**F-2** p10:283-330 (`[COST-3]`). V5 section 2's row "a second pass of a loop, then `retain`, only where
AN-04's fusion conditions fail" is not in the cost table, and no part says the compiler fuses the two
passes.

**F-3** Refusal messages of V5 section 3 not carried (the rule is there, the message the design gives is
not, in any part):
- a. E3021 `refill` ("`refill` may replace the lists of `w1.lists` through `dst` while `src` shows numbers
  inside one of them ..."): p07:285-303 (`[BRW-11]`) gives only the `blend` message.
- b. E3020 `scores.for_each(...)` ("pushes to `LOG` (through the function it is given) ... move line 4
  below the loop"): p15:310-313 (`[STD-19]`) and p10:136 (`[EFF-26]`) give the case, not the message.
- c. E3020 `h(this)` / `close_all` (walk the handlers by position, `handlers: Array[fn(ref mut Ui)]`):
  p07:445-480 (`[BRW-18]`).
- d. E3060 `bag.items.push(k)` / `h.view` (a view put in through `ref mut`): p07:658-665 (`[LT-47]`).
- e. E3060 `m.insert(...)` / `ks` with `.copy_each()`: p07:424-444 (`[BRW-17]`), p15:38-42 (`[STD-26]`).
- f. E2118 "`buf` is stored inside the value ... `let buf: String`": p07:634-639 (`[LT-45]`), p04:321-325.
- g. E3026 the `Button` total ("keep the total in an object the handler can own a share of ... `Counter`"):
  p06:285-296 (`[CLO-4]`).
- h. E7015 AN-78 ("C may call this function from any thread (`threads(any)`) ... share the game between
  threads"): p16:398-400 (`[FFI-22]`) describes the message instead of giving it.
- i. E2117 "`radius` reads `mesh`, which its other owners may change without `Mesh` seeing it":
  p05:376-378 (`[CLS-12]`) gives a different message for that case.
- j. E0115 ("an `Option` or a `weakref` link is opened with `exists as`: write `if ui exists as ref mut
  me:`"): p03:249-250 (`[GRM-19]`) and p03:273-274 give the code without the line to write.

**F-4** C12 (verdict 60): the unsafe form of the export attribute disagrees across parts.
- a. p05:524 (attribute table) lists the plain `@export("symbol")` and `@export_table(...)` and no
  `@unsafe(...)`, while p05:494-495 (`[ATT-1]`) makes every attribute not in the table `E0104`. So
  `@unsafe(export("name"))` is `E0104` by Part V and the required form by p16:568-583 (`[FFI-26]`), which
  makes the plain form `E3100`.
- b. p03:51-52 (`attribute := "@" attr_name`, `attr_name := IDENT {...}`) cannot produce `@unsafe(...)`,
  since `unsafe` is reserved (p02:83); yet p03:8-9 says every construct of the spec appears in Part III,
  and p01:37 (`[TIER-1]`), p16:563 (example), p18:85 (`[MNG-2]`) and p21:211 (`[TCB-4]`) use it. Agent 03
  raised this; nothing settled it.
- c. p11:278 (`[THR-20]`) still writes `@export(threads=any)` and `@export(threads=creator)`, against
  p16:606-607 (`[FFI-33]`: `@unsafe(export("score", threads=any))`).

**F-5** C25 and C12's safety notes: p16:122-123 (`[FFI-51]`) and p16:579-580 (`[FFI-26]`) ask for a
`# SAFETY:` note above `unsafe import c` and `@unsafe(export(...))` (`W3012` when missing), but p02:52-54
(`[LEX-23]`) records a safety note only before an `unsafe:` block or `unsafe extend`/`extern`/`overlay`,
and p09:289-292 (`[UNS-8]`) and p09:308-311 (`[UNS-11]`) list only those; by Parts II and IX such a note
is no note and `W3012` would always fire.

**F-6** p12:220-222 (`[ECS-4]`): "Systems passed as values, whose sets are not known at compile time, are
checked against each other before any of them starts, and a conflict panics." This is a check of
exclusive access made while the program runs, which the rework removes (p01:216 `[BRW-1]` non-goal,
p17:326 `[PRF-4]`: "No check of exclusive access runs while a program runs"). R12's rule (what a
function value does is worked out from every function the program can put there) would let it be
refused when compiling (`E7020`), but that refuses programs that run today. Agent 05 kept it and raised
it for the owner; it is in `for_owner`.

**F-7** The bounds-check stop text says three things: p10:355 (`[COST-6]`, V5's text) "index 3 is out of
bounds for a length of 3"; p13:121 (`[ERR-13]`) "index 7 out of range for length 3; use .get(i) for an
Option", which p17:340 (`[PRF-4]`) adopts; p04:161 (`[TYP-31]`) "`index out of bounds`". `[COST-6]` and
`[PRF-4]` both claim to be the table of stop texts.

**F-8** The count limit of a `thread_shared` value (C23): p08:128-134 (`[RC-4]`) and p18:261-264
(`[RT-7]`) raise it with one atomic add and stop when the value before the add is 2,147,483,647 or more
("too many owners of a `Player` (2,147,483,647)"); but p17:322-323 (`[PRF-4]`) says every count is
compared "with its maximum before each +1" and its table (p17:334-335) has only the 4,294,967,295 texts;
p24:14 (glossary) and p01:153 (I.4 table) say the same as `[PRF-4]`; and p17:435-442 (`[TST-31]`) says the
8-bit test build stops after 255 for every count, where p18:267-268 (`[RT-7]`) says 127 for a
`thread_shared` value.

**F-9** p17a:161 (E4072) "`@fastmath` or `@fp(contract)` inside `@deterministic`", against p10:261-265
(`[DET-5]`): it is `@fp(allow_fma)`, and `@fp(contract)` is `E0104`.

**F-10** p17a:191 (E5060) "upcast in a context that cannot hold the `Retained` token" (the object form C16
removes), against p21:136-138 (`[FFI-39c]`): E5060 is an object of an `owner="ember"` class given to a
parameter that keeps it.

**F-11** p18:291 (`[RT-12]`, C29) names `E9014` (a toolchain that cannot emit stack probes); p17a has no
row for it, against p17:491-493 (`[DIA-6a]`: every code the document names is registered).

**F-12** p17a:195 (E5064) "interior NUL in a value converted to `cstr`": `cstr` is a replaced word
(`CStrView`), and p15:113 (`[TXT-3]`) gives E5064 to a `c"…"` literal containing a NUL, a different case.

**F-13** Two examples cannot pass `ember check` though marked ` ```ember ` (p00:96-97 and p17:388-392
`[TST-7]` require that, or ` ```ember,ignore ` with a reason for a deliberate error): p08:339-347
(`[WK-18]`'s walk: statements at file level using `leaf` and `this`, which nothing declares); p08:381-391
(`[WK-6]`: its last line is the refused one, `E3032`).

**F-14** Statements at module level, three readings of one case:
- p03:28-31 (`[GRM-2]`) gives a statement at file scope outside the entry file `E0100` with the help "move
  it into a function"; p05:463-466 (`[STA-1]`) and p17a:17 give the same case `E0100` whose help is
  "`static mut FRAME: int = 0`".
- In the entry file, p03:28-30 (`[GRM-2]`) and p05:185-192 (`[FN-8]`) make such statements the implicit
  `main` (a function using one of its names is `E1010`), and Appendix E's program (p23:140-152) relies on
  it; p05:463 (`[STA-1]`) says "A statement at module level that would give a module variable a value is
  `E0100`" with no exception for the entry file. V5's E0100 row says nothing of scripts. Agent 08 raised
  it for the lead.

Lower-severity points, each a pair of parts that disagree, for the lead:
- **F-15** p24:19 (glossary, "enforcement ladder": "prove statically, else check at run time, else require
  `unsafe`") against p01:64-73 (`[PHIL-8]`: a program the compiler cannot prove is refused with the line
  to write; a run-time check only for run-time data).
- **F-16** p24:47 (glossary, "proof-carrying value ... `check_disjoint` ... carries the fact") against p09:449-451
  (`[DSJ-2]`: the fact belongs to the two names; `check_disjoint` returns `Result[void, void]`).
- **F-17** p17:320-326 (`[PRF-4]`) presents the list of checks that run while a program runs, but omits ones other
  parts define in every profile: the double `adopt` check (p16:345-347, `[FFI-36b]`), the panic converting
  a function value with state to `extern "C" fn` (p06:260-262, `[CLO-3]`), and the draw-range checks
  (p22:58-71, `[GPU-12]`); none of these has a `[COST-3]` row either (p10:340-341, `[COST-5]`).
- **F-18** p07:217 (`[BRW-14]`) and p08:288 (`[WK-12]`) cite `[GRM-19]` for `exists as`, which is `[GRM-44]`.

## 7. Check 5: rules that contradict each other

All found are in section 6: F-4, F-5, F-7 to F-10, F-12, F-14 and F-15 to F-18. Areas compared
and found consistent: `copy of` on links and on structs holding links (`[OWN-8]`, `[TYP-36]`, `[STR-5]`,
`[DRV-1]`); `thread.spawn`, `scope.spawn`, `jobs.submit` signatures (`[THR-10]`, `[THR-11]`, `[JOB-2]`,
`[CLO-7]`); noalias and `this` (`[BRW-22]`, `[SIMD-3]`, `[CG-C-4]`, `[CG-LL-3]`); the object header
(`[OBJ-1]`, `[RT-7]`, `[HR-12]`); `@no_alias_checks` and both disjointness checks (`[EFF-12]`, `[DSJ-6]`);
the call check and its tie-break (`[BRW-21]`, `[BCK-8]`, `[DIA-28]`); E7018 and E3020 inside a lock
(`[BRW-18]`, `[THR-19]`, `[DIA-28]`); `exists as` and kind tests (`[GRM-44]`, `[GRM-45]`, `[TYP-44]`,
`[DSP-4]`, `[CTL-11]`, `[EXP-9]`); arenas and `using` (`[ARN-1]`, `[ALC-1]`, `[ALC-5]`, `[LT-4]`, `[TYP-46]`);
the reduction rules (`[THR-21]`, `[DET-2]`, `[LT-46]`).

## 8. For the owner

One point, raised by agent 05 and not settled by any verdict: `[ECS-4]`'s run-time conflict check between
systems passed as values (F-6). Keeping it keeps a check of exclusive access while the program runs;
replacing it with a refusal when compiling (from every system the program can put in the list) refuses
programs that run today.
