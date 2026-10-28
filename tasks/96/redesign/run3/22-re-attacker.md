# 22 fix re-attacker: report (run 3, phase 3, round 2)

Power: `PowerLineStatus` printed `Online` at the start, before the AddressSanitizer runs and before writing
this report. No compiler change, no git command, no Ember program run (only
`target/redesign3/bin/ember.exe check`). Files written: this report and the files under
`target/redesign3/22-re-attacker/` (section 6). Nothing another agent wrote was edited.

**Result: the fixes do not all hold. 11 new problems: 5 safety problems proven with AddressSanitizer
(G-01 to G-05), and 6 readability problems (G-06 to G-11), each shown from the model's own text (G-08
also with `ember check` and `rustc` output).** Fixes F-05 and F-10 hold; F-08 holds except one message
(G-09); F-11 holds for AN-81 and not for AN-84 (G-11).

## 0. Terms

- **the model**: `RUN3/combined-v4.md`, which agent 21 (the fixer, round 2) wrote; its changes are marked
  **21:** and name the problem they fix (F-01 .. F-12, from `RUN3/20-re-attacker.md`).
- **a fix**: one change 21 made, named by the problem it fixed, as listed in `RUN3/21-fixer.md`. I attacked
  only these, and the rules they lean on where a fix's claim depends on them.
- **AddressSanitizer (ASan)**: a memory checker built into the C compiler. "heap-use-after-free" means the
  program read memory that had already been given back (freed): the error the model must make impossible.
- **view**: a name or value that points into something someone else owns (`ref T`, `Span[T]`,
  `MutSpan[T]`, `StrView`, a plain parameter, a plain loop or pattern name; section 0 of the model).
- **weak count**: the number in an object's header that keeps its memory block (not the object) until the
  last counted `weak` link to it ends. When the object has ended and the count reaches zero, the block is
  freed. Reading a `weak` link tests a flag in that block ("is the object still alive?").
- **deferred-drop region** (area D, AN-62): a loop during which an object whose last **owner** ends is set
  aside whole and freed when the loop ends (section 0: "an object whose count reaches zero is set aside
  whole and freed when the stretch ends").
- **mode 1 / mode 2** in each C program: mode 1 is the C the model, as written, lets through; mode 2 is the
  program the missing check (or the message it would give) leads to, to show that it closes the hole.
- **the owner's test**, **the error-message rule**: as in the brief. **A-8**: area A's rule "no help that
  does not apply".

## 1. What I did

1. Read the brief, `JOB.md`, my duty, `21-fixer.md`, `20-re-attacker.md`, 21's whole diff
   (`target/redesign3/21-fixer/v3_to_v4.diff`), the model's sections 0 to 9, area D's rules D1 to D8, and
   every answer a fix touches (AN-11, 22, 23, 28, 48, 60, 62, 71, 75, 76, 77, 78, 84), and 21's four C
   proofs, on which my C is modelled (the same weak-count, region and handler code).
2. For each fix that changes what is accepted: wrote a NEW program the fixed rule accepts, quoted the rule,
   wrote the C that rule gives, and ran it under ASan (`clang -O1 -g -fsanitize=address -I
   target/redesign/bench`), modes 1 and 2. For the readability side: the owner's test, the error-message
   rule and "one word, one meaning" applied to the fix's own text.
3. Ran `ember check` on two programs and `rustc` on one, for G-08 (section 4).
4. Checked the agreed rows: every agreed spelling appears in v4 at least as often as in v3 (grep counts,
   and 21's `check_v4_output.txt`): rows 3 to 9 are untouched.

## 2. Results, fix by fix

| fix | holds? | new problem |
|---|---|---|
| F-01 (R2: when a copied `weak` link takes its weak count) | no | G-01 (safety) |
| F-02 (R4: what a view put in through `ref mut` may point into) | no | G-02 (safety) |
| F-03 (R4: a `MutSpan` of non-`Copy` elements and another view) | no | G-03 (safety) |
| F-04 (R5's exception for function values, R16, AN-76 rule 3) | no | G-04 (safety), G-06, G-07 |
| F-05 (`r = ref mut first_even(...)`) | yes | |
| F-06 (R2: a name whose value is a view is given its value once) | no | G-08 |
| F-07 (R12: a call through a stored value does not make its holder an argument; AN-60's `for` form restored) | no | G-05 (safety) |
| F-08 (`thread_shared x` only on a `thread_shared` value, E7017) | yes, except one message | G-09 |
| F-09 (a `for` generator writes `from log`; E3060 / E3062) | no | G-02 (its second half) |
| F-10 (E0113 excepts `mut self`, `static mut`) | yes | |
| F-11 (two messages in their programs' names) | AN-81: yes; AN-84: no | G-11 |
| F-12 (one comma rule for every list of names) | no | G-10 |

## 3. New problems

### G-01 (safety, fix F-01): a copy of a `weak` link whose target ended before the loop takes no count, and the region does not keep that block

**The fix.** R2 (21): "The add is left out only where the copy's whole life is covered: by R10's point 1
..., or by a deferred-drop region that ends after the copy does (a local of the loop's body, as AN-62's
`a`; R10's point 2)." A region keeps only objects whose last **owner** ends inside it. An object whose owner
ended **before** the loop is not set aside by anything: its block lives only on weak counts, and when the
last counted link ends inside the loop the block is freed at once. A copy that took no count then tests a
freed block. (AN-62 itself is safe: its `n.parent` is a `set by` link, whose target is alive while it holds
`n`, D7 rule 5.)

**The program** (NEW):

```ember
class Node:
    value: int = 0

class Holder:
    keep: weak Node = None
    on_visit: fn(int)

    fn walk(self) changes keep, on_visit -> int:
        total = 0
        for k in 0..3:
            a = copied self.keep        # R2 (21): a local of the loop's body, inside the region: no count
            self.keep = None            # the last counted link to the node ends
            self.on_visit(k)            # an unknown call: the region (R10 point 2)
            if Some(n) = a:             # tests the header of the node's block
                total += n.value
        return total

fn main():
    h = Holder(on_visit=fn(k: int): pass)
    first = Node(value=5)
    h.keep = weak first                 # a link stored in a field: it takes its weak count
    first = Node(value=6)               # the first node ends; its block stays for `h.keep`'s count
    println(h.walk())
```

**Why the model accepts it.** R10's point 1 does not hold (`self.keep = None` writes a place of type
`weak Node`), so R2's second way applies: the loop makes an unknown call and `Node` has no `drop` (R10
point 2), so there is a region, and `a` is a local of its body, as AN-62's `a`. So `a` takes no count.

**Proof** (`c/g1_weak_copy_dead_target.c`; `g1_weak_copy_dead_target_output.txt`):

```text
--- mode 1
==61192==ERROR: AddressSanitizer: heap-use-after-free on address 0x12bf6efa0768 at pc 0x7ff6ddc83294 bp 0x00ed2514f950 sp 0x00ed2514f998
READ of size 4 at 0x12bf6efa0768 thread T0
    #0 0x7ff6ddc83293 in walk C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g1_weak_copy_dead_target.c:68
    #1 0x7ff6ddc82c0c in main C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g1_weak_copy_dead_target.c:83
freed by thread T0 here:
    #2 0x7ff6ddc8303e in walk C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g1_weak_copy_dead_target.c:66
--- mode 2
mode2 total=0
g1_weak_copy_dead_target ms=0.051 peak_kb=8200 checksum=0
exit=0
```

Line 68 is `if Some(n) = a:` (the header test); line 66 is `self.keep = None` (the last counted link ends,
the block is freed). Mode 2, where `a` takes its count: `total=0` (the node is gone), no report.

**What would close it** (a rule, no word): in a region, the add is left out only for a copy of a link whose
target is known to be alive when the copy is made (a `set by` link, as AN-62's `n.parent` and `p.parent`);
a copy of any other `weak` link takes its count unless R10's point 1 holds for its whole life, its "no
write to a place of type `weak T`" clause included. Cost: Rust's `Weak::clone` where it applies; AN-62's
walk still counts nothing.

### G-02 (safety and error message, fixes F-02 and F-09): a view recorded as pointing "into an object" survives a push to one of its lists

**The fixes.** F-02 (R4): "After a call, the caller's place given to a `ref mut` parameter whose value is a
view ... may point into whatever the call's other view arguments point into ... Inside the callee only such
a view may be put there: another view parameter or a part of one". F-09 (R14): "The position walk's
generator points into the object (R6's default), which a push to `log` does not end, so it sees the list
as it is at each resume." So the model records views at the level of an argument or an object, and says
in so many words that a view "into the object" is not ended by a push to one of the object's lists.

**The program** (NEW): the callee puts in a view of a part of a plain object argument.

```ember
class Bag:
    items: Array[int] = []

struct Best:
    view: Span[int]

fn pick(ref mut best: Span[int], c: Bag):
    if c.items.len() > best.len():
        best = c.items.as_span()        # a part of the view parameter `c` (R4, 21)

fn main():
    a = [1]
    bag = Bag(items=[3, 4, 5])
    h = Best(view=a.as_span())
    pick(h.view, bag)                   # h.view may now point into what `bag` points into (R4, 21)
    for k in 0..64:
        bag.items.push(k)               # a push to one of `bag`'s lists: F-09 says it does not end a view into `bag`
    println(h.view[0])                  # reads the list's old buffer
```

**Why the model accepts it.** `h.view` is a field holding a view, the place F-02 asks for. After the call
it may point into `a` and into `bag`. `bag.items.push(k)` is a push to a list of `bag`, which by F-09's
sentence does not end a view into `bag`. Nothing else ends `h.view`.

**Proof** (`c/g2_refmut_view_into_object.c`; `_output.txt`):

```text
--- mode 1
==46056==ERROR: AddressSanitizer: heap-use-after-free on address 0x12c25daa07c0 at pc 0x7ff76ad72f7d bp 0x00eb9c51f680 sp 0x00eb9c51f6c8
READ of size 8 at 0x12c25daa07c0 thread T0
    #0 0x7ff76ad72f7c in main C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g2_refmut_view_into_object.c:60
freed by thread T0 here:
    #1 0x7ff76ad725cd in realloc S:\compiler-rt\lib\asan\asan_malloc_win_thunk.cpp:101
    #2 0x7ff76ad72ca3 in main C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g2_refmut_view_into_object.c:59
--- mode 2
mode2 h.view[0]=3
g2_refmut_view_into_object ms=0.124 peak_kb=8276 checksum=3
exit=0
```

Line 60 is `println(h.view[0])`; line 59 is `bag.items.push(k)` (the buffer moves). Mode 2 is the program a
refusal at the push would lead to (`h.view` used before it): no report.

**The other reading fails too.** If R8 (b) is read the safe way (a view into an object ends at a reshape of
anything the object owns), then the position-walk generator, which F-09 says points into the object, is
ended by `d.log.push(...)` as well. Then AN-77's E3060 message for a stored generator used after a push
("change it after `g`'s last use, or make `replay` walk by position: `while i < len(self.log):`") and the
E3062 message both offer a way out that is refused on the same line, `g.next()` after the push (the
error-message rule: the line given must compile), and AN-77's sentence "A generator meant to see the list
as it is at each resume ... says so by walking by position" is false for a stored generator.

**What would close it**: the model says which reading R8 (b) has. With the safe reading (the cheap one: no
run-time cost), F-02 keeps its rule, and F-09's two messages drop the "walk by position" way out for a
stored generator used after a change (it gets only "change it after `g`'s last use"), and AN-77's sentence
is limited to a generator consumed before the change. With F-09's reading, R6's default and F-02's rule must
record the part (or "everything the object owns"), not the object.

### G-03 (safety, fix F-03): a `MutSpan` of lists and a view of numbers one of those lists owns pass the overlap test

**The fix.** R4 (21): "A `MutSpan[T]` argument whose `T` is not `Copy` ... counts here as a `ref mut`
argument: it and any other view argument of the call are two different places, proved when compiling (two
different lists) or checked by one overlap test before the call (two compares, as `assert_disjoint`)." A
non-`Copy` element owns memory **outside** the span's range. The other view may point into that memory: the
two ranges never overlap, so "two different lists" holds and the two-compare test passes, yet a write
through the `MutSpan` frees what the other view shows.

**The program** (NEW):

```ember
fn refill(dst: MutSpan[Array[int]], src: Span[int]):
    dst[0] = [9]                  # R5's exception: a plain MutSpan writes; the old first list ends
    println(src[0])               # src views that old list's numbers

class Wrapper:
    lists: Array[Array[int]] = []

fn main():
    w1 = shared Wrapper(lists=[[1, 2, 3]])
    w2 = shared w1                # one more owner (R3): the compiler cannot tell w1 and w2 apart
    refill(w1.lists.as_mut_span(), w2.lists[0].as_span())
```

(In one function, `refill(lists.as_mut_span(), lists[0].as_span())` is also "two different lists", the
outer one and its first element; whether the compiler counts that as R6's "both come from `lists`" is not
said.)

**Why the model accepts it.** Not proved different (two owners of one `Wrapper`), so F-03's overlap test
runs, and passes: the span's range is the outer list's element headers, the other view's range is the inner
list's numbers. Inside `refill` the two parameters are "two different places" by R4's promise, and their
types differ (R8 (b): "unless the compiler knows they differ (different types ...)"), so the write does not
end `src`.

**Proof** (`c/g3_mutspan_nested.c`, the overlap test included; `_output.txt`):

```text
--- mode 1
mode1 overlap test passed
==123544==ERROR: AddressSanitizer: heap-use-after-free on address 0x121d497a07c0 at pc 0x7ff6bdaa301e bp 0x0077b4eff7f0 sp 0x0077b4eff838
READ of size 8 at 0x121d497a07c0 thread T0
    #0 0x7ff6bdaa301d in refill C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g3_mutspan_nested.c:44
    #1 0x7ff6bdaa2c81 in main C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g3_mutspan_nested.c:64
freed by thread T0 here:
    #2 0x7ff6bdaa2f43 in refill C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g3_mutspan_nested.c:43
--- mode 2
mode2 overlap test passed
mode2 src[0]=1
g3_mutspan_nested ms=0.074 peak_kb=8220 checksum=1
exit=0
```

Line 44 is `println(src[0])`; line 43 is `dst[0] = [9]`. Mode 2 is the program the refusal's line leads to
(a copy of the inner list given as `src`): no report.

**What would close it**: for a non-`Copy` `T`, no run-time test of two compares can decide it; the call is
refused when compiling unless the compiler proves the other view cannot point into memory an element of
type `T` owns (for example, its element type cannot be reached from `T`), with F-03's message and its copy
line. This is Rust's rule (compile time only). Numbers (`Copy` `T`) keep AN-11's and AN-14's meaning and
cost.

### G-04 (safety, fix F-04): a function that calls the function value it is given is not "a call that may run a function value"

**The fix.** R5 (21) (b): "a call that may run a function value (a call through one, an unknown call, or a
call of a function whose first line lets it change a place that holds function values: `changes handlers`,
`mut self`, a `ref mut` parameter holding them, R12) is taken to change every variable that a function
value of the program changes". AN-76 rule 3 (21) gives the same list. `run(log_click)` below is none of
the three: it is not a call through a function value; `run` is a known function (if a call of a known
function that runs a function value were already an "unknown call", 21's third item, written for
`b.click()`, would not be needed); and `run`'s first line changes nothing. D3's own sentence ("every call
that may run that function value") is wider, so the model now says two things; R5 and rule 3 are the ones
the loop check reads.

**The program** (NEW):

```ember
static mut LOG: Array[String] = []

fn run(f: fn()):
    f()

fn main() changes LOG:                  # R5 (a): main writes the lambda that pushes to LOG
    for k in 0..4:
        LOG.push(String("start"))
    log_click = fn(): LOG.push(String("clicked"))
    for entry in LOG:
        run(log_click)                  # not in R5 (b)'s list
        println(entry)
```

**Proof** (`c/g4_fnvalue_param_loop.c`; `_output.txt`):

```text
--- mode 1
==93868==ERROR: AddressSanitizer: heap-use-after-free on address 0x116a84ba0448 at pc 0x7ff7a32a2fc6 bp 0x0005a093fc50 sp 0x0005a093fc98
READ of size 8 at 0x116a84ba0448 thread T0
    #0 0x7ff7a32a2fc5 in print_entry C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g4_fnvalue_param_loop.c:43
    #1 0x7ff7a32a2bcb in main C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g4_fnvalue_param_loop.c:55
freed by thread T0 here:
    #1 0x7ff7a32a25cd in realloc S:\compiler-rt\lib\asan\asan_malloc_win_thunk.cpp:101
    #2 0x7ff7a32a2e64 in log_clicked C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g4_fnvalue_param_loop.c:39
    #3 0x7ff7a32a2bec in main C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g4_fnvalue_param_loop.c:54
--- mode 2
mode2 log=5
g4_fnvalue_param_loop ms=0.054 peak_kb=8188 checksum=2232
exit=0
```

Line 55 is `println(entry)`, line 54 `run(log_click)`, line 39 the lambda's push. Mode 2 (the call after
the loop, as rule 3's message says): no report. The same holds for any library call that runs a function it
is given (`inspect`, `retain`, a sort with a comparison function).

**What would close it**: R5 (b) and AN-76 rule 3 say D3's sentence instead of a list: a call may run a
function value when the callee's inferred summary (R16) says it may call a function value it is given or
reaches. No run-time cost.

### G-05 (safety, fix F-07): AN-60's restored `for` walk lets a handler change the list through its own capture

**The fix.** R12 (21): "R8 (c) sees the list changed only if an argument may change it ... a `for ref mut h
in self.handlers:` walk may give each handler a read-only holder (`h(self)` with `handlers:
Array[fn(Ui)]`), since no handler can then change the list the loop walks (AN-60, point 6)". A handler does
not need its argument to reach the Ui: AN-60 itself gives the spelling, "A handler that must not keep the UI
alive captures it weakly: `... fn() with weak ui: if Some(ref mut me) = ui: me.clicks += 1`", and D6 says
"`Some(ref mut p)` binds a view that may change the target". Its change is not an argument's (R12, 21), and
not a "variable" (R5 (b), 21; D3 counts "the real variables it captured and its own named state").

**The program** (NEW):

```ember
class Ui:
    handlers: Array[fn(Ui)] = []
    clicks: int = 0

    fn fire(self) changes handlers:             # AN-60 point 6, as 21 restored it
        for ref mut h in self.handlers:
            h(self)

fn noop(u: Ui):
    pass

fn main():
    ui = Ui()
    ui.handlers.push(noop)
    ui.handlers.push(fn(u: Ui) with weak ui:    # AN-60's own spelling for a handler that reaches its Ui
        if Some(ref mut me) = ui:
            for k in 0..8:
                me.handlers.push(noop))         # changes the list `fire` walks
    ui.handlers.push(noop)
    ui.fire()
```

**Proof** (`c/g5_handler_weak_ui_for.c`; `_output.txt`):

```text
--- mode 1
==73768==ERROR: AddressSanitizer: heap-use-after-free on address 0x1274598a0460 at pc 0x7ff626b8342a bp 0x0069b8b8f840 sp 0x0069b8b8f888
READ of size 8 at 0x1274598a0460 thread T0
    #0 0x7ff626b83429 in fire_for C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g5_handler_weak_ui_for.c:56
    #1 0x7ff626b82dde in main C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g5_handler_weak_ui_for.c:70
freed by thread T0 here:
    #1 0x7ff626b825cd in realloc S:\compiler-rt\lib\asan\asan_malloc_win_thunk.cpp:101
    #2 0x7ff626b8328d in adder C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g5_handler_weak_ui_for.c:51
    #3 0x7ff626b83402 in fire_for C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\c\g5_handler_weak_ui_for.c:56
--- mode 2
mode2 clicks=10 handlers=11
g5_handler_weak_ui_for ms=0.057 peak_kb=8216 checksum=10
exit=0
```

Line 56 is the `for` walk (it reads the next handler from the list's old buffer); line 51 is the handler's
`me.handlers.push(noop)`. Mode 2 is AN-60's position loop (`while self.current < len(self.handlers):`), which
reads the buffer again at each step: no report. The same with `with shared ui` (AN-60 point 6 warns of the
cycle but accepts it).

**What would close it**: either R8 (c) counts a call through a function value as possibly changing every
list of a type that its captured links (`weak`, `shared`, a box) can reach, which refuses this walk; or the
`for` form is withdrawn again and the position loop stays the one form for handlers that get the Ui (19's
removal, now with a reason that holds). Neither needs a word.

### G-06 (readability: one word, two meanings, fix F-04 (a)): `changes LOG` is required where nothing changes `LOG`, and absent where it changes

R5 (21) (a): "a lambda's body is part of the function that writes it: that function's first line names a
module variable the lambda changes ..., or E2105: ... name it on `setup`'s first line: `fn setup(ref mut b:
Button) changes LOG:`". R5 (19): "A function that calls one whose first line says `changes X` changes X too,
and its own first line names it". Together:

```ember
fn setup(ref mut b: Button) changes LOG:        # (a): it writes a lambda that pushes to LOG
    b.on_click(fn(): LOG.push(String("clicked")))

fn make_ui() changes LOG -> Button:             # R5 (19): it calls setup, whose first line says changes LOG
    b = Button()
    setup(b)
    return b

fn press(ref mut b: Button):                    # runs the handler, so changes LOG; no word (R5's exception)
    b.click()
```

A call of `setup` or `make_ui` never changes `LOG`; every call of `press` does. The owner's test ("a keyword
says exactly what it does, and does the same for every type"): on `setup` and `make_ui`, `changes LOG` now
means "writes, or calls something that writes, a function value that may change `LOG` later", and on `tick`
it means "a call changes `LOG`". AN-76 keeps refusing the second kind of false line ("`tick` says `changes
LOG` and never changes it; remove `LOG` from its first line") while (a) demands it on `setup`. Rule (a) adds
nothing to safety: (b) and D3 carry the change. **What would close it**: drop (a) (the exception, stated in R5,
then covers the lambda's writer as it covers the caller), or the owner's type word of section 9 point 9
(`fn() changes LOG`).

### G-07 (readability: error message, fix F-04 (b)): the refusal names a handler the button does not hold

Rule (b) takes `b.click()` to change "every variable that a function value **of the program** changes". So:

```ember
fn main() changes LOG:
    b = Button()
    c = Button()
    c.on_click(fn(): LOG.push(String("clicked")))   # line 4: a handler of `c`
    for entry in LOG:
        b.click()                                    # `b` holds no handler
        println(entry)
```

is refused, and AN-76's 21 message would say "`b.click()` (line 6) runs the handlers in `b.handlers` ...,
one of which, made on line 4, pushes to `LOG`": false (line 4's handler is `c`'s). The error-message rule
asks for the reason in one line; this one is wrong, and whether the loop compiles is decided by a lambda
written for another object, possibly in another function or package, which no word in `main`'s loop or in
`Button` says (JOB: "the compiler does not silently decide what the programmer did not say"). **What would
close it**: the message says what the rule knows ("a handler made on line 4 pushes to `LOG`, and the
compiler cannot tell which handlers `b` holds; click after the loop"), and the conservative breadth is
written in R5 as a stated limitation.

### G-08 (readability and Python/Rust parity, fix F-06): the most common parse loop is refused, the message's line does not work in a loop, and a struct around the view escapes the rule

R2 (21): "A name whose value is a view is given its value once, wherever the view came from ... `=` on it
later is refused the same way (E3023): "`n` reads `p.name` for its whole life; for `q`'s name use a new
name: `m = q.name.trim()`"". The program (today's spelling, `today/t1_repoint_view_loop.em`):

```ember
fn count_fields(line: str) -> int:
    n = 1
    rest = line.trim()
    while true:
        match rest.split_once(","):
            Some((head, tail)):
                n += 1
                rest = tail.trim()          # NEW: E3023 under F-06
            None:
                return n
    return n
```

Today's Ember accepts it (`today/check_output.txt`):

```text
=== t1_repoint_view_loop.em
warning[L1001]: `head` is never read
 --> t1_repoint_view_loop.em:7:19
  |
7 |             Some((head, tail)):
  |                   ^^^^ declared here and never used
  |
  = help: remove it, or rename it `_head` if that is deliberate

exit=0
=== t2_repoint_view_simple.em
exit=0
```

and so does safe Rust (`today/r1_repoint_view_loop.rs`, `let mut rest = line.trim(); ... rest =
tail.trim();`): `rustc exit=0`, the program prints `3`. Under F-06 the NEW model refuses `rest =
tail.trim()` (19 left this case accepted: `rest` was given a call's result). Three failures:
- **The message's line does not apply** (A-8): "use a new name" cannot carry a view from one round to the
  next. What works is a position (an index into `line`), a copy per round (`String(...)`, an allocation per
  field: slower than C and Rust), or a struct around the view (below); the message gives none of them.
- **Not one rule.** R2 (21) replaces by `=` "a struct or a generator a call makes even when it holds views",
  while R7 says "A struct that holds a view is itself a view" and F-09 says "`g = d.replay()` is a view of
  `d.log`". So `cur = Cursor(rest=line.trim())` then `cur = Cursor(rest=tail.trim())` is accepted with the
  same meaning as the refused `rest = tail.trim()`, and `g` is a view that `=` replaces. 21's stated result,
  "whether a later `=` compiles is read from what the name holds, a view or a value of its own", is not what
  the rule does: it is read from the type's name (`Span`, `StrView`, `MutSpan` against everything else).
  Since the wrapper is safe, the refusal of the bare view protects nothing.
- **Annoying where Rust is not**: the brief's three answers do not include "refuse what Rust and today
  accept".

**What would close it**: one rule for every value that holds views (bare or in a struct or generator):
either all are replaced by `=` (with R8 tracking the new targets, as for `cur`), which needs the owner to
revisit 19's R18-03 ruling that `=` on a view name writes into its place (section 9 point 11); or none are.
Meanwhile the message for a name given a value outside a loop and again inside it gives the position form.

### G-09 (readability: error message, fix F-08): AN-78's new message leads to a refused program

AN-78 (21): "`with shared game` under `threads(any)`: `error[E7015]: ... create the game shared between
threads, \`game = thread_shared Game()\`, take it as \`game: thread_shared Game\` in \`setup\`, and write
\`with thread_shared game\``". The callback is `game.add_point()`, and `fn add_point(self) changes score:
self.score += 1` with `score: i32 = 0`. R9: "A `thread_shared` value is never changed directly by any owner:
only through `Atomic`, `Mutex`, `RwLock`, a channel, or a `guarded by` field under its lock". So the program
the message leads to is refused at `game.add_point()` (E7014, "a `thread_shared` value changed directly").
**What would close it**: the message adds the field's line: "`score: Atomic[i32]` and
`self.score.fetch_add(1)` in `add_point`" (or `score: i32 guarded by lock` and `with self.lock:`), as AN-71's
E7014 message does for `names`.

### G-10 (readability: one comma shape, two rules, fix F-12): untyped lambda parameters

R3 (21): "A parameter list is not such a list: each parameter is written whole, with its own type (`fn
collide(ref mut a: Body, b: Body):`)." Lambda parameters need no type: the spec writes `retain(fn(p) =>
p.lifetime > 0.0)` (spec 289) and `apply_twice(fn(x) => x + step, 1)` (spec 2053), and R4 itself writes
"`fn(ref mut ui)`". So `fn(ref mut ui, dx) => ui.x += dx` is the comma shape F-12 refuses
(`for e, ref mut pos, vel in q:`, "it can be read as covered by that word") with no type to tell the reader
otherwise, and F-12's reason for leaving parameter lists out does not hold for it. Owner's test on `dx`: no
word, no type; is it `ref mut`? **What would close it**: R3 says that a lambda parameter list written
without types follows the comma rule (`fn(ref mut ui, ref dx)`, or types on both).

### G-11 (readability: error message, fix F-11): AN-84's help, applied as given, gives a refused program

AN-84 (21): the refused `place` is `i = 0`, `@parallel`, `while i < idx.len():`, `xs[i] = idx[i]`, `i += 1`,
and the help is "count over the range: `for i in 0..idx.len():`". Written in place of line 4, as the help
says, the program keeps `i = 0` and `i += 1`; `i += 1` then changes the loop name `i`, which R5 refuses ("A
plain name never changes anything: ... a plain loop or pattern name", E3023). The accepted `place` has
neither line. **What would close it**: "count over the range instead of `i = 0`, the `while` line and `i +=
1`: `for i in 0..idx.len():`".

## 4. The fixes that hold, and how each was checked

- **F-05**: `r = ref mut first_even(a.as_mut_span())` passes the owner's test (`ref mut` says what `r` is and
  that it changes it), `=` keeps one meaning, and no NEW block binds a `ref mut` call plainly (grep: the
  plain bindings at lines 2799, 2900, 4523, 4645, 4648 and 8285 of v4 are all OLD or means-exactly code,
  each with 21's note). Plain `r = first_even(...)` then `r = 7` copies, as `n = counts[0]` does: Python's
  meaning, written in R2.
- **F-08**, apart from G-09: R3's `thread_shared x` line, E7017's one meaning, AN-71's three messages (the
  thread only reads `names`, so `names = thread_shared Names()` compiles there) and AN-75's code.
- **F-10**: E0113 no longer covers AN-70's `virtual fn on_update(mut self, dt: float):` or `static mut`.
- **F-11** for AN-81: the help keeps only `if i != j:`, which applies to `step`.
- **F-09**'s codes: E3060 for a stored generator used after a change, E3062 for a `for` walk without `from`,
  E3020 for a loop that consumes it: one mistake, one code (its reading of R8 (b) is G-02).
- Agreed rows 3 to 9: every agreed spelling at least as often in v4 as in v3 (`fn note(...) changes log:`
  3/3, `return ref g[0]` 3/3, `word = line.to_lower().trim()` 10/11, `fn() with copied score:` 4/4,
  `fn load_assets(...)` 3/3, `State.Walking(ref mut path):` 8/8, `snapshot = copied items` 4/4); none of
  their text was edited.

## 5. What the closings would cost

All are compile-time rules or message text, except G-01's (a weak-count add and subtract on copies of plain
`weak` links inside a region, Rust's `Weak::clone`; none in AN-62) and G-03's (none: the run-time test is
replaced by a compile-time refusal). None needs a new word; G-06 and G-08 have a better form that needs
the owner (section 9 points 9 and 11).

## 6. Files

All under `C:\Users\ism19\Code\ember\target\redesign3\22-re-attacker\`:
- `c\g1_weak_copy_dead_target.c`, `c\g2_refmut_view_into_object.c`, `c\g3_mutspan_nested.c`,
  `c\g4_fnvalue_param_loop.c`, `c\g5_handler_weak_ui_for.c`, each with its `.exe` and `_output.txt` (the full
  ASan reports, modes 1 and 2; `.pdb` files deleted to keep the disk footprint small).
- `today\t1_repoint_view_loop.em`, `today\t2_repoint_view_simple.em` and `today\check_output.txt` (their
  `ember check` output, both `exit=0`); `today\r1_repoint_view_loop.rs` and its `.exe` (safe Rust, accepted,
  prints `3`); `today\target\` is `ember check`'s own cache.
