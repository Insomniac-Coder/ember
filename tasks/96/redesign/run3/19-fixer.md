# 19 fixer: report (run 3, phase 3, round 1)

Power: `PowerLineStatus` printed `Online` at the start, before the first edit, before the AddressSanitizer
run and before writing this report. No compiler change, no git command, no Ember program run (only
`target/redesign3/bin/ember.exe check`). Files written: this report, `RUN3/combined-v3.md` (made with `cp`
from `combined-v2.md`, then edited; `combined-v2.md` is untouched) and the files under
`target/redesign3/19-fixer/` (section 7).

## 0. Terms

- **the model**: `RUN3/combined-v3.md`. Every change in it is marked **19:** and names the problem it fixes.
- **problem**: one entry of the list I was given: R18-01 .. R18-22 from `18-readability.md`, and 26 "VOC"
  entries from `15-vocabulary-review.md` section 5 (a name in a NEW block with no row in the vocabulary).
- **NEW block**: an answer's proposed code. **refusal**: a program the model does not accept, with its message.
- **the owner's test**: for every name in a line, the word that says what it is (own, copy, handed over)
  and the word that says whether it may change it; "none (default: ...)" only where the default never
  changes, aliases or copies anything unseen.
- **the error-message rule**: a refusal names the program's own names, says why in one line, and gives the
  exact line to write; a form accepted but not doing what its words mean elsewhere is refused.
- **view name**: a name that refers to someone else's place (a plain parameter, a loop or pattern name, a
  name bound to a part such as `n = p.name`, a `ref mut` name). **own name**: a name holding its own value.
- **fixed**: the model now says one thing, and the test that caught the problem passes on the new text.
  **needs the owner**: the only fix changes or adds a word, which my duty forbids; left open, listed in
  the model's section 9.

## 1. What I did

1. Read the brief, `JOB.md`, my duty, `14-combiner.md`, `15-vocabulary-review.md`, `16-safety-one-thread.md`,
   `17-safety-threads.md`, `18-readability.md`, and the parts of the model every problem touches (rules R1 to
   R17, sections 1 to 9, and answers AN-02, 05, 11, 15 to 17, 20, 28, 30, 34 to 38, 43 to 48, 50, 52 to 54, 59
   to 62, 70, 71, 73, 74, 76 to 79, 81, 84).
2. Agents 16 and 17 stopped early with no findings, so there was no AddressSanitizer proof of theirs to
   re-run. Where a fix changed a NEW block whose C exists, I re-ran that C (AN-62, section 3).
3. Fixed each problem in `combined-v3.md` with rules and text only: no word added or changed.
4. Checked the result with `target/redesign3/19-fixer/check_v3.py`: every agreed spelling of rows 3 to 9
   appears at least as often as in `combined-v2.md` (11 spellings; `fn() with copied score:` 3 -> 4, the others
   equal); the 84 answers are all there, in order, each with its eight parts; 84 **19:** marks. Output:
   `check passed`. The file keeps `combined-v2.md`'s CRLF line ends (11,103 lines).

## 2. Results

| problem | status | where in `combined-v3.md` | test re-applied | passes |
|---|---|---|---|---|
| R18-01 | fixed | R5 (one exception, one sentence), R4 (what `ref mut` adds, one rule), AN-11, AN-30, AN-48 | owner's test on `pos`, `scores`, `total`, `b` | yes |
| R18-02 | fixed | R12, AN-53 (`click`, new `game_copy`), AN-60, AN-62 | owner's test on `self`, `h` in `click`; row 6 in AN-53's own `Button` | yes |
| R18-03 | fixed | R2, AN-28 | one meaning of `=` on a name that already has a value | yes |
| R18-04 | fixed | R5, R16, AN-76 (rule 2, `main`) | one scope of `changes` for methods and free functions | yes |
| R18-05 | fixed | section 8, AN-84 (table and a message) | E7016's meaning against AN-74's and AN-84's accepted loops | yes |
| R18-06 | fixed | R12, section 1.3, AN-59 | mode on the name (R4); `ember check` | yes |
| R18-07 | fixed | AN-43 | error-message rule against R4 | yes |
| R18-08 | fixed | AN-47, AN-54, AN-78, section 8 | one code, one text, the program's names | yes |
| R18-09 | fixed | section 8 (rewritten), AN-11, 35, 37, 38, 44, 45, 46, 52, 78, 79 | one meaning per code, one code per mistake | yes |
| R18-10 | fixed | R14, section 1.3, AN-77 | agreed limitation for every `for` loop | yes |
| R18-11 | fixed | AN-70 | R5's `virtual` rule against the intent; `ember check` | yes |
| R18-12 | fixed | AN-71 | owner's test on `names` (line 17) | yes |
| R18-13 | fixed | R2, section 1.2, AN-62 | one rule for `weak` with `=`; AddressSanitizer re-run | yes |
| R18-14 | needs the owner | section 9 point 7 | not re-run: needs a change of a word | no |
| R18-15 | fixed | R3, AN-15, AN-16's table, AN-54 | one comma rule for capture lists and loops | yes |
| R18-16 | needs the owner | section 9 point 6 | not re-run: needs a call-site word | no |
| R18-17 | fixed | AN-05 | the text says the order and the stale-position case | yes |
| R18-18 | fixed | AN-35 | error-message rule against the NEW signature | yes |
| R18-19 | fixed | AN-46, AN-79, AN-81 | every name in a message is in its program; every refusal has a code | yes |
| R18-20 | fixed | R4, AN-02 | stated once, with the line to write; `ember check` | yes |
| R18-21 | fixed | AN-34 | the block no longer uses a name it handed over | yes |
| R18-22 | needs the owner | section 9 point 8 (and point 4) | not re-run: a judgement on a word's promise | no |
| VOC-len .. VOC-Add (26) | fixed | section 1.4 (new) | each name found in the spec with the meaning the NEW code uses | yes |

## 3. Each fix, and the test that caught it applied again

**R18-01.** R5 now states its one exception in one sentence: a plain name may change what a `MutSpan`,
`Mutex`, `RwLock`, `Atomic` or channel end leads to, only with that type's own operation written at the use
(`pos[i] = ...`, `with g = m.lock():`, `fetch_add`, `send`). Reason given in the model: no owner may change a
`thread_shared` value directly (R9), so across threads these are the only way, and a `MutSpan` is the
changeable view itself. `Cell` stays on the default side (B's ruling): on one thread `ref mut`, `changes`,
`mut self` or a `shared` owner says the change at no cost. R4 now gives `ref mut` one meaning for every type:
it names the caller's place, so it gives every change through the name and `x = v` into that place; on a
`shared` link that is the re-point and the player, on a `MutSpan` the re-point only (the elements were
already writable). Test: `pos: MutSpan[float]`: what it is `MutSpan` (a view of the caller's elements), may
change `MutSpan` (R5's exception, stated); `scores: Mutex[...]`: `Mutex`, the list inside through `lock()`;
`ref mut b: shared Player`: `b = shared q` re-points the caller's link, which the line now says by R4.

**R18-02.** R12: the type `fn()` cannot say whether a value changes its own state, so the check is made where
the value is known (a parameter: AN-54's refusal stays), and a call through a stored function value counts
as a change of the field or list that holds it. AN-53's `click` becomes `fn click(self) changes handlers:` with
`for ref mut h in self.handlers: h()`; `game_copy` shows row 6's copy spelling given to that `Button` (its
"means exactly" is AN-53's own OLD `an53_button_owned_fn.em`, `exit=0`, and `m53_button_box.em` already used
`iter_mut()`); AN-62's `depth_total` says `changes on_visit`; AN-60's second way out (read-only `fn(Ui)`
handlers in a `for` loop) is removed with the reason. Test: `self` in `click`: `changes handlers`; `h`:
`ref mut`, may change, its own state included; the agreed copy spelling now has a defined meaning in the
model's own program. Safety gained: a plain name never runs a function that changes its own state, so two
tasks cannot run one at once (E7007). Cost: section 5.

**R18-03.** R2: no view name is pointed elsewhere (AN-28's rule, row 8's `path`, AN-11's `pos`, all already
said so for their names). `=` on a name that already has a value means "put this value where the name
refers": an own name is replaced, a `ref mut` name writes into the original, a plain view name is refused
(E3023) with "use a new name: `m = q.name`". The names that refer to a place are listed (parts written as a
place, parameters, loop and pattern names); a name given a call's result (`word = line.to_lower().trim()`,
`w = it.next()`) is an own name, so Python's `w = it.next()` loops stay. Test: `n = q.name` after `n = p.name`:
refused with the line; `r = 20` after `r = ref mut a`: writes `a`, which R2 now says is what `=` does
everywhere; `n = String("x")` after `n = p.name`: refused, the line given. One shape, one meaning.

**R18-04.** R5: a function's first line names what its body changes, the calls it makes included, for a
method and a free function alike. AN-76's rule 2 ("callers write nothing") is replaced, and its `main`
becomes `fn main() changes FRAME, LOG:` (it calls `tick()`); a function started by `thread.spawn` is not a
call (E7012 checks it). R16 notes that only scoped tasks' summaries stay inferred. Test: AN-17's refusal
and AN-76 now follow one rule; a reader of `fn main()` sees that it changes `FRAME`.

**R18-05.** E7016 means: a `@parallel` loop over something that is neither a range nor a list walked by
element (for example a `while` loop). The two accepted forms are no longer named as refused. AN-84 gains
its message ("`@parallel` splits a range or a list into chunks, and this `while` loop is neither", with
both lines to write).

**R18-06.** `fn call_once(owned f: once fn()):`, and a plain `f: once fn()` is refused with that line.
`target/redesign3/19-fixer/m59_owned_once.em` (`fn call_once(owned f: once fn()):`) gives `exit=0` today.

**R18-07.** AN-43's message: "to give it a second owner declare `owned n: shared Node`".

**R18-08.** One code, E0109, and one text in AN-47, AN-54 and AN-78: "`owned fn` does not say what the
function takes or copies; name it on each captured name: ...; a name not listed is the real variable". The
line to write uses each program's names (AN-78: `fn() with shared game:`, its program has `game`; AN-47:
the lambda without the word, since `task` captures nothing).

**R18-09.** Section 8 is rewritten from every code a refusal gives (18's `codes.txt`, checked again): 32
codes, each one meaning. A view used after what it points into ended or moved is E3060 everywhere (AN-35,
37, 46, 52 moved to it; AN-50 and AN-73 already had it); a list changed while a `for` loop walks it stays
E3020; a change through a name that only reads is E3023 everywhere (AN-38, 44, 46 moved from E3021); E0100
keeps today's parser meaning only (AN-38's `mut path` gets the new code E0113, AN-78's `owned fn` E0109);
AN-45's `.clone()` moves from E2020 (a type mismatch) to E1010; AN-35's `v.push(template)` from E3041 to
E3013. E0113 and W3013 are new: neither appears in `compiler/**/*.rs` or the spec (checked with grep over
both: the compiler has E0099 .. E0112 and only W3012 in that family).

**R18-10.** R14 and AN-77: a generator that walks a list with `for` is a view of it (R6), so the list may
not change while the generator is still used, in place or stored or passed (E3020, with the message for
`g = d.replay()`, `d.log.push(...)`, `g.next()`). A generator meant to see the list at each resume walks
by position with `while i < len(self.log):`, which its line then says. The position in the frame stays.

**R18-11.** AN-70's NEW: `virtual fn on_update(mut self, dt: float):`, with a `SwingDoor` override that sets
`angle` (the spec's own `Door`, spec 1704). Today's Ember accepts that program
(`m70_virtual_changes.em`, `exit=0`; a class method's `self` may write fields today, which is the NEW `mut
self`). The refusal for the plain-`self` virtual is added with its line.

**R18-12.** AN-71's `share_and_keep` creates `names = thread_shared Names()` (area E's own ruling and AN-80's
spelling), so the line that makes `names` says it has owners on several threads and may not be changed
directly; a `with thread_shared` capture of a plain local is refused with that line. The one-thread case
(`other = shared items` on a later line) is row 9's own advice ("the error names ... `shared`"), so it stays:
the word sits on the line that names `items`. Test: `names` (line 17): `thread_shared`, may change: no (R9).

**R18-13.** R2 no longer copies a `weak` link bound from a part; `weak` follows R1 like every non-`Copy`
type (`a = n.parent` views the link, `a = copied n.parent` is your own link, `w2 = w1` hands over). AN-62
writes `a = copied n.parent` / `a = copied p.parent`. A copied link adds to the target's weak count (as in
area D's own `c/an63_links.c`, `weak_store`), left out where R10's points 1 and 2 hold; section 1.2 has the
new row. The C of AN-62 is unchanged (`a` is a plain pointer inside the deferred-drop region), and I re-ran
it under AddressSanitizer (`target/redesign3/19-fixer/c/an62_defer_asan_output.txt`, built with
`clang -O1 -g -fsanitize=address -I target/redesign/bench` from area D's `c/an62_defer_asan.c`):

```text
--- mode 1
==45092==ERROR: AddressSanitizer: heap-use-after-free on address 0x1298552a0918 at pc 0x7ff7c1743211 bp 0x00c42d98fa20 sp 0x00c42d98fa68
READ of size 8 at 0x1298552a0918 thread T0
    #0 0x7ff7c1743210 in main C:\Users\ism19\Code\ember\target\redesign3\11-area-D\c\an62_defer_asan.c:89
--- mode 2
mode2 depth=1 leaf.parent=None
an62_defer_asan ms=0.058 peak_kb=8264 checksum=1
--- mode 3
mode3 depth=1 leaf.parent=None
an62_defer_asan ms=0.053 peak_kb=8248 checksum=1
```

Mode 2 is the fixed program's code (no weak count on `a`, the region keeps every node's block): no report.

**R18-15.** R3: a word belongs to the one name after it; where a word stands before the first of several
names (a capture list, a loop's names), every name writes its own word. AN-15 becomes `for ref mut pos,
ref vel in q:` (`ref` is the existing word for the read-only default, R2); `for ref mut pos, vel in q:` and
`with copied name, score` get the same advice ("`ref mut` here belongs to `pos` only; say what `vel` is").

**R18-17.** AN-05 now says that the handlers run last-dead-first (the reverse of the OLD order) and why, how
to keep the OLD order (a read-only handler, then `retain`), and what happens when a handler removes another
enemy (a later position names another enemy, or stops at the index check; never a memory error).

**R18-18.** AN-35: "`v.push(template)` would hand over the caller's player, but `template` only reads it" /
"for a player of your own in each slot write `v.push(copied template)`" (E3013).

**R18-19.** AN-46's refusal is now given on a written program (`ks = sorted(m.keys())`, `m.insert(...)`,
`println(ks[0])`), so it names `ks`; AN-81's on AN-03's `step` marked `@no_alias_checks`, so
`collide(bodies[i], bodies[j])` is in its program; AN-79's warning has the code W3013.

**R18-20.** R4: a plain parameter is never changed in the function, a number included; a changeable copy is
a new own name (`count = n`, `mine = copied p`). Today's compiler already refuses `n += 1` on `n: int`
(`o20_plain_int_param.em`): `error[E3023]: cannot mutate borrowed parameter \`n\``, `exit=1`; the line the
NEW message gives (`m20_int_param_copy.em`, `count = n` then `count += 1`) gives `exit=0`. AN-02's row for
`x` no longer implies `x += 1` is accepted.

**R18-21.** AN-34's two other spellings are two functions of their own (`hand_over`, `two_owners`), as AN-36
shows its forms; the agreed spellings of row 9 are unchanged.

**VOC (26 names).** Section 1.4 lists each with its spec meaning and line (prelude table 1538, `[STD-26]`,
`[TXT-10]`, `[STD-16]`, `[STD-19]`, `[THR-10]`, `[THR-14]`, `[SPN-1]`, `[SPN-8]`, `[SOA-3]`, `[ECS-7]`,
`[STD-21]`, `[TYP-43]`, std.ffi). All 26 exist in the spec (grep over
`docs/spec-source/Ember_v0.9.10_Hardened_35.md`), none says who owns or changes a value, so none is a word of
the memory model and none was added to or changed in the vocabulary. In AN-67, `inspect` is the program's own
method name.

`ember check` outputs of the four files, all in `target/redesign3/19-fixer/check_output.txt`:

```text
===== m59_owned_once.em
exit=0
===== m70_virtual_changes.em
exit=0
===== o20_plain_int_param.em
error[E3023]: cannot mutate borrowed parameter `n`
 --> o20_plain_int_param.em:3:5
  |
2 | fn add_one(n: int) -> int:
  |            ------ `n` is borrowed here
3 |     n += 1
  |     ^ this write needs mutable access
  |
  = help: bind the new value to a new local (`new_n = …`) and use that from here on; declare the parameter `mut n: i64` only if the caller is to see the new value, as a single owner passing it `mut`
  = note: Python rebinds a parameter's name; an Ember parameter is the caller's `i64`, borrowed [FN-1]

error: could not compile due to 1 error(s)
exit=1
===== m20_int_param_copy.em
exit=0
```

## 4. Left open: needs the owner

Each is in the model's section 9 (points 6 to 11) with what the model does meanwhile.
- **R18-14** (`mut self`, `static mut`): the spelling that passes row 8's test as every other name does would
  be `ref mut self`; a change of a word.
- **R18-16** (`bump(a)` changes `a` with no word in the line): needs a call-site word (open point 1).
- **R18-22** (`ref mut` promises "the only way in" on a parameter, not on a kept view): a judgement on what a
  word promises (open point 4).
- Options a new word would open, noted, not taken: a type word for "a function value that changes no state
  of its own" (Rust's `Fn` against `FnMut`), which would let AN-53's `click(self)` call through plain `h`
  again (point 9); `weak` made `Copy`, so `a = n.parent` copies with no word (point 10); a re-point the line
  says (C#'s `r = ref b`), which needs AN-28's ruling and the `ref` row changed (point 11).

## 5. What the fixes cost

- **Readability, against Python**: a method that calls stored callbacks writes `changes handlers` and walks
  them with `ref mut h` (R18-02), Rust's `&mut self` + `iter_mut()` in words; `line = line.strip()` on a
  parameter and "best so far" (`best = e` over views) are refused with a new name, a position or a copy
  (R18-03); a walk up `weak` links writes `copied` (R18-13); `main` names the module variables its calls
  change (R18-04). Each is written in the model's section 9 or in the answer.
- **Run time**: nothing new, except a weak-count add and subtract for a `copied` `weak` link where no
  deferred-drop region or R10 point 1 covers it (Rust's `Weak::clone`); none in AN-62's measured walk.
  Every other fix is a refusal or a message, checked when compiling.

## 6. Vocabulary rows that must follow (for the lead; `vocabulary-final.md` not edited)

No word's spelling changes; these rows state what a rule now says:
`Copy` (drop "R2's one exception": a `weak` link bound from a part is no longer copied); `weak` (`a =
n.parent` views the link, `copied` copies it); `ref` and `ref mut` (no view name is pointed elsewhere; `=`
on a plain view name is refused, R2; `ref mut` on a link or view parameter covers the place, R4); `once`
(a parameter is `owned f: once fn()`); `fn` (a call through a stored function value changes its holder,
R12; the capture-list refusal's first clause); `for` (R3's one rule for a word before a list of names);
`changes` and `static` (callers name what their calls change, R5); `Mutex`, `RwLock`, `Atomic`, `MutSpan`,
`Cell` (R5's one exception); `virtual` (AN-70's example now `mut self`). The error codes in section 8
(E0113 and W3013 new).

## 7. Files

- `C:\Users\ism19\Code\ember\tasks\96\redesign\run3\combined-v3.md`: the model, 84 **19:** marks.
- `C:\Users\ism19\Code\ember\tasks\96\redesign\run3\19-fixer.md`: this report.
- `C:\Users\ism19\Code\ember\target\redesign3\19-fixer\`: `check_v3.py` (the check of section 1, step 4),
  `v2_to_v3.diff` (every line changed), `m59_owned_once.em`, `m70_virtual_changes.em`,
  `o20_plain_int_param.em`, `m20_int_param_copy.em` and their `check_output.txt`,
  `c/an62_defer_asan.exe` and `c/an62_defer_asan_output.txt` (the AddressSanitizer re-run). `target/` there is
  `ember check`'s own cache.
