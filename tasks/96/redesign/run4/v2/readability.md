# Readability check of run 4, step 1, v2 (agent 07)

Agent 07 (readability checker), run 4, step 1 (v2). This file checks every NEW block of the five v2 area
files against the owner's test, the error-message rule, the owner's decisions and the agreed rows. It
fixes nothing. `RUN4` means `tasks/96/redesign/run4/`, `V2` means `tasks/96/redesign/run4/v2/`; "R1" ..
"R18" are the rules of `V2/rules.md`; "row N" is the owner's agreed row N and "decision N" the owner's
decision N in `inputs/JOB.md` ("Owner verdicts on run 4's open questions"). Line numbers are lines of the
file named.

Power: `PowerLineStatus` printed `Online` at the start of this work and again before writing this file.
No compiler change, no git command, no Ember program run, no `ember check` (this file makes no claim
about what today's Ember accepts).

## 0. Terms

- **NEW block**: code an area file proposes under the v2 rules: part 3 ("NEW") of each answer, and the
  refused programs written out in part 6 ("Still refused"). OLD code (part 2) and "means exactly" code
  (part 5) are today's Ember, carried unchanged, and are not judged here.
- **message**: the text of a refusal or a warning, in an area file or in `V2/rules.md` section 3.
- **the owner's test**: for every name a line declares, the word on the line that says what it is (the
  owner's own thing, a copy, or handed over) and the word that says whether it may change it; a name
  with no word passes only where the stated default cannot surprise a reader (never changes anything,
  never aliases, never copies silently).
- **the error-message rule**: a message names what was refused in the program's own names, says why in
  one line, and gives the exact line to write, a line that compiles in that program under the rules
  (brief; R18).
- **old spelling**: a word the decisions removed or renamed: `owned`, `shared` as an owner word,
  `copied`, `self`, `weak` alone, `set by`, `Some(...)` in an `if` or `while` header.
- **failure**: a place that does not meet one of these tests. **Concern**: something that meets the
  tests' letter but that the owner may judge differently; listed apart, not counted as a failure.
- **CONFIRMED**: checked against the rule text, no reading needed. **PLAUSIBLE**: depends on a reading of
  the rules that the rules do not settle in words.

## 1. What was checked, and how

- Read in full: `inputs/brief.md`, `inputs/JOB.md`, `inputs/duties.md` (section 07), `V2/rules.md`,
  `V2/vocabulary.md`, and the NEW parts (1, 3, 4, 6, 7, 8 and every section outside the answers) of
  `V2/area-A.md` .. `V2/area-E.md`.
- `target/redesign4/v2-07-readability/extract.py` pulls every `ember` block out of the five area files
  and sorts it OLD or NEW by the part it sits in (output `blocks.txt`): **218 blocks, 96 NEW** (area A
  21, B 18, C 21, D 20, E 16). On the NEW blocks it searches for every old spelling, run 4's opening
  `if p = x:`, `ref mut this` on a first line, `mut` alone, `sref`, `Box[`, `RefCell`, `Weak[`,
  `.upgrade(`, `.clone()`, `owned fn`, `Array[T, A]`, the comma rule (R3), and every call to a function
  whose parameter is `ref mut` or `MutSpan` with no `ref mut` at the call, or the other way round
  (output `mech_checks.txt`). **Result: 1 hit**, `.copied()` in AN-46 (F-12). A second pass over the
  inline code outside the blocks (`prose_old_spellings.txt`, 421 hits in the eight v2 files) was read
  hit by hit: every hit in the area files names today's code, a run-4 spelling being replaced, or a
  refused program, except the two in F-12 and F-13.
- `strip_old.py` (same folder) wrote each area file without its OLD and "means exactly" parts
  (`new_area-A.md` .. `new_area-E.md`, original line numbers kept); those were read by hand: every
  owner's-test table, every declared name in the NEW blocks, and every message.
- Messages checked: every message in part 6 of the 84 answers, the messages inside parts 3, and every
  row of `V2/rules.md` section 3.

## 2. Results at a glance

| test | result |
|---|---|
| no old spelling left | **2 left** in material that is not today's code: `.copied()` in AN-46 and its vocabulary row (F-12), and `from(self)` in the vocabulary's "it will mean" column of `from` (F-13). Every other occurrence is a description of today's code or of a refused form |
| `holds` for opening (decision 1) | **passes**: AN-60, AN-62, AN-63, AN-65, AN-67, AN-68 open with `holds`; no NEW block writes `if p = x:` or `if Some(...)` |
| `ref mut` at calls, `MutSpan` included; no word for `Mutex` and friends (verdict R18-16, decision 3) | **passes**: no NEW call misses `ref mut` or adds it to a plain parameter (script, then by hand); `top(m)` (AN-30), `place(ref mut a.as_mut_span(), ...)` (AN-84) |
| no `self`; `this`; `changes this` after the brackets (decisions 6, 18) | **passes** in every NEW block; one `self` left in the vocabulary (F-13) |
| `weakref`, `copy of`, `owns`, `shares`, `shares mut`, `using`, `static fn`, `static mut`, `set by` gone (decisions 4, 7-12, 14, 19, 20) | **passes**, apart from `.copied()` (F-12) |
| functions say what they change (decision 17) | **passes** by the rules' letter (AN-53, AN-57, AN-60, AN-62, AN-76, AN-78); one concern for the owner on what "outside itself" means (C-1) |
| a lost change refused with both ways out (decision 13) | AN-10 **passes**; AN-23's E2140 message gives a way out that loses the change again (F-02) |
| agreed rows 3 to 9 | **all exactly as agreed**, in the new spellings (section 4) |
| the owner's test on every declared name | one failure: a plain local holding a `MutSpan` (F-14); every other name has its word or a stated default that never changes, aliases or copies |
| the error-message rule | **11 failures** (F-01 to F-11): lines that the rules refuse, names the program does not have, ways out given in words, a way out that does not do what was meant |

## 3. Failures

Most severe first. Each with the blocks it is in, the test it fails, and why.

### F-01. Messages tell the programmer to name an element again with a `ref mut` name, which R2 refuses (CONFIRMED)

- **Blocks:** `V2/rules.md` section 3, E3060 row (line 584: "name the enemy again after it:
  `boss = ref mut enemies[0]`"); AN-01 part 6, E3060 (`area-A.md` 292-294); AN-02 part 6, the three
  messages for `on_hit`, `kill_weak` and `heal_all` (`area-A.md` 474-486); AN-16's table row for that
  line (`area-A.md`, AN-16); AN-61 part 6, E3060 (`area-D.md` 1109-1111: "name it again after the call:
  `h = ref mut this.handlers.get_mut(i).unwrap()`").
- **Test:** the error-message rule (the line a message gives compiles in that program under the rules,
  R18).
- **Why:** in each program the name (`boss`, `h`) was declared `= ref mut ...`. R2: "A `ref mut` name
  stands for the place it was made from: `r = ref mut a` then `r = 20` writes `a`, and `r = ref mut b`
  is refused (E3023)". So `boss = ref mut enemies[0]` on the existing `boss` is refused by the very rules
  the message speaks for (AN-28's own message says "to refer to `b` use a new name"). The line that
  compiles names a new name (`boss2 = ref mut enemies[0]`) or puts the first `boss` in a block of its own.
  (The same message for a plain view, AN-13's `p = ps.get(0).unwrap()` and AN-21's
  `s = m.sorted_verts()`, is fine: R2 lets `=` give a plain view name a new target.)

### F-02. AN-23's E2140 message leads to a program that compiles and still loses the change (CONFIRMED)

- **Block:** AN-23 part 6 (`area-B.md` 1127-1130): `first_value(xs) += 1`, message "keep it in a name
  first: `n = first_value(xs)`, then `n += 1`".
- **Test:** decision 13 ("the message gives both ways out": change the element itself, or make the
  function give back the element itself) and JOB's "a rule that makes code compile but do something the
  programmer did not mean is wrong".
- **Why:** AN-23's intent is "add 1 to the element a function hands back". `n = first_value(xs)` then
  `n += 1` changes a local that nothing reads, so the element is never changed: the same lost change,
  now accepted. The two ways out decision 13 names (here `xs[0] += 1`, or `first(ref mut xs) += 1` with
  `-> ref mut int`, which AN-23's own NEW writes) are not in the message. AN-10's E2140 message has
  both and is the model.

### F-03. E3026's advice names a type and a field the program does not have (CONFIRMED; run 4's F-06, still open)

- **Blocks:** `V2/rules.md` section 3, E3026 row (line 605); AN-53 part 6 (`area-D.md` 222-226); AN-57
  part 3's message (`area-D.md` 576-587).
- **Test:** the error-message rule (the program's own names; a line that compiles in that program).
- **Why:** the refused program (`fn register(ref mut b: Button, ref mut total: int):
  b.on_click(fn(): total += 1)`) has `total: int` and no `Counter`. The help gives
  `count = shares mut Counter()` and `count.n += 1`, which need a class `Counter` with a field `n` that
  the message does not give, and a `Button` whose list is `Array[fn() changes Counter]`, which AN-57's
  message does name but AN-53's and the rules' row only describe ("kept in a list of
  `fn() changes Counter`").

### F-04. The rules' E3063 example gives a line its own program refuses (CONFIRMED)

- **Block:** `V2/rules.md` section 3, E3063 row for a kept view (line 589): "let the watchers own a
  share: `target: shares Player`".
- **Test:** the error-message rule.
- **Why:** the row cites AN-52, whose watchers change their players. Under decision 4 a `shares` owner
  only reads (R9), so that line would be refused (E3023). AN-52 itself writes `target: shares mut
  Player`; area C's note 1 says the same.

### F-05. Messages use a list or a lock the program does not declare, and give no line for it (CONFIRMED; run 4's group)

- **Blocks:** `V2/rules.md` section 3, E3020 (line 581, `spawned`) and E7012 (line 617, `FRAME_LOCK`);
  AN-04 E3020 (`area-A.md` 884-886, `spawned`); AN-05 both E3020 messages (`area-A.md` 1059-1067,
  `dead`); AN-17 E3020 (`area-B.md` 266-269, `joined`); AN-48 E3020 (`area-C.md` 1538-1540,
  `new_items`); AN-76 E7012 (`area-E.md` 1194-1198, `FRAME_LOCK`); AN-84 E7012 (`area-E.md` 2117,
  `HITS_LOCK`).
- **Test:** the error-message rule (R18: where the line replaces or needs several, the message names
  every one; the line compiles in that program).
- **Why:** `spawned.push(...)`, `dead.push(i)`, `this.players.extend(joined)`,
  `cam.target.inventory.extend(new_items)`, `guarded by FRAME_LOCK` do not compile until the list or the
  lock is declared, and no message gives that line (`spawned: Array[Enemy] = []`, `dead: Array[int] =
  []`, `static FRAME_LOCK: Lock = Lock()`). AN-72's message shows the form that passes ("write
  `parts: Array[int] = []` before the `with` block, ...").

### F-06. Messages that give a way out in words, not the exact line (CONFIRMED; partly run 4's F-07)

- **Blocks and the words at fault:**
  - AN-07 E3063 (`area-A.md` 1631-1635): "make the result `Array[String]` and push `String(w)` for each
    word" (its second way out is an exact line);
  - AN-11 E3063 (`area-A.md` 2289-2291): "put a view of what the caller gave there, or return a list of
    your own" (the rules' own row, line 588, gives the lines: `best = other`, `-> Array[int]`);
  - AN-13 E1010 for `ps[0].load()` (`area-A.md` 2624): "write `copy of ps[0]`", with no name of what was
    refused and no why;
  - AN-16 (`area-A.md` 3146): `collect`'s message "write `words: Array[StrView] = ...`";
  - AN-24 E3063 (`area-B.md` 1288-1290): "or keep `reg` alive as long as `vs`";
  - AN-25 E3063 (`area-B.md` 1420-1422): "change `words` to `Array[String]`" (the field is declared
    `words: Array[StrView] from text`, so the line to write is not given);
  - AN-31 E2100 (`area-B.md` 2078-2080): "set `this.renderer` before this line" (the line is
    `this.renderer = Renderer(width=width)`);
  - AN-42 E3020 (`area-C.md` 994): "collect what to add in a list, then call `reg.emplace(...)` for each
    once the loop ends";
  - AN-53 and AN-58 E3020 (`area-D.md` 231-233, 681-684): "or collect what to add and push it after the
    loop";
  - AN-77 E2221 (`area-E.md` 1404-1406): "or its position"; and "walk it by an index or a `Handle`"
    (`area-E.md` 1416-1417);
  - `V2/rules.md` section 3, E3023 row (line 563): "or make `game` with `shares mut`".
- **Test:** the error-message rule (the exact line to write).
- **Why:** each names an action, not a line the programmer can type into that program.

### F-07. AN-36's cycle warning tells the programmer to make the owning list weak (CONFIRMED)

- **Block:** AN-36 part 6, L3001 (`area-C.md` 382-385): "`Node.children` can lead back to a node that
  holds it; if it does, link it with `weakref Node` instead".
- **Test:** the error-message rule, and AN-64's own fix ("L3001 tells the programmer to make the owning
  edge weak").
- **Why:** `children: Array[shares mut Node]` is the only link in that `Node`; "link it with `weakref
  Node`" reads as making the children list weak, which would let every child die: the defect AN-64
  removed (its message names the back link and gives the exact line, `best: weakref Person = None`). No
  exact line is given either.

### F-08. AN-11's E3023 message calls `integrate` with names the program does not have (CONFIRMED)

- **Block:** AN-11 part 6 (`area-A.md` 2279-2282) and AN-16's table row for it (`area-A.md` 3199):
  "to point the caller's view elsewhere, write `ref mut pos: MutSpan[float]`, called as
  `integrate(ref mut ps, vs, 0.5)`".
- **Test:** the error-message rule (the program's own names).
- **Why:** AN-11's caller has `p` and `v`, and passes `p.as_mut_span()`, a span made in the call; `ps`
  and `vs` are the rules' example names (R4), not this program's. A `ref mut pos: MutSpan[float]`
  parameter re-points the caller's own view, so the caller also needs a line that keeps the span in a
  name (`ps = p.as_mut_span()`), which the message does not give.

### F-09. A message asks the caller to rewrite another package (CONFIRMED; run 4's F-08, still open)

- **Block:** AN-17 part 6 (`area-B.md` 270-272): from another package, E3021 "if it does not, say what it
  reads: `fn note(msg: String) reads frame changes log:`".
- **Test:** the error-message rule (a line the refused programmer can write in that program).
- **Why:** `note` is in another package; the programmer whose loop is refused cannot change its first
  line. The message gives no line for the calling package (for example, call `note` after the loop, or
  keep the line in a list and log it after).

### F-10. A refused example names its struct `Handle` (CONFIRMED; run 4's F-09, still open)

- **Block:** AN-24 part 6 (`area-B.md` 1271-1276): `struct Handle:` with `reg: ref mut Registry`, and the
  message "write `reg: sharedref mut Registry` in `Handle`".
- **Test:** keywords and names mean the same everywhere (the vocabulary's `Handle` row: "handle" is used
  only for a token that names something elsewhere, `Handle[T]`, `JoinHandle`, `@ffi(handle)`).
- **Why:** `Handle` here is a view struct, the meaning the vocabulary removed; a reader of the message
  meets "handle" in the sense AN-50 retired.

### F-11. E7007's way out uses `this` where the change it names has none (PLAUSIBLE)

- **Blocks:** AN-74 part 6 (`area-E.md` 792-794) and `V2/rules.md` section 3, E7007 row (line 614):
  "two tasks change `stats.total` (line 19); guard it: `total: int guarded by lock`, and change it
  inside `with this.lock:`".
- **Test:** the error-message rule (the line compiles where it goes; the message names the lines it
  replaces).
- **Why:** line 19 of AN-74's program is the task in `count`, a free function, where the object is
  `stats` and `this` does not exist (E1010). If the change is meant to be inside a method, the message
  does not say which line `with this.lock:` goes around; if it is the task's own line, the line to write
  is `with stats.lock:`.

### F-12. `copied` left as `.copied()`, doing the job decision 12 gave `copy of` (CONFIRMED that it is there; whether it is allowed is the owner's)

- **Blocks:** AN-46 part 3 (`area-C.md` 1287: `names = sorted(m.keys().copied())`), its owner's-test row
  (`area-C.md` 1299: the word that says `names` is its own is `.copied()`), and its E3060 help
  (`area-C.md` 1321: "keep your own strings: `ks = sorted(m.keys().copied())`"); the vocabulary's
  `.copied()` row (`vocabulary.md` 295, "keep").
- **Test:** decision 12 applied everywhere with no old spelling left ("`copied` is renamed `copy of`").
- **Why:** the vocabulary keeps `.copied()` as today's iterator adapter, not the keyword, and area C
  flagged it for this check (its note 4). But a reader sees `copied` making "a copy of your own" on the
  line, the meaning decision 12 moved to `copy of`, and the owner's test for `names` rests on it. Listed
  for the owner (section 7).

### F-13. `self` left in the vocabulary's meaning of `from` (CONFIRMED)

- **Block:** `vocabulary.md` line 160, the `from` row, column "it will mean": "every use reads 'comes
  from': `from m import x`, C's `from(self)`, `From[E]`; ...".
- **Test:** the verdict "no `self`" applied everywhere.
- **Why:** this column says what the word will mean, and still writes the C contract's lifetime as
  `from(self)` (spec 4669). With `self` gone it would be `from(this)` or another spelling; the row does
  not say which.

### F-14. A plain local that holds a `MutSpan` writes elements with no word on its name, and the rules disagree on what it is (PLAUSIBLE)

- **Blocks:** AN-14 part 3 (`area-A.md`, `mix` and `mix_any`: `d = tracks[dst_name].as_mut_span()`,
  then `blend(ref mut d, s)`), its owner's-test row (`area-A.md` 2733); AN-16's table row for
  `rest = other` (`area-A.md` 3200: "may it change it: none (default: by its type, as `other`)").
- **Test:** the owner's test (a name with no word only where the stated default cannot surprise).
- **Why:** R1 says that where `a` is a view, "`b = a` makes `b` a read-only view of what `a` views", and
  R2 makes a name bound to a call that gives a view a read-only view; R5's exception lets a plain name
  write what a `MutSpan` leads to. For `d` and `rest` the two rules give opposite answers, and the
  tables' "by its type" does not say which wins. A reader of `d = tracks[dst_name].as_mut_span()` sees
  no word on `d` that says it may change `tracks`' elements, while `boss = enemies[0]` needs `ref mut`
  to change one element. Area A's note 2 found the same gap from the safety side.

## 4. The agreed rows 3 to 9

| row | where | as agreed? |
|---|---|---|
| 3 | AN-17: `fn note(msg: String) changes log:`, checked against its body (E2105 messages) | **yes**, without `self`, as the owner allowed |
| 4 | AN-30: `return ref g[0]` inside `with g = scores.lock():` refused with "this points into the locked list, which can change once the lock ends; return a copy, or do the work inside the `with` block" | **yes**, word for word |
| 5 | AN-08: `word = line.to_lower().trim()` then `word.len()`; AN-09: returning it refused, message contains "return `String(...)` to hand back your own copy" | **yes** |
| 6 | AN-53: `b.on_click(fn(): score += 1)` changes `game`'s real `score`; `fn() with copy of score:` for a copy | **yes**, in the new spelling |
| 7 | AN-31: `fn load_assets() reads renderer -> Array[String]:`, checked against its body and at the call | **yes**, without `self` |
| 8 | AN-37, AN-38: `State.Walking(ref mut path):`; plain `path` and `mut path` both get "to change that list through `path`, write `State.Walking(ref mut path):`" | **yes**, word for word |
| 9 | AN-34: `snapshot = copy of items` copies, `snapshot = items` hands over, E3040 names both ways out (`copy of`, `shares`); AN-44 | **yes**, in the new spelling |

## 5. The owner's decisions, one by one

| decision | applied everywhere? |
|---|---|
| verdicts of 2026-10-25 (no `self`; `ref mut` at a call; a new word for a shared changeable link; rework of `if Some(ref mut me) = ui:`) | yes in every NEW block and message; `from(self)` left in the vocabulary (F-13) |
| 1 `holds` | yes |
| 2 `ref mut` keeps reads out too | yes (AN-01, AN-02, AN-13, AN-23, AN-28, AN-37, AN-44, AN-51) |
| 3 `MutSpan` says `ref mut` at the call; `Mutex` and friends say nothing | yes (AN-11, AN-14, AN-28, AN-30, AN-84) |
| 4 `shares mut` | yes (area C, AN-78) |
| 5 a read-only name's promise | yes (AN-01, AN-13, AN-22, AN-24, AN-62) |
| 6 `changes this` after the brackets | yes (AN-18, AN-32, AN-60, AN-61, AN-70) |
| 7 `static fn` | yes in the rules (R5, E2115); no answer needs one |
| 8 `static mut` | yes (AN-76, AN-84) |
| 9 `weakref`; no `Some(...)` in a header | yes |
| 10 another link is `weakref x.keep`; `copy of` a link refused | yes (rules E2116, AN-62, AN-65) |
| 11 every `weakref` counted | yes (R11; costs in AN-41, AN-62, AN-63) |
| 12 `copy of` | yes, except `.copied()` (F-12) |
| 13 a lost change refused, both ways out | AN-10 yes; AN-23's message no (F-02) |
| 14 `set by` removed | yes (AN-39, AN-41, AN-62, AN-63, AN-64) |
| 15 a walk stops at the first deleted parent | yes (R11, AN-62 (b)) |
| 16 one rule for every loop | yes (AN-42, AN-77) |
| 17 functions and lists of functions say what they change | yes by the rules' letter; concern C-1 |
| 18 `changes this` promises nothing about other names | yes (R4, R5, AN-18, AN-19) |
| 19 `owns`, `shares`, `using` | yes (AN-40's `owns Mesh using LevelArena`) |
| 20 `thread_shared`, `sharedref mut` kept | yes |

## 6. Concerns (meet the tests' letter; for the owner's judgement, not counted as failures)

- **C-1. "Changes nothing outside itself" counts the real variables a lambda captures as itself.**
  R12 and the vocabulary (section 1) read decision 17 so that row 6's `fn(): score += 1` is a plain
  `fn()`: `game`'s `score`, captured as the real variable, is "part of the function itself". So
  `handlers: Array[fn()]` (AN-53), `hs: Array[fn()]` (AN-58), AN-71's and AN-74's tasks, AN-55's and
  AN-56's lambdas are plain `fn()` though they change a variable of the function that made them, and
  nothing on the list's type or at `b.click()` says so. Safety is kept (the function that writes the
  lambda checks it), and row 6 is as agreed; but the owner's words were "plain `fn()` means 'changes
  nothing outside itself'", and a reader of `Array[fn()]` may take `score` to be outside. Area D's note 1
  shows the safety edge of the same reading (a lambda stored inside the variable it changes). For the
  owner (section 7).
- **C-2. The first owner of a shared value keeps what it could do, with no word on its own lines**
  (`items` after `other = shares items`, AN-34; `child` after `push(shares mut child)`, AN-36; `p`,
  `m`, `player`, `a`, `game` in AN-39, AN-44, AN-45, AN-48, AN-78). The word is on the sharing line.
  Already the owner's question 3 in `V2/changes.md` section 11.
- **C-3. Two `if`s in a downcast opening**, `if cast s if Circle holds c:` (AN-67, AN-68). Already the
  owner's question 5.
- **C-4. Calling a stored function needs `changes handlers` / `changes on_visit`** (AN-53, AN-58,
  AN-62, AN-76), which reads as "re-assigns the field". It is Rust's `FnMut`; carried from run 4.
- **C-5. `=` on a `ref mut` name writes the place it was made from** (`r = 20` writes `a`, AN-28;
  `r = 7` writes an element, AN-11), while `=` on a plain view name re-points it. One sign, two effects,
  told apart by how the name was declared; carried from run 4 (R2, G-08).
- **C-6. The word that lets a plain name change something is in its type, not on the name**: `MutSpan`
  parameters, `Mutex` (`g` from `scores.lock()`), `Atomic`, `guarded by` fields. Decision 3 accepts it
  for calls; for declared names it passes only because the type sits on the same line (and see F-14).

## 7. For the owner

1. **C-1**: does "a plain `fn()` changes nothing outside itself" (decision 17) cover a lambda that
   changes a variable of the function that made it, as row 6's `fn(): score += 1` does? Today's rules say
   yes (it is "part of the function itself"), so `handlers: Array[fn()]` may hold it and `b.click()`
   says nothing about `score`. The other reading would make row 6's handler `fn() changes score` and the
   list `Array[fn() changes score]`, which row 6 as agreed does not write.
2. **F-12**: may the iterator adapter keep the name `.copied()`, beside the keyword `copy of`? It is
   today's library name and means a copy of each item, but it is the spelling decision 12 retired.

## 8. Files

- `C:\Users\ism19\Code\ember\tasks\96\redesign\run4\v2\readability.md` (this file).
- Under `C:\Users\ism19\Code\ember\target\redesign4\v2-07-readability\`: `extract.py` (the block
  extraction and the mechanical checks), `blocks.txt` (every block, labelled OLD or NEW),
  `mech_checks.txt` (the one hit), `prose_old_spellings.txt` (the 421 inline-code hits read by hand),
  `strip_old.py` and `new_area-A.md` .. `new_area-E.md` (the area files without their OLD and "means
  exactly" parts, original line numbers kept).
