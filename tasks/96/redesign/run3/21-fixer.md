# 21 fixer: report (run 3, phase 3, round 2)

Power: `PowerLineStatus` printed `Online` at the start, before the AddressSanitizer runs and again
before writing this report. No compiler change, no git command, no Ember program run (no `ember check`
was needed: every claim about today's Ember below cites output agent 19 or 20 already pasted). Files
written: this report, `RUN3/combined-v4.md` (made with `cp` from `combined-v3.md`, then changed only by
`target/redesign3/21-fixer/apply_v4.py`; `combined-v3.md` is untouched) and the files under
`target/redesign3/21-fixer/` (section 6).

**Result: all 12 problems fixed, with rules and text only; no word added or changed. One error code was
added (E7017). The four AddressSanitizer proofs (F-01 to F-04) were re-run: the C of the v3 model
still reads freed memory, the C of the fixed model (or the program the fixed model's message leads
to) runs clean.**

## 0. Terms

- **the model**: `RUN3/combined-v4.md`. Every change in it is marked **21:** and names the problem it
  fixes (F-01 .. F-12, `RUN3/20-re-attacker.md`); 69 marks.
- **AddressSanitizer (ASan)**: a memory checker built into the C compiler; "heap-use-after-free" means
  the program read memory already given back.
- **mode 1 / mode 2**: each C proof runs twice. Mode 1 is the C the v3 model lets through (20's proof);
  mode 2 is the C the fixed model emits, or the program its new message tells the programmer to write.
- **view**: a name or value that points into something someone else owns (`ref T`, `ref mut T`,
  `Span[T]`, `MutSpan[T]`, `StrView`, a plain parameter, a plain loop or pattern name). **own value**: a
  value the name holds itself (a number, a `String`, a list, a link of its own, a struct).
- **weak count**: the number in an object's header that keeps its memory block (not the object) until
  the last `weak` link to it ends. **deferred-drop region**: a loop during which an object whose last
  owner ends is set aside whole and freed when the loop ends (area D, AN-62).
- **the owner's test**, **the error-message rule**: as in the brief.
- **fixed**: the model now says one thing, and the test that caught the problem passes on the new text.

## 1. What I did

1. Read the brief, `JOB.md`, my duty, `20-re-attacker.md`, `19-fixer.md`, 20's four C proofs and its
   `today/` checks, the model's sections 0 to 9 and every answer the problems touch (AN-03, 11, 14, 15,
   22, 23, 28, 53, 60, 61, 62, 71, 75, 76, 77, 78, 81, 84), area D's rules D1 to D8, and the spec's
   `[GRM-4]` (spec 659-662: a name is declared in the block where it is first given a value).
2. Wrote every change as an exact replacement that must match once (`apply_v4.py`, 56 replacements),
   applied to a fresh `cp` of `combined-v3.md`; the file keeps its CRLF line ends (11,338 lines).
3. Copied 20's four C proofs to `target/redesign3/21-fixer/c/` (a header note added, code unchanged),
   built each with `clang -O1 -g -fsanitize=address -I target/redesign/bench` and ran modes 1 and 2;
   re-ran area D's `an62_defer_asan.c` (modes 2 and 3) to show AN-62 is unchanged by F-01's fix.
4. Re-applied the readability tests: the owner's test on every name a fix writes, the error-message
   rule on every new message, and two scans of the NEW blocks (section 4).
5. `check_v4.py` (output in `check_v4_output.txt`): 19's check (every agreed spelling of rows 3 to 9 at
   least as often as in v3; 84 answers in order, each with its eight parts) passes, every problem has a
   **21:** change, and the faulty v3 forms 20 quoted are gone. `v3_to_v4.diff` has every changed line.

## 2. Results

| problem | status | where in `combined-v4.md` | test re-applied | passes |
|---|---|---|---|---|
| F-01 | fixed | R2 (weak count of a copy), 1.2 table, AN-62 note | ASan, `f1_weak_copy_escape.c`; AN-62's C re-run | yes |
| F-02 | fixed | R4 (what a view put in through `ref mut` may point into), R2, 1.3, section 8 (E3023, E3060, E3063), section 9 point 12 | ASan, `f2_refmut_view_repoint.c` | yes |
| F-03 | fixed | R4 (a `MutSpan` of non-`Copy` elements is a `ref mut` argument for the "two places" promise), 1.2, 1.3, AN-14 | ASan, `f3_mutspan_overlap.c` | yes |
| F-04 | fixed | R5 (function values, the one exception, said), R16, AN-76 rules 2 and 3 and two refusals, 1.3, section 8 (E2105, E3020), section 9 point 9 | ASan, `f4_fnvalue_static_loop.c`; owner's test on `click`, `press` | yes |
| F-05 | fixed | R2 (a `ref`/`ref mut` call result is a place: `r = ref mut first_even(...)`), AN-11, AN-22, AN-23, AN-28, section 9 point 13 | owner's test on `r`; one meaning of `=` | yes |
| F-06 | fixed | R2 (a name whose value is a view is given its value once, wherever the view came from), AN-28, 1.3, section 3, section 8 (E3023), section 9 point 11 | one rule for every view name; scan of the NEW blocks | yes |
| F-07 | fixed | R12 (what "a change of the place that holds it" means for R4 and R8 (c)), AN-60 (second way out restored) | consistency of AN-60's two forms; owner's test on the restored form | yes |
| F-08 | fixed | R3 (`thread_shared x` only on a value already `thread_shared`), AN-71 (note and three messages), AN-75, AN-78, 1.3, section 8 (E7017 new) | owner's test on `names`; one code, one meaning | yes |
| F-09 | fixed | R14 (`from log` on the generator's first line; E3062 / E3020 / E3060), AN-77 (NEW, test, refusals), 1.3, section 8 | owner's test on `g`, `log`; one mistake, one code | yes |
| F-10 | fixed | section 8 (E0113 excepts `mut self` and `static mut`), 1.3 | E0113's meaning against AN-70's accepted line | yes |
| F-11 | fixed | AN-84 (the refused `place` written out, the message in its names), AN-81 (help) | error-message rule: every name in the message is in its program; no help that does not apply | yes |
| F-12 | fixed | R3 (the comma rule for every list of names and a word on any name) | owner's test on `vel` in `for e, ref mut pos, vel in q:` and in a pattern | yes |

None needs the owner to close it. Two options a new word or a new position of a word would open are
noted for the owner in the model's section 9 (points 12 and 13), not taken.

## 3. Each fix, and the test that caught it applied again

### F-01: a copied `weak` link stored in a field takes its weak count (R2)

**Fix.** The weak-count add is left out only where the copy's **whole life** is covered: by R10's
point 1, or by a deferred-drop region that ends after the copy does (a local of the loop's body, as
AN-62's `a`). A copy stored in a field, a list or a capture, handed to an `owned` parameter, returned,
or kept in a local that outlives the region takes its count when it is made (Rust's `Weak::clone`).
The 1.2 cost row says the same. AN-62's own walk is unchanged: `a` is a local of the `for` body.

**Test: ASan** (`c/f1_weak_copy_escape_output.txt`). 20's program: `self.top = copied a` inside the
walk, the first root ended after it, then `if Some(t) = tree.top:`.

```text
--- mode 1
==114460==ERROR: AddressSanitizer: heap-use-after-free on address 0x1248d92a0448 at pc 0x7ff7e8643538 bp 0x00eb9478fa60 sp 0x00eb9478faa8
READ of size 4 at 0x1248d92a0448 thread T0
    #0 0x7ff7e8643537 in main C:\Users\ism19\Code\ember\target\redesign3\21-fixer\c\f1_weak_copy_escape.c:116
...
--- mode 2
mode2 depth=2
top=gone
f1_weak_copy_escape ms=0.067 peak_kb=8260 checksum=2
exit=0
```

Line 116 is `if Some(t) = tree.top:` (the header test on a freed block). Mode 2, the fixed rule (the
copy stored in `tree.top` takes its count), prints `top=gone`, no report. AN-62's walk, area D's C
re-run (`c/an62_defer_asan_output.txt`): `mode2 depth=1 leaf.parent=None` and `mode3 depth=1
leaf.parent=None`, no report.

### F-02: what the caller's view points into after a `ref mut` call (R4, R2)

**Fix.** After a call, the caller's place given to a `ref mut` parameter whose value is a view may
point into whatever the call's other view arguments point into, as well as into what it pointed into
before (R6's "into all of them", applied to what the call may put there), so R8 checks every later use
against all of them (E3060). Inside the callee only such a view may be put there; a view of the
callee's own locals is refused (E3063, message in R4). A caller's view **name** is given its value once
(F-06's rule), so it is not given to such a parameter (E3023): "`pick_longer` may point `best`
elsewhere (its first line says `ref mut best`), but `best` views `a` for its whole life; let the
function return the view to keep: `fn longer(x: Span[int], y: Span[int]) -> Span[int]:`, then
`best2 = longer(best, b.as_span())`". The place given is a field or list element holding a view.

**Test: ASan** (`c/f2_refmut_view_repoint_output.txt`). 20's program is now refused at
`pick_longer(best, ...)`; the field form (`pick_longer(c.best, b.as_span())`, `b = [6]`, `c.best[0]`)
is refused at the last line (E3060) by the new R4 rule. The C of the field form is the same as mode 1
with the span inside a struct; it was not run separately.

```text
--- mode 1
==105708==ERROR: AddressSanitizer: heap-use-after-free on address 0x118e305a0790 at pc 0x7ff6611e2cda bp 0x0018e38ffbc0 sp 0x0018e38ffc08
READ of size 8 at 0x118e305a0790 thread T0
    #0 0x7ff6611e2cd9 in main C:\Users\ism19\Code\ember\target\redesign3\21-fixer\c\f2_refmut_view_repoint.c:54
...
--- mode 2
mode2 best[0]=3
f2_refmut_view_repoint ms=0.046 peak_kb=8196 checksum=3
exit=0
```

Line 54 is `println(best[0])` after `b = [6]` (line 53). Mode 2 is the program the messages lead to
(the view used before `b`'s list ends): no report.

### F-03: a `MutSpan` of strings and another view of the same call (R4)

**Fix.** A `MutSpan[T]` argument whose `T` is not `Copy` counts, for R4's "two different places"
promise, as a `ref mut` argument: proved when compiling, or one overlap test (two compares) before the
call. When both come from one list in the caller, the call is refused when compiling: "`refill` may
replace strings of `names` through `dst` while `src` shows them; give it two different lists, or a
copy: `old = copied names`, then `refill(names.as_mut_span(), old.as_span())`". For `Copy` elements
(numbers) an overlapping write frees nothing, so AN-11 and AN-14 keep their meaning and cost.

**Test: ASan** (`c/f3_mutspan_overlap_output.txt`):

```text
--- mode 1
==18996==ERROR: AddressSanitizer: heap-use-after-free on address 0x129b0eea0132 at pc 0x7ff7b0ff2f66 bp 0x00a1d04ff7a0 sp 0x00a1d04ff7e8
READ of size 1 at 0x129b0eea0132 thread T0
    #0 0x7ff7b0ff2f65 in refill C:\Users\ism19\Code\ember\target\redesign3\21-fixer\c\f3_mutspan_overlap.c:51
    #1 0x7ff7b0ff2b5b in main C:\Users\ism19\Code\ember\target\redesign3\21-fixer\c\f3_mutspan_overlap.c:60
...
--- mode 2
panic: `refill(names.as_mut_span(), names.as_span())`: `dst` and `src` overlap
exit=101
```

Line 51 reads `w` after `dst[0] = String("new")` (line 49) freed its text. Mode 2 is the one overlap
test where the compiler cannot prove the lists differ: the program stops before anything is freed.
(Today's Ember refuses the same call, E3021: 20's `today/t3_overlap_spans.em`, `exit=1`.)

### F-04: a function value that changes a `static mut` (R5, R16, AN-76)

**Fix.** R5 now says its sentence "a reader of any first line sees everything a call of it may change"
has one exception, function values, and how it is handled: (a) a lambda's body is part of the function
that writes it, so that function's first line names a module variable the lambda changes (E2105
otherwise, message in R5 and AN-76); (b) a call that may run a function value (a call through one, an
unknown call, or a call of a function whose first line lets it change a place holding function values:
`changes handlers`, `mut self`, a `ref mut` parameter, R12) is taken to change every variable that a
function value of the program changes: D3's rule, inferred and recorded in the interface file (R16 now
lists it). AN-76's rule 3 uses it, with the message: "this loop walks `LOG`, and `b.click()` (line 17)
runs the handlers in `b.handlers` (`fn click(self) changes handlers:`), one of which, made on line 15,
pushes to `LOG`; click after the loop, or collect what to add and push it after the loop" (E3020).

**Test: ASan** (`c/f4_fnvalue_static_loop_output.txt`):

```text
--- mode 1
==109624==ERROR: AddressSanitizer: heap-use-after-free on address 0x1236ce0a0448 at pc 0x7ff7332230f6 bp 0x004f0372f5b0 sp 0x004f0372f5f8
READ of size 8 at 0x1236ce0a0448 thread T0
    #0 0x7ff7332230f5 in print_entry C:\Users\ism19\Code\ember\target\redesign3\21-fixer\c\f4_fnvalue_static_loop.c:57
    #1 0x7ff733222c5b in main C:\Users\ism19\Code\ember\target\redesign3\21-fixer\c\f4_fnvalue_static_loop.c:70
...
--- mode 2
mode2 log=5
f4_fnvalue_static_loop ms=0.064 peak_kb=8232 checksum=2232
exit=0
```

Mode 1 (the loop accepted on first lines alone) reads `entry` from the buffer the handler's push moved
(`log_clicked`, line 53). Mode 2 is the program the new refusal leads to (the click after the loop):
no report. **Owner's test:** on 20's `fn press(ref mut b: Button): b.click()`, `b` is `ref mut`: `press`
may change `b`, which includes running its stored handlers (R12); what a handler changes is written in
its body and named on the first line of the function that writes it. `click`'s `changes handlers` says
a call runs them. R5's sentence is now true as written, with its one exception stated.

### F-05: `r = first_even(...)` (R2)

**Fix.** A call whose result type is `ref mut T` (or `ref T`) names a place (R6 already said so), so a
name bound to it follows R2's rule for parts: `r = ref mut first_even(a.as_mut_span())` is a `ref mut`
name of the element and `r = 7` writes it, as AN-28's `r = 20` writes `a`; plain `r = first_even(...)`
copies the number, as `n = counts[0]` does. AN-11, AN-22 and AN-23 now say that their means-exactly
lines (`r = first_even(...)`, `p = reg.get_mut(0, 0)`, `f = first(xs)`) are today's spelling of the NEW
`ref mut` lines. No NEW block binds such a call, so none changed.

**Test (owner's test).** `r = ref mut first_even(a.as_mut_span())`: `r`: `ref mut` (a name for the
element `first_even` returns), may change it: `ref mut`. Plain `r = first_even(...)`: none (default:
a number of its own, R2's stated default for numbers, which changes nothing else). `=` keeps one
meaning: "put the value where the name refers". 20's own suggestion (no word, a `ref mut` name anyway)
was not taken: no word on that line would say `r` may change the element (section 9, point 13).

### F-06: one rule for every name whose value is a view (R2)

**Fix.** "A name whose value is a view is given its value once, wherever the view came from": a part
(`n = p.name`), a loop or pattern name, or a call whose result is a `StrView`, `Span` or `MutSpan`
(`n = p.name.trim()`, `best = a.as_span()`, `a = p.next_word()`, row 5's `word`). `=` on such a name is
E3023 with "use a new name: `m = q.name.trim()`". A name holding its own value (a number, a `String`, a
list, a link of its own, a struct or generator a call makes, AN-22's `p`, AN-77's `g`) is replaced by
`=`; a name first given its value in a loop body is given it once per round (`[GRM-4]`).

**Test.** `n = p.name` then `n = q.name`, and `n = p.name.trim()` then `n = q.name.trim()`: both
refused, one message; the reader asks one question (does `n` only point at someone's text?), the one
row 5 already asks. The owner's-test tables already gave such names that default: AN-22's `rest`,
`a`, `b`: "none (default: a read-only view, as row 5's `word`)". Scan of the NEW blocks
(`scan_reassign.py`, every name given a value with `=` twice): five hits, all own values or `ref mut`
names (AN-21 `r` a number; AN-28 `r`, `rb` `ref mut`; AN-62 `a` a `copied` link; AN-69 `acc` a
`copied` value); 20's `scan_rebind.py` on v4: one hit, AN-21's number. No NEW block breaks.

### F-07: AN-60's two forms (R12)

**Fix.** R12 now says what "a call through a stored function value is a change of the place that
holds it" means: it decides which name may make the call; it does not make the list element an
argument (the call copies the two words out first, D4, and its state is in its own block). So R4 sees
only the call's arguments, and R8 (c) sees the list changed only if an argument may change it. With
that, 19's reason for removing AN-60's read-only-Ui `for` form is withdrawn, the same reason having
applied to the accepted `self.handlers[self.current](self)`; the form is restored, spelled as R12 asks:
`handlers: Array[fn(Ui)]`, `fn fire(self) changes handlers:`, `for ref mut h in self.handlers: h(self)`.

**Test.** One rule for both calls: neither makes the handler's list element an argument; the position
loop gives `self` as `ref mut` with no compare; the `for` loop gives `self` read-only, so no handler can
change the list it walks (the agreed limitation met). Owner's test on the restored form: `self`:
`changes handlers`; `h`: `ref mut`, may change it (its own state); the handler's `ui`: none (default: a
read-only view), never changes it. Safety: the handler's state is reachable only by calling it, which
R12 refuses through a read-only `ui`, so nothing else reads or frees it during the call.

### F-08: `thread_shared x` (R3, E7017)

**Fix.** R3: `thread_shared x` makes one more owner of a value already `thread_shared` (made so on its
own line, or a `x: thread_shared T` parameter); before a new value it makes the value with owners on
any thread; on a name whose value is not `thread_shared` it is refused, before a value and in a capture
list alike (it would freeze a value its own line lets its owner change). `shared x` on a plain local
stays as row 9 has it (a `shared` value may still be changed, so the earlier line keeps its meaning).
New code E7017 for this one mistake (free: the compiler's sources and the spec use E7000 to E7006,
E7010, E7011 and E7020 only). AN-71's three messages that told the programmer to write `with
thread_shared names` on its plain `names`, and AN-78's on a `shared` game, now give the line that
compiles (`names = thread_shared Names()`, then the capture); AN-75's case (a plain parameter) moved
from E7015 to E7017.

**Test.** Owner's test on `names` (AN-71, line 17): `thread_shared`, may change: no (R9), said on the
line that creates it; no later line changes what `names` may do. E7015 keeps its one meaning (a
one-thread owner or a lent value reaching another thread); E7017 has one meaning.

### F-09: the generator's first line says what it walks (R14, AN-77)

**Fix.** A generator whose `for` loop walks a list across `yield` writes R6's `from` with that list:
`gen fn replay(self) -> Generator[int] from log:` (AN-77's NEW). Without it the line is refused
(E3062, today's code for a result pointing into something the first line does not name, now listed in
section 8): "`replay` walks `self.log` with `for`, so the generator it returns views `log`; write `gen
fn replay(self) -> Generator[int] from log:`, or walk by position: `while i < len(self.log):`". With
`from log`, `g = d.replay()` is a view of `d.log`, so R8 (b) ends it at `d.log.push(...)`; the stored
generator used after that is E3060 (as AN-46's `ks`), and E3020 keeps its one meaning (a list changed
while a `for` loop walks it, the loop that consumes a generator included). The position-walk generator
points into the object (R6's default), which a push to `log` does not end.

**Test.** Owner's test on `g = d.replay()`: the word that says `d.log` may not change while `g` is used
is `from log` on `replay`'s first line, which the caller reads (and, in another package, the interface
gives as written). AN-77's owner's-test table gains the row for `log` in `from log` (11 names, 8
without a word). One mistake, one code: a view used after what it points into changed is E3060 in
AN-46 and AN-77 alike.

### F-10: E0113 and `mut self` (section 8)

**Fix.** E0113 is "`mut` on a name other than `self`"; `mut self` (R5, AN-70) and `static mut` (a
declaration) keep `mut` and are not this code (open point 7). The 1.3 row says "`mut x` on a parameter
other than `self`". **Test:** AN-70's `virtual fn on_update(mut self, dt: float):` is accepted and no
code's meaning covers it.

### F-11: two messages in their programs' names (AN-84, AN-81)

**Fix.** AN-84 now writes out the refused program (`place` with `@parallel` above `while i <
idx.len():`), applies the owner's test to its one new name (`i`), and gives the message on it:
"`@parallel` splits a range or a list into chunks, and `while i < idx.len():` (line 4) in `place` is
neither", help "count over the range: `for i in 0..idx.len():`" (the element-walk help is dropped: the
body indexes both `xs` and `idx` by `i`, so it would not apply). AN-81's help keeps only `if i != j:`
around the call (`step`'s `j` is a parameter). **Test:** every name in both messages (`place`, `i`,
`idx`, `step`, `collide`, `bodies`, `i`, `j`) is in its program; each help line applies there (A-8).

### F-12: one comma rule for every list of names (R3)

**Fix.** The rule covers capture lists, loop names and pattern names, and a word on any name: a name
with no word that follows a name with a word is refused with the same advice
(`for e, ref mut pos, vel in q:` and `State.Moving(ref mut pos, vel):` get "`ref mut` here belongs to
`pos` only; say what `vel` is: ... `ref vel` ..., or `ref mut vel` to change it too"). Names before the
first word need none (`for k, ref mut v in m:`, AN-06's `for e, ref mut h in q:`). A parameter list is
not such a list (each parameter has its own type). **Test:** owner's test on `vel`: in the line to
write, `ref` says what it is and that it only reads. A scan of the model for a worded name followed by
a wordless one finds only the refusal examples themselves.

## 4. Readability re-checks over the whole model

- Agreed rows 3 to 9: every agreed spelling appears at least as often as in v3 (19's `check_v3.py`, run
  by `check_v4.py`); none of their text was edited (`word = line.to_lower().trim()` 10 -> 11: R2 now
  quotes it once more).
- `scan_reassign.py` and 20's `scan_rebind.py` over every NEW block: no NEW block is broken by F-05/F-06
  (section 3, F-06).
- Every new message names the program's names, says why in one line and gives the line to write; the
  lines given are ones the fixed model accepts (in particular, no message offers `with thread_shared x`
  on a value that E7017 refuses: checked by `check_v4.py`).

## 5. What the fixes cost

- **Run time.** F-01: one weak-count add and subtract for a `weak` copy that is stored, returned or
  outlives a region (Rust's `Weak::clone` and its drop); none in AN-62's measured walk. F-03: one
  overlap test (two compares) per call that passes a `MutSpan` of non-`Copy` elements with another view
  the compiler cannot prove apart, the check `assert_disjoint` makes (AN-14's cost row); none for
  numbers. Every other fix is a compile-time refusal or a message.
- **Readability, against Python.** A name holding a view a call gave is given its value once (F-06),
  as part names already were (section 9, point 11); a re-pointable local view (Rust's `&mut &str`
  parameters) needs a field or a list element (point 12); `r = ref mut first_even(...)` writes the
  word where it acts (F-05); a function that writes a lambda that changes a module variable names it
  (F-04); a `for` generator writes `from log` (F-09). Each is written in the model.

Vocabulary rows that must follow (for the lead; `vocabulary-final.md` not edited; no spelling changes):
`ref`/`ref mut` (a `ref`/`ref mut` call result is a place; a view name is given its value once,
whatever made the view), `weak` (when a copy takes its count), `thread_shared` (only on a value already
`thread_shared`, E7017), `from` (a `for` generator's first line names the list it walks), `MutSpan`
(non-`Copy` elements: two places with another view argument), `fn` (a call through a stored value does
not make its holder an argument; what function values change is inferred, R5's exception), `changes`
(a lambda's change of a module variable is named by the function that writes it). Error codes: E7017
new; E3062 listed; E0113, E2105, E3020, E3023, E3060, E3063 meanings updated.

## 6. Files

- `C:\Users\ism19\Code\ember\tasks\96\redesign\run3\combined-v4.md`: the model, 69 **21:** marks.
- `C:\Users\ism19\Code\ember\tasks\96\redesign\run3\21-fixer.md`: this report.
- `C:\Users\ism19\Code\ember\target\redesign3\21-fixer\`: `apply_v4.py` (every change, each matching
  once), `check_v4.py` and `check_v4_output.txt`, `scan_reassign.py`, `v3_to_v4.diff`, and `c\`: the four
  proofs (`f1_weak_copy_escape.c`, `f2_refmut_view_repoint.c`, `f3_mutspan_overlap.c`,
  `f4_fnvalue_static_loop.c`) and area D's AN-62 re-run, each with its `.exe` and `_output.txt` (full
  ASan reports; `.pdb` files deleted to keep the disk footprint small).
