# Readability check of run 4, step 2, v3 (agent 07)

Agent 07 (readability checker), run 4, step 2, phase 4. This file checks every NEW block of the five v3
area files, and the v3 rules, vocabulary and change list, against the owner's test, the error-message
rule, the owner's verdicts 1 to 52 (25 excepted, replaced by 27) and the agreed rows, after the lead's
phase 3 message fixes. It fixes nothing. `V3` means `tasks/96/redesign/run4/v3/`, `V2` means
`tasks/96/redesign/run4/v2/`; "R1" .. "R18" are the rules of `V3/rules.md`; "verdict N" is item N of
`inputs/JOB.md`, "Owner verdicts on run 4's open questions"; "row N" is the owner's agreed row N. Line
numbers are lines of the file named, as it stands at the start of phase 4.

No compiler change, no git command, no Ember program run, no `ember check` (this file makes no claim
about what today's Ember accepts). This step ran in a cloud container; no power reading was taken.

## 0. Terms

- **NEW block**: code an area file proposes under the v3 rules: any fenced `ember` block of an answer
  outside parts 2 (OLD) and 5 ("means exactly"), and the code blocks outside the answers (area D's two
  in "For the owner", the walk in R11). OLD code and "means exactly" programs are today's Ember, carried
  unchanged, and are not judged.
- **message**: the text of a refusal or a warning: in an answer's part 6 (inline, or as an `error[...]`
  block), inside its part 3, in `V3/rules.md` section 3, and in the vocabulary's "stays refused" column.
- **the owner's test**: for every name a line declares, the word on the line that says what it is (the
  owner's own thing, a copy, or handed over) and the word that says whether it may change it; a name with
  no word passes only where the stated default never changes anything, never aliases silently and never
  copies silently.
- **the error-message rule**: a message names what was refused in the program's own names, says why in
  one line, and gives the exact line to write, a line that compiles in that program under the rules; where
  it replaces several lines it names every one (R18).
- **twin**: a copy of a message, or of a rule's example, in another file (most often a vocabulary row
  that repeats a rules row or an answer's message).
- **CONFIRMED**: checked against the rule text, no reading needed. **PLAUSIBLE**: depends on a reading the
  rules do not settle in words.

## 1. What was checked, and how

- Read in full: `inputs/brief.md`, `inputs/JOB.md`, `inputs/duties.md` (section 07), `V2/readability.md`,
  `V3/phase3-messages.md`, `V3/rules.md`, `V3/vocabulary.md`, `V3/changes.md`, and every part of
  `V3/area-A.md` .. `V3/area-E.md` but the OLD and "means exactly" parts (read through the stripped copies
  below, which keep the original line numbers). Where a message was about an OLD program, that OLD part
  was read too (AN-50, AN-80).
- Scripts, all under `target/redesign4/v3-07-readability/`:
  - `extract.py` pulls every fenced block out of the five area files and labels it OLD, NEW or outside
    an answer (`blocks.txt`, `new_blocks.txt`): 479 blocks, **109 NEW `ember` blocks** (area A 25, B 19,
    C 22, D 20 and 2 outside the answers, E 21) and 47 NEW `text` blocks holding messages (areas C, D, E).
    With the one `ember` block of `V3/rules.md` (R11's walk), **110 NEW code blocks**.
  - `scan.py` searches the NEW blocks for every old spelling the duty lists and the forms v2's check
    searched (`holds`, `cast x if`/`when`, `.copied()`, `owned`, `shared` as an owner word, `copied`,
    `self`, `weak` alone, `set by`, `Some(...)` in an `if`/`while` line, type parameters in brackets after
    a function's, struct's or class's name, a plain `MutSpan` parameter, a plain `= ....as_mut_span()`,
    run 4's `if p = x:`, `Box[`, `RefCell`, `.upgrade(`, `.clone()`, `owned fn`, `ref mut this`)
    (`mech_newblocks.txt`): 20 hits, every one a refused word quoted in a message, an English "holds" or
    "copied", or `ref mut this` as an argument (R4). **No old spelling in NEW code.**
  - `strip_old.py` wrote `new_area-A.md` .. `new_area-E.md` (the area files without parts 2 and 5).
  - `scan_spans2.py` searches every inline code span outside the blocks, including spans that cross a
    line break, in the five stripped files and in `rules.md`, `vocabulary.md` and `changes.md`
    (`spans_hits2.txt`, 488 hits; `v50_hits.txt`, the 28 hits for verdict 50). Every hit was read in its
    sentence. Apart from section 3's failures, every hit names today's Ember, a refused or removed word in
    a message, the owner's own words quoted, Rust, or English.
- By hand: every owner's-test table of the 84 answers against its NEW code, every declared name in the
  NEW blocks, every message in parts 3 and 6, every row of `V3/rules.md` section 3, and the "stays
  refused" column of every vocabulary row.

## 2. Results at a glance

| test | result |
|---|---|
| no old spelling left in NEW code | **passes**: no NEW block writes `holds`, `cast x if`/`when`, `.copied()`, `owned`, `shared`, `copied`, `self`, `weak` alone, `set by`, `Some(...)` in a header, a plain writing `MutSpan`, or type parameters after a name (verdict 50 is met in every code block) |
| no old spelling left in the text | **1 kind left**: type parameters after the name in 2 messages, 1 refused program's description and 16 prose spans, almost all spans that cross a line break, which the lead's verdict 50 script did not see (F-01) |
| the owner's test on every declared name | **passes**: every declared name in the 110 NEW blocks has its word, or a stated default that never changes, aliases or copies silently; the tables match the code |
| the error-message rule | **9 failures** (F-07 to F-15): message twins in the vocabulary the phase 3 fixes did not reach (F-08 to F-11), copy ways out that do not say which line they replace (F-12), three rules rows (F-07, F-13, F-14) and one answer's message (F-15) |
| verdicts 1 to 52 (25 excepted) applied everywhere | **43 of 51 yes**; not everywhere: 24, 36, 39, 40, 42, 45, 46, 50 (section 5) |
| `V2/readability.md` F-01 to F-11 and F-13 | **all fixed** in the blocks they named (F-01 by verdict 42 instead); twins of F-05, F-06 and F-08 left in the vocabulary are this file's F-08 to F-10 (section 4) |
| agreed rows 3 to 9 | **all exactly as agreed**, in the verdicts' spellings (section 6) |

## 3. Failures

Most severe first. Each with the blocks it is in, the test it fails, and why.

### F-01. Type parameters still in brackets after the name in two messages, a refused program and 16 prose spans (CONFIRMED; verdict 50)

- **Blocks.**
  - Messages that give the old form as the line to write: AN-05 part 6, E2113 (`area-A.md` 1146-1151:
    "write `update`'s first line as `fn update[C](ref mut enemies: Array[Enemy], on_death:
    DeathHandler[C]) changes C:`"), while AN-05's own part 3 and AN-16's table write `[C]` above `fn
    update(...)`; AN-29 part 6, E2113 (`area-B.md` 1959-1963: "write `fn apply[C, F: fn() changes C ->
    int](owns f: F) changes C -> int:`").
  - A refused program described in the old form: AN-22 part 6 (`area-B.md` 1088-1089, "`fn put[T](k:
    u128, e: int, owns c: T):` with the same body"; the message after it writes `[T]` above).
  - Prose and rules text: `rules.md` 56 (section 0, change list), 262 (R5), 613 (R12, `fn inspect[C,
    F: ...]`); `changes.md` 66 (2.1), 704 (section 11, question 4's "Meanwhile"); `area-A.md` 787 (AN-04
    part 3, `pub fn retain[C, F: ...]`), 3530 (section 5, item 9); `area-B.md` 2477 (section 6, note 7);
    `area-D.md` 47 (terms, `fn each_name[C](...)`), 519 (AN-55, `fn inspect[...]`), 527 (AN-55, `pub fn
    spawn[R, CHANGED]`); `area-E.md` 164 (section 2, reading 2) and 1018 (AN-75 part 3), both `pub fn
    spawn[R, CHANGED](...)`; `vocabulary.md` 566 (section 6.2, the `[THR-10]` row's "becomes":
    `spawn[R, CHANGED](owns f: ...) changes CHANGED`).
  - PLAUSIBLE: `vocabulary.md` 152, the `some I` row's "stays refused" column, which says E2260's help
    "gives the line: ... `fn f[T: I](x: T)`". The lead's script kept it as today's help quoted; but the
    same column says what E2261's help becomes ("`owns dyn I`"), so the row reads as the design's
    message, and that message would give `[T: I]` above `fn f(x: T)`.
- **Test:** verdict 50 applied everywhere (also to the examples of verdicts 38 and 49, as the duty
  says: AN-05's line is verdict 49's example, AN-29's and the rules' are verdict 38's form).
- **Why:** every one is a generic function written `fn name[...](...)`. Most are inline spans that
  cross a line break, which `tools/v50_template_line.py` (it rewrites one line at a time) does not see.
  The two messages matter most: a programmer who follows them writes the form verdict 50 replaced.
  (Kept rightly: verdict 21's own sketch, quoted as the owner's, `changes.md` 73 and 706, `vocabulary.md`
  420; today's library lines, `vocabulary.md` 348 and 420; today's probe line, `vocabulary.md` 378.)

### F-02. Text that still says a `ref mut` name is never given another ref, or that no word says so (CONFIRMED; verdict 42)

- **Blocks:** AN-28 part 8 (`area-B.md` 1859-1860: "an understandable limitation kept (a `ref mut`
  name of a place is never re-pointed), with the one misleading form refused"), while AN-28's own
  "Meaning changed" line says verdict 42 now accepts `r = ref mut b`; AN-11 part 7 (`area-A.md`
  2447-2452: a `ref mut` view parameter "may point the caller's span elsewhere ... Across a package's
  edge no word or recorded fact says it"); area A section 5, found for the lead, item 8 (`area-A.md`
  3522-3527: "no fact says that a `ref mut` view parameter ... is never given another view").
- **Test:** verdict 42 applied everywhere.
- **Why:** verdict 42 lets every `ref mut` name be given another ref and gives the word for one that
  never is, `const ref mut` (`fn blend(const ref mut dst: MutSpan[float], src: Span[float]):`). AN-28's
  kind now states the opposite of its own NEW; AN-11 and area A's item 8 ask for a word the owner has
  given (area A's item 1, in the same section, says so).

### F-03. `V3/changes.md` still states the reading verdict 39 rejected (CONFIRMED)

- **Block:** `changes.md` 110-112 (2.1, "Foreign calls run the callbacks C keeps"): "The callbacks'
  changes go on the first lines of the functions that hand them to C, not on those of every function
  that makes a foreign call: this step reads 'a missing mark only refuses a loop' that way (question
  5)."
- **Test:** verdict 39 applied everywhere.
- **Why:** verdict 39 ("tick is a") puts the kept callbacks' changes on every function that makes the
  foreign call: `fn tick() changes Game, LOG:`. R12 and AN-78 apply it, and `changes.md` 10a and 11
  record it, but 2.1, the change list's own account of the rule, says the opposite with no correction.

### F-04. Text that still says a changing adapter cannot be made a `dyn Iterator` (CONFIRMED; verdict 46)

- **Blocks:** `changes.md` 80-81 (2.1: "Such an adapter is not made a `dyn Iterator` unless its list is
  empty (E2113). Listed for the owner (question 3)"); `changes.md` 232 (section 3's section-3 row: "E2113
  for ... a `dyn Iterator` of a changing adapter"); `vocabulary.md` 420, the iterator row's "stays
  refused": "an adapter that keeps a changing function made a `dyn Iterator` (E2113)", with no word that
  the `dyn` type may now say the changes; `vocabulary.md` 471-479, section 5, "Words still open for the
  owner", which lists verdict 46's and verdict 38's answers as still open.
- **Test:** verdict 46 applied everywhere.
- **Why:** verdict 46 lets a `dyn` type say what its values change (`owns dyn Iterator[Item = int]
  changes LOG`) and refuses only a `dyn` type that says less (R12, `rules.md` 619-623, and E2113's row
  860). The lines above still give the refusal of every changing adapter made `dyn`.

### F-05. The vocabulary's rewrite of a spec example sends on a channel and names no change (CONFIRMED; verdicts 36, 45)

- **Block:** `vocabulary.md` 510, section 6.1, the `owned fn` row's notes: "3495
  `thread.spawn(owned fn() => produce(tx))` becomes `thread.spawn(fn() with owns tx => produce(tx))`".
- **Test:** verdict 36 (a send is named on the first line: the verdict's own line for this very
  example is `fn() with owns tx changes Sender[int]: produce(tx)`), verdict 45 (a `=>` lambda says what
  it changes after its parameters), and verdict 30 (the caller of the `spawn` names it).
- **Why:** `produce` sends on `tx`, so the lambda changes `Sender[int]`; the "becomes" column is the
  line a spec rewrite will copy, and it writes the lambda as one that changes nothing.

### F-06. AN-58's summary row and prose still describe its list with `captured` (CONFIRMED; verdict 40)

- **Blocks:** `area-D.md` 92, the table "What v3 changed": "the local list is `Array[fn() changes
  captured]` (verdict 22)"; AN-58 part 3 (`area-D.md` 845-846): "`hs` holds functions that may each
  change the one variable they captured".
- **Test:** verdict 40 applied everywhere (a name visible where the line is written is written by its
  name).
- **Why:** AN-58's NEW, its v3 line and its owner's-test row write `hs: Array[fn() changes score] = []`
  (verdict 40); the table and the sentence say the list is the `captured` form the verdict replaced
  here, so a reader of the summary is told the opposite of the code.

### F-07. The rules' E3021 example for two spans gives a check that cannot make the call compile (CONFIRMED)

- **Block:** `rules.md` 833, section 3, E3021 row, second example: "`blend` changes `dst` while `src`
  shows the same list; prove them apart first: `assert_disjoint(dst, src)`, or give `src` a copy: `old
  = copy of xs`". Area A raised it (`area-A.md` 3502-3505, found for the lead, item 3); phase 3 did not
  change it.
- **Test:** the error-message rule (the program's own names; a line that compiles in that program).
- **Why:** `dst` and `src` are `blend`'s parameters, not the caller's names, and R4 requires the proof
  at the call, in the caller: inside `blend` the check always passes and the refused call stays
  refused. AN-14's own message gives the caller's line, `assert_disjoint(d, s)` before the call.

### F-08. A vocabulary message still uses a list the program never declares (CONFIRMED; twin of v2's F-05)

- **Block:** `vocabulary.md` 101, the `for` row's "stays refused": E3020 "`enemies.push(...)` adds to
  `enemies`, which this loop walks (line 3); collect the new enemies in `spawned` and write
  `enemies.extend(spawned)` after the loop".
- **Test:** the error-message rule (R18).
- **Why:** `spawned` is never declared and no line declares it; the rules' E3020 row and AN-04 were
  fixed in phase 3 (`spawned: Array[Enemy] = []` before the loop, the push in place of the line); this
  twin was not.

### F-09. Vocabulary messages that give a way out in words or a fragment, where the fixed answers give lines (CONFIRMED; twins of v2's F-06 kind)

- **Blocks and the words at fault:**
  - `vocabulary.md` 108, the `let` row, and 575, section 6.2's E2104 row: "set it in `init`, or remove
    `let` from its declaration" (AN-33's messages now give the field line, `items: Array[int]`);
  - `vocabulary.md` 146, the `from` row: E2118 for a parameter and a field of one name, "rename the
    parameter" (the rules' row and AN-22 now give both renamed first lines);
  - `vocabulary.md` 242, the `RefCell` row: "for a scene several objects own and change write `shares
    mut Scene`" (AN-49 now gives `s = Scene(entities=[])`, the field `scene: shares mut Scene` and
    `Panel(scene=shares mut s)`).
- **Test:** the error-message rule (the exact line to write).
- **Why:** each names an action or a fragment, not a line the programmer can type into that program;
  the phase 3 fixes of the same messages did not reach these vocabulary twins.

### F-10. A vocabulary message calls `integrate` with names no program declares (CONFIRMED; twin of v2's F-08)

- **Block:** `vocabulary.md` 245, the `MutSpan` row's "stays refused": E3028 "say so at the call:
  `integrate(ref mut ps, vs, 0.5)`".
- **Test:** the error-message rule (the program's own names).
- **Why:** `ps` and `vs` are verdict 3's example names; the rules' E3028 row was fixed in phase 3 to
  R4's and AN-11's own call, `integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`. (The same spelling
  in the `ref mut` and `MutSpan` rows' "it will mean" columns is verdict 3's own example, not a
  message, and is not counted.)

### F-11. The arena's messages in the vocabulary leave out the making line and the caller's lines (CONFIRMED; verdict 24, and the error-message rule)

- **Blocks:** `vocabulary.md` 238, the `Box` row: E1010 "for `Box[Mesh, LevelArena]` write `owns Mesh
  using LevelArena`", where AN-40's same message (`area-C.md` 826) adds "and on the line that makes the
  value name the arena: `lvl = Level(mesh=owns Mesh(verts=[0.0]) using current)`"; `vocabulary.md` 382,
  the `using` row: E3063 "make it in an arena the caller keeps: `fn load_level(ref mut arena:
  LevelArena) -> Enemy:`, with `using arena`", without the caller's `current = LevelArena()` and `boss =
  load_level(ref mut current)`, which the rules' row 846 now gives (area C's note 6, fixed in the rules
  only).
- **Test:** verdict 24 (the line that makes the value says which arena) and R18 (a message that
  replaces several lines names every one).
- **Why:** following the `Box` row's line alone leaves the making line without `using`, which R3
  refuses (E2020); following the `using` row's leaves the caller without an arena to give.

### F-12. Copy ways out in E3060 messages that do not say which line they replace (PLAUSIBLE)

- **Blocks:** AN-26 part 6 (`area-B.md` 1631-1633: "or keep a copy of your own: `n = String(first(names))`");
  AN-35 part 6 (`area-C.md` 294-295: "or keep your own copy: `n = copy of p.name`"); AN-46 part 6
  (`area-C.md` 1412-1413) and its rules twin, `rules.md` 840, E3060's third example ("or keep your own
  strings: `ks = sorted(m.keys().copy_each())`"); AN-49 part 6 (`area-C.md` 1748-1750: "or keep a copy:
  `a = copy of s.entities`"); AN-50 part 6 (`area-C.md` 1824-1825: "or keep your own copy: `first = copy
  of o.bag.items[0]`").
- **Test:** the error-message rule (R18: where the line replaces another, the message names it; the
  line compiles in that program).
- **Why:** in each program the name was declared as a view on an earlier line (`n = p.name`, `ks =
  sorted(m.keys())`, `a = s.entities`, `first = ref o.bag.items[0]`); written below it, the copy line puts
  a value of its own into a view name, which does not compile. Area A found this flaw in AN-01's
  E3060 (`area-A.md` 3540-3541), and phase 3 fixed that one only ("write `n = String(boss.name.trim())`
  in place of line 8"); AN-29's E3060 shows the form that passes ("write line 1 as `head =
  String(names[0])`"). PLAUSIBLE because the messages name the earlier line's number in their first half,
  so a reader may take the copy line as its replacement.

### F-13. The rules' E7015 message gives fragments and leaves out lines it replaces (CONFIRMED)

- **Block:** `rules.md` 873, section 3, E7015 row: "create the game shared between threads, `game =
  thread_shared Game()`, take it as `game: thread_shared Game` in `setup`, write `with thread_shared
  game`, and, since `add_point` changes `score`, declare `score: Atomic[i32]` and write
  `this.score.fetch_add(1)` in `add_point`".
- **Test:** the error-message rule (R18: the exact lines, every line it replaces).
- **Why:** "take it as ... in `setup`" and "write `with thread_shared game`" are fragments of lines;
  `setup`'s whole first line (with `changes Game`, verdict 30), the call in `main` (now `setup(game)`, no
  longer `setup(shares mut game)`) and the line that reads the score (`game.score.load()`) are not
  given. AN-78's own message (`area-E.md` 1888-1897) gives them all and says it is "the rules' E7015 row,
  completed with every line it replaces, R18".

### F-14. E7014 tells the programmer to guard a field with a lock the class does not have, and gives no line (CONFIRMED; v2's F-05 kind)

- **Blocks:** `rules.md` 871, section 3, E7014 row; AN-71 part 6 (`area-E.md` 432-434); the vocabulary's
  `thread_shared` row (`vocabulary.md` 394): "keep the list in a field `guarded by` a lock and change it
  inside `with names.lock:`".
- **Test:** the error-message rule (the exact line to write; R18: a line that compiles in that
  program).
- **Why:** AN-71's `Names` has no lock (`items: Array[String] = []`), so `with names.lock:` does not
  compile until the class gains one, and no line gives it (`lock: Lock = Lock()` and `items:
  Array[String] guarded by lock = []` in `Names`, then `with names.lock: names.items.push(s)` in place of
  the refused line). Phase 3 fixed the same kind of message for `FRAME_LOCK` and `HITS_LOCK` (E7012) and
  for `Player` (E7010).

### F-15. AN-24's E3021 message changes the field's type but names only one of the two lines that make a view (PLAUSIBLE)

- **Block:** AN-24 part 6 (`area-B.md` 1366-1371): "write `reg: sharedref mut Registry` in `View` and
  `View(reg=sharedref mut reg, entity=1)`".
- **Test:** the error-message rule (R18: where it replaces several lines, it names every one).
- **Why:** AN-24's NEW makes two views, `View(reg=ref reg, entity=0)` and `View(reg=ref reg, entity=1)`;
  once the field is `sharedref mut Registry`, the `entity=0` line no longer fits the field and the
  message does not name it. PLAUSIBLE because the refused snippet the message quotes shows only the
  `entity=1` line.

## 4. `V2/readability.md`'s failures F-01 to F-11 and F-13

| v2 | fixed? | where checked |
|---|---|---|
| F-01 (`boss = ref mut enemies[0]` on an existing `ref mut` name) | **yes**, by verdict 42 instead of a rewrite: every `ref mut` name may be given another ref, so the lines compile (R2) | AN-01 E3060 (`area-A.md` 302-304), AN-02's three messages (489-502), AN-16's row (3375), AN-61 E3060 (`area-D.md` 1317-1320), the rules' E3060 row (840) |
| F-02 (AN-23's E2140 way out loses the change) | **yes**: both ways out of verdict 13, `xs[0] += 1` and `first(ref mut xs) += 1` with `-> ref mut int` | `area-B.md` 1210-1215 (its v3 line, 1110-1112, still says the message is v2's: a stale sentence, section 7) |
| F-03 (E3026's `Counter` and `count.n` not in the program) | **yes**: every line given, `struct Counter:` with `n: int = 0` to `fn click() changes handlers, Counter:` | `rules.md` 864, AN-53 (`area-D.md` 280-282), AN-57 (`area-D.md` 738-751) |
| F-04 (E3063's `target: shares Player` for watchers that change) | **yes**: `target: shares mut Player`, `players: Array[shares mut Player]`, `Watcher(target=shares mut players[r])` | `rules.md` 845, AN-52 (`area-C.md` 2004-2010) |
| F-05 (lists and locks the program does not declare) | **yes** in every block it named: `spawned`, `dead`, `joined`, `new_items`, `TICK_LOCK`, `HITS_LOCK` declared | `rules.md` 836, 877; AN-04, AN-05, AN-17, AN-48, AN-76, AN-84. Twins not named by v2 remain: this file's F-08 (vocabulary `for` row) and F-14 (E7014) |
| F-06 (ways out in words) | **yes** in every block it named | AN-07, AN-11, AN-13, AN-16, AN-24, AN-25, AN-31, AN-42, AN-53, AN-58, AN-77, the rules' E3023 row (813). Vocabulary twins remain: this file's F-09 |
| F-07 (AN-36's L3001 tells the owning list to go weak) | **yes**: names the line that would close the cycle and gives `parent: weakref Node = None` for a back link | `area-C.md` 402-408 |
| F-08 (AN-11's `integrate(ref mut ps, vs, 0.5)`) | **yes**: AN-11's messages give its own call, `integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`; the rules' E3028 row too | `area-A.md` 2402-2441, 3412; `rules.md` 830. The vocabulary's `MutSpan` row keeps the old call: this file's F-10 |
| F-09 (asking the caller to rewrite another package) | **yes**: lines for the calling package (`died` collected, `note` called after the loop) | `area-B.md` 306-311 |
| F-10 (a view struct named `Handle`) | **yes**: `Editor`, `eds` | `area-B.md` 1354-1360 |
| F-11 (E7007's `with this.lock:` in a free function) | **yes**: an `Atomic` field, or a guarded field with an `add_total` method, called on line 19 | `rules.md` 874, AN-74 (`area-E.md` 905) |
| F-13 (`from(self)` in the vocabulary) | **yes**: `from(this)`, today's `from(self)` named as today's | `vocabulary.md` 146 |

## 5. The owner's verdicts 1 to 52 (25 excepted)

| verdict | applied everywhere? | where not |
|---|---|---|
| 1 `holds` (replaced by 26) | yes: no `holds` opening is left; every English "holds" in prose and messages is English | — |
| 2 `ref mut` keeps reads out | yes | — |
| 3 `ref mut` at a call, a `MutSpan` included; no word for `Mutex` and friends | yes | — |
| 4 `shares mut` | yes | — |
| 5 a read-only name's promise | yes | — |
| 6 `changes this` after the brackets | yes | — |
| 7 `static fn` | yes | — |
| 8 `static mut` | yes | — |
| 9 `weakref`; no `Some(...)` in a header | yes | — |
| 10 another link is `weakref x.keep`; `copy of` a link refused | yes | — |
| 11 every `weakref` counted | yes | — |
| 12 `copy of` | yes | — |
| 13 a lost change refused, both ways out | yes (AN-10, AN-23, the rules' E2140 row) | — |
| 14 `set by` removed | yes (left only where run 3's measurements are named) | — |
| 15 a walk stops at the first deleted parent | yes | — |
| 16 one rule for every loop | yes (AN-77) | — |
| 17 functions and lists of functions say what they change | yes | — |
| 18 `changes this` promises nothing about other names | yes | — |
| 19 `owns`, `shares`, `using` | yes | — |
| 20 `thread_shared`, `sharedref mut` kept | yes | — |
| 21 library functions given a changing function | yes (the brackets in its prose spans are F-01, verdict 50's) | — |
| 22 a lambda names the captured variables it changes; `captured`, `captured...` | yes | — |
| 23 the first owner keeps what it could do | yes | — |
| 24 the making line names the arena | **no** | F-11: the vocabulary's `Box` row (238) gives only the field for `Box[Mesh, LevelArena]`, and its `using` row (382) leaves out the caller's lines |
| 26 `exists as` | yes | — |
| 27 `if s is Circle as c:`; `cast s as Circle` | yes | — |
| 28 the deleting walk holds a `weakref` | yes (AN-62 (b) word for word; R11) | — |
| 29 a change under a lock is named | yes | — |
| 30 starting a thread counts | yes | — |
| 31 a C call may run kept callbacks; `@ffi(no_callbacks)` | yes | — |
| 32 the word on the name sets the rights | yes | — |
| 33 `.copy_each()` | yes | — |
| 34 a writing `MutSpan` name says `ref mut` | yes | — |
| 35 a function that runs stored functions repeats their type's word | yes (AN-53's `click`, AN-54's `run`) | — |
| 36 a send is named | **no** | F-05: `vocabulary.md` 510, `thread.spawn(fn() with owns tx => produce(tx))` |
| 37 a receive through a given end is named | yes (rules and vocabulary; no answer uses a channel) | — |
| 38 library functions take the function as a template parameter | yes | — |
| 39 a function that calls C names what kept callbacks change | **no** | F-03: `changes.md` 110-112 |
| 40 `captured` covers only what a line cannot see | **no** | F-06: `area-D.md` 92 and 845-846 (AN-58) |
| 41 named change lists, strict | yes (R12, the `changelist` row) | — |
| 42 every `ref mut` name may be given another ref; `const ref mut` | **no** | F-02: `area-B.md` 1859 (AN-28 part 8); `area-A.md` 2447-2452 (AN-11 part 7) and 3522-3527 (section 5, item 8) |
| 43 a call that may take a held lock is refused (E7018) | yes | — |
| 44 `guarded by` follows the `Mutex`'s rules | yes | — |
| 45 a `=>` lambda says what it changes | **no** | F-05: `vocabulary.md` 510 |
| 46 a `dyn` type may say what it changes | **no** | F-04: `changes.md` 80-81 and 232; `vocabulary.md` 420 ("stays refused") and 471-479 (section 5) |
| 47 an arena-made value carries its arena's address; lists too | yes | — |
| 48 a repeated function type is named with `type` | yes (AN-76's `Handler`) | — |
| 49 a long function type named with `type`, change list as its parameter | yes (AN-05's `DeathHandler[C]`; its message's bracket form is F-01) | — |
| 50 type parameters on a line of their own | **no** (every code block yes) | F-01: two messages, one refused program, 16 prose spans |
| 51 a long lambda inline or as a named local function | yes (the vocabulary's `fn` row) | — |
| 52 a `spawn` under the loop's lock allowed; the join inside refused | yes (R12, R15, E7018, AN-76) | — |

## 6. The agreed rows 3 to 9

| row | where | as agreed? |
|---|---|---|
| 3 | AN-17: `fn note(msg: String) changes log:`, checked against its body (E2105) | **yes**, without `self`, as the owner allowed |
| 4 | AN-30: `return ref g[0]` inside `with g = scores.lock():`, refused with "this points into the locked list, which can change once the lock ends; return a copy, or do the work inside the `with` block" | **yes**, word for word |
| 5 | AN-08: `word = line.to_lower().trim()` then `word.len()`; AN-09: returning it refused, the message contains "return `String(...)` to hand back your own copy" | **yes** |
| 6 | AN-53: `b.on_click(fn() changes score: score += 1)` changes `game`'s real `score`; a copy only when named, `fn() with copy of score:` | **yes**, in verdict 22's spelling |
| 7 | AN-31: `fn load_assets() reads renderer -> Array[String]:`, checked against its body and at the call | **yes**, without `self` |
| 8 | AN-37, AN-38: `State.Walking(ref mut path):`; plain `path` and `mut path` both get "to change that list through `path`, write `State.Walking(ref mut path):`" | **yes**, word for word |
| 9 | AN-34: `snapshot = copy of items` copies, `snapshot = items` hands over, E3040 names both ways out (`copy of`, `shares`); AN-44 | **yes**, in the new spelling |

## 7. Notes for the lead (meet the tests' letter; not counted as failures)

1. **Stale "v3" lines that say a message is still v2's**: AN-23 (`area-B.md` 1110-1112: "The E2140
   message in part 6 is left as v2 wrote it"), AN-57 (`area-D.md` 701-702: "The E3026 message is v2's
   ... the lead's") and AN-61 (`area-D.md` 1226-1227). Phase 3 fixed or settled all three; the lines
   misdescribe their own answers.
2. **E3020 and E7018 both fit one call.** `note()` inside `for line in log.lines:` inside `with
   log.lock:` meets E3020's row (837) and E7018's row (869), with different texts; R18 asks for one
   message per mistake, and the rules do not say which comes first (AN-75 gives E3020 for the loop and
   E7018 without it).
3. **R5's example of a caller** (`rules.md` 284-285: "`main` calls `tick()`, which changes `FRAME`, so
   `fn main() changes FRAME:`") sits beside R5's own `fn tick() changes FRAME, LOG:` (264), with which
   `main` would say `changes FRAME, LOG`, as AN-76's does.
4. **`owns` before a value on a making line** (area C's note 4, still open): verdict 24 writes `mesh=owns
   Mesh(points=[0.0, 1.0]) using current`, while R3's first point refuses `owns x` before a value its
   place already hands over (E0109) and AN-40 writes `Entity(mesh=Mesh(verts=[0.0]))` with no word. R3
   could say that `owns` with `using` on a making line is allowed.
5. **AN-80's `@sync` message** (`area-E.md` 2180) leads from the OLD class to one that compiles and
   keeps the OLD's race on `longest` (two separate atomic steps). The compiler cannot see that race, so
   the message says only what it knows (R18); the answer's NEW, with guarded fields, is the fix.
6. **`changes.md` section 6** (error codes) does not list E7018, new by verdicts 43 and 52; only
   section 10a and the rules' section 4 do.
7. **Area E's section 4.2, item 3** (`area-E.md` 2601-2603) keeps the pre-verdict note ("this file
   applies the verdict as written ('start the thread after the loop' moves one line)") under a heading
   that says verdict 52 answered it the other way; the answer's text is right.
8. **The rules' E3021 example and area A's notes 1, 2 and 4** (`check_disjoint`'s `Ok` branch as a
   proof; making a view while a `ref mut` name of the same elements is in use; two keys of one map) are
   still open in `V3/rules.md`; F-07 is the one that breaks the message rule.

## 8. For the owner

Nothing new. Every question this check met is answered by verdicts 21 to 52, or is the lead's (the
failures and notes above).

## 9. Files

- `/home/user/ember/tasks/96/redesign/run4/v3/readability.md` (this file).
- Under `/home/user/ember/target/redesign4/v3-07-readability/`: `extract.py`, `blocks.txt`,
  `new_blocks.txt` (the block extraction); `scan.py`, `mech_newblocks.txt` (old spellings in NEW
  blocks, 20 hits, none a failure); `strip_old.py`, `new_area-A.md` .. `new_area-E.md` (the area files
  without parts 2 and 5, original line numbers kept); `scan_spans.py`, `scan_spans2.py`,
  `spans_hits.txt`, `spans_hits2.txt` (inline spans, one line and across a line break), `v50_hits.txt`
  (the verdict 50 hits).
