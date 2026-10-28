# 20 fix re-attacker: report (run 3, phase 3, round 1)

Power: `PowerLineStatus` printed `Online` at the start, before the AddressSanitizer runs and before writing
this report. No compiler change, no git command, no Ember program run (only
`target/redesign3/bin/ember.exe check`). Files written: this report and the files under
`target/redesign3/20-re-attacker/` (section 5). Nothing another agent wrote was edited.

**Result: the fixes do not all hold. 12 new problems: 4 safety problems proven with AddressSanitizer
(F-01 to F-04; F-04 is also a readability problem), and 8 readability problems (F-05 to F-12), each
shown from the model's own text.** Fixes R18-06, 07, 08, 17, 18, 20, 21 and the VOC table hold.

## 0. Terms

- **the model**: `RUN3/combined-v3.md`, which agent 19 (the fixer) wrote; its changes are marked **19:**.
- **a fix**: one change 19 made, named by the problem it fixed (R18-01 .. R18-22, VOC), as listed in
  `RUN3/19-fixer.md`. I attacked only these.
- **AddressSanitizer (ASan)**: a memory checker built into the C compiler. "heap-use-after-free" means the
  program read memory that had already been given back (freed): the memory error the model must make
  impossible.
- **view**: a name or value that points into something someone else owns (section 0 of the model).
  **own name**: a name holding a value of its own. Since 19's R2, a name given the result of a call is an
  own name even when its value is a view.
- **weak link**: a link that does not keep its target alive; reading it tests the target's header.
  **weak count**: the number in the target's header that keeps its memory block (not the object) until
  the last weak link to it ends; when it is zero and the object has ended, the block is freed.
- **deferred-drop region** (area D, AN-62): a loop during which an object whose last owner ends is set
  aside whole and only freed when the loop ends.
- **mode 1 / mode 2** in each C program: mode 1 is the C the model, as written, lets through; mode 2 is
  the same program with the check the model is missing, to show that the check closes the hole.
- **the owner's test**, **the error-message rule**: as in the brief.

## 1. What I did

1. Read the brief, `JOB.md`, my duty, `19-fixer.md`, the whole diff 19 made
   (`target/redesign3/19-fixer/v2_to_v3.diff`), the model's sections 0 to 9, and the answers each fix
   touches (AN-11, 14, 22, 23, 53, 60, 62, 70, 71, 75, 76, 77, 80, 84), area A's rules A-1 to A-8 and area
   D's rules D1 to D8, and area D's C (`11-area-D/c/an62_defer_asan.c`, `an63_links.c`), on which my C is
   modelled.
2. Agents 16 and 17 stopped early, so the only earlier safety proof a fix touched is 19's AN-62 re-run;
   I attacked the safety of every rule a fix wrote (R2, R4, R5, R12, R14, R16).
3. For each attack that could free memory: wrote the NEW program, quoted the rules that accept it, wrote
   the C those rules give, and ran it under ASan (`clang -O1 -g -fsanitize=address -I
   target/redesign/bench`). For the readability attacks: applied the owner's test, the error-message
   rule and the "one word, one meaning" rule to the fix's own text.
4. Ran `ember check` on four programs to show what today's Ember does where a finding compares with it
   (`today/check_output.txt`), and re-ran it on 19's four files: the exit codes 19 pasted are right
   (`m59_owned_once.em` 0, `m70_virtual_changes.em` 0, `o20_plain_int_param.em` 1,
   `m20_int_param_copy.em` 0).
5. Scanned every NEW block (81 blocks, 80 answers) for a name bound to a part, a loop name or a pattern
   name that is later given a value with `=` (`scan_rebind.py`), which 19's R2 now refuses: none (one
   false hit, AN-21's `r = 0.0` then `r = max(...)`, a number). So R18-03 broke no NEW block.

## 2. Results, fix by fix

| fix | holds? | new problem |
|---|---|---|
| R18-01 (R4 `ref mut` on links and views; R5's one exception) | no | F-02, F-03 (safety) |
| R18-02 (R12: a call through a stored function value changes its holder) | no | F-07 |
| R18-03 (R2: no view name re-pointed; a call's result is an own name) | no | F-05, F-06 |
| R18-04 (R5: callers name what their calls change; R16) | no | F-04 (safety and readability) |
| R18-05 (E7016's meaning and message) | no | F-11 |
| R18-06 (`owned f: once fn()`) | yes | |
| R18-07 (AN-43's `owned n: shared Node`) | yes | |
| R18-08 (one code and text for `owned fn`) | yes | |
| R18-09 (section 8, one code per mistake) | no | F-09 (second part), F-10 |
| R18-10 (stored generators keep the loop limitation) | no | F-09 |
| R18-11 (AN-70's virtual is `mut self`) | yes, but see F-10 | F-10 |
| R18-12 (AN-71 `names = thread_shared Names()`) | no | F-08 |
| R18-13 (R2: a `weak` link is copied with `copied`) | no | F-01 (safety) |
| R18-15 (R3: one comma rule) | no | F-12 |
| R18-17 (AN-05's order stated) | yes | |
| R18-18 (AN-35's E3013 message) | yes | |
| R18-19 (messages name the program's names) | no | F-11 (AN-81's help) |
| R18-20 (a plain parameter is never changed) | yes | |
| R18-21 (AN-34's two forms in two functions) | yes | |
| VOC (section 1.4) | yes (spot-checked spec lines 1538, 4154, 3520, 3506, 2536, 4238, 4310, 3692: each names what 1.4 says) | |
| R18-14, R18-16, R18-22 | left open by 19 (needs the owner); not attacked | |

## 3. New problems

### F-01 (safety, fix R18-13): a copied `weak` link kept after the deferred-drop region has no weak count; reading it reads freed memory

**The fix.** R2: "A copied link is one pointer plus, as Rust's `Weak::clone`, one add to the target's weak
count ..., left out by R10's points 1 and 2 wherever nothing in the copy's life can end the target's last
owner or a deferred-drop region covers it (AN-62's walk counts nothing)." AN-62's 19 note: "the walk is
inside the deferred-drop region, so the copied link takes no weak count". So the add is left out for a
copy **made** inside the region. Nothing says the region must cover the copy's **whole life**, and the
region only keeps what it set aside until the loop ends (section 0: "freed when the stretch ends"). A
copy is an own value: it can be stored in a field and outlive the loop.

**The program** (NEW): AN-62's own walk with one added field and one added line.

```ember
class Node:
    value: int = 0
    parent: weak Node set by children
    children: Array[Node] = []

class Tree:
    root: Node
    leaves: Array[weak Node] = []
    on_visit: fn(Node)
    top: weak Node = None                       # the highest node the last walk reached

    fn depth_total(self) changes on_visit, top -> int:
        depth = 0
        for leaf in self.leaves:
            if Some(n) = leaf:
                a = copied n.parent
                while Some(p) = a:
                    self.on_visit(p)
                    depth += 1
                    self.top = copied a         # the added line: a link of the tree's own
                    a = copied p.parent
        return depth

fn main():
    tree = Tree(root=Node(value=1), on_visit=fn(n: Node): pass)
    mid = Node(value=2)
    leaf = Node(value=3)
    tree.leaves.push(weak leaf)
    mid.children.push(leaf)
    tree.root.children.push(mid)
    println(tree.depth_total())                 # 2; tree.top links the first root
    tree.root = Node(value=9)                   # the first root ends
    if Some(t) = tree.top: println(t.value) else: println("gone")
```

**Why the model accepts it.** Every line is AN-62's, or R2's `copied` link, or a store into a field the
first line names (`changes ... top`, R5). The region is AN-62's (R10 point 2: the loop makes an unknown
call and `Node` has no `drop`). The copy is made inside it, so by R2 as 19 applied it, it takes no count.
`set by` links take no count either (D7 rule 5), so nothing keeps the first root's block once it ends.

**Proof** (`target/redesign3/20-re-attacker/c/f1_weak_copy_escape.c`, area D's weak-count and region
model; `f1_weak_copy_escape_output.txt`):

```text
--- mode 1
==115348==ERROR: AddressSanitizer: heap-use-after-free on address 0x12160b1a0448 at pc 0x7ff78bc73538 bp 0x00bb101af780 sp 0x00bb101af7c8
READ of size 4 at 0x12160b1a0448 thread T0
    #0 0x7ff78bc73537 in main C:\Users\ism19\Code\ember\target\redesign3\20-re-attacker\c\f1_weak_copy_escape.c:113
freed by thread T0 here:
    #2 0x7ff78bc73184 in main C:\Users\ism19\Code\ember\target\redesign3\20-re-attacker\c\f1_weak_copy_escape.c:112
--- mode 2
mode2 depth=2
top=gone
f1_weak_copy_escape ms=0.119 peak_kb=8232 checksum=2
```

Line 113 is `if Some(t) = tree.top:` (the header test); line 112 is `tree.root = Node(value=9)`. Mode 2,
where the copy stored in a field takes its count: `top=gone`, no report.

**What would close it** (a rule, no word): the add is left out only while the copy's whole life is inside
the region or point 1 holds for its whole life (a local that ends in the loop, as AN-62's `a`); a copy
stored in a field or a list, returned, or used after the loop takes its count, as Rust's `Weak::clone`.

### F-02 (safety, fix R18-01): `ref mut` on a view parameter lets the callee point the caller's view into another argument, and no rule tells the caller

**The fix.** R4: "`ref mut x` names the caller's place itself ... For a parameter whose value is a link or
a view (`shared T`, `weak T`, `MutSpan[T]`, `Span[T]`, `StrView`), that place holds the link or the view,
so ... `pos = other` points the caller's span at `other`." Nothing says what the caller's view points
into after such a call: R6 ("Results that point into something") is about results only.

**The program** (NEW):

```ember
fn pick_longer(ref mut best: Span[int], other: Span[int]):
    if other.len() > best.len():
        best = other                    # R4 (19): `other` goes into the caller's place

fn main():
    a = [1, 2]
    best = a.as_span()                  # R2 (19): a call's result, an own name; it views `a` (R6)
    b = [3, 4, 5]
    pick_longer(best, b.as_span())      # the call writes no word (R4); `best` now views `b`
    b = [6]                             # R2 (19): `=` replaces an own name's value; the old list ends
    println(best[0])                    # reads the old list of `b`
```

**Why the model accepts it.** R4's alias rule is met: `best` and `b.as_span()` view two different lists.
After the call the checker still takes `best` to view `a` (no rule moves it). `b = [6]` ends `b`'s old
list, which nothing known to the checker views. `best[0]` reads a view of `a`, which is alive.

**Proof** (`c/f2_refmut_view_repoint.c`, `f2_refmut_view_repoint_output.txt`):

```text
--- mode 1
==101692==ERROR: AddressSanitizer: heap-use-after-free on address 0x111f4f2a0760 at pc 0x7ff768c22cda bp 0x000d428ff660 sp 0x000d428ff6a8
READ of size 8 at 0x111f4f2a0760 thread T0
    #0 0x7ff768c22cd9 in main C:\Users\ism19\Code\ember\target\redesign3\20-re-attacker\c\f2_refmut_view_repoint.c:51
freed by thread T0 here:
    #2 0x7ff768c22b3e in main C:\Users\ism19\Code\ember\target\redesign3\20-re-attacker\c\f2_refmut_view_repoint.c:50
--- mode 2
mode2 best[0]=3
f2_refmut_view_repoint ms=0.049 peak_kb=8180 checksum=3
```

Line 51 is `println(best[0])`, line 50 is `b = [6]`. Mode 2 is the program a refusal would lead to (`best`
used before `b = [6]`): no report.

**What would close it**: R6's default extended to a `ref mut` parameter whose value is a view: after the
call, the caller's view may point into what the call's other view arguments point into (Rust's
`&mut &'a [T]` with `other: &'a [T]`), or narrowed by `from other` on the parameter (R6's existing word);
inside the callee, only such a view may be put into it (a view of the callee's own locals refused, E3063).
`shared T` and `weak T` parameters are safe as they are (a re-point there is counted).

### F-03 (safety, fix R18-01): R5's one exception lets a plain `MutSpan` overwrite an element another span parameter of the same call is viewing

**The fix.** R5: "a plain name may change what a `MutSpan` ... leads to ... a `MutSpan`'s elements
(`pos[i] = ...`; the type's name says `Mut`)", for every element type. R4's promise is written for
`ref mut` only: "Two `ref mut` arguments of one call, or a `ref mut` argument and any other view argument,
are two different places". A plain `MutSpan` parameter is not `ref mut`. R8 (b) counts two paths as "may
reach the same object" only when "at least one goes through a `shared` or `weak` link or a kept view";
two span parameters are neither. AN-14 accepts two spans of one list (its `blend_any` handles the overlap
in its `Err` arm, and its cost line measures "two spans of one list ... without" noalias).

**The program** (NEW):

```ember
fn refill(dst: MutSpan[String], src: Span[String]):
    w = src[0].trim()            # a view of the text of `src`'s first string
    dst[0] = String("new")       # R5's exception; the old first string ends
    println(w)                   # reads the old text

fn main():
    names = [String("  ann  "), String("bob")]
    refill(names.as_mut_span(), names.as_span())
```

**Why the model accepts it**: as quoted: no compare at the call (no `ref mut`), and inside `refill` the
write through `dst` does not end `w` (no "may reach"). Today's Ember refuses the call
(`today/t3_overlap_spans.em`): `error[E3021]: \`names\` is borrowed here and mutably borrowed elsewhere`,
`exit=1`, so here the model is weaker than today.

**Proof** (`c/f3_mutspan_overlap.c`, `f3_mutspan_overlap_output.txt`):

```text
--- mode 1
==109716==ERROR: AddressSanitizer: heap-use-after-free on address 0x116ed09a0132 at pc 0x7ff7a4fc2f66 bp 0x002b4afcf640 sp 0x002b4afcf688
READ of size 1 at 0x116ed09a0132 thread T0
    #0 0x7ff7a4fc2f65 in refill C:\Users\ism19\Code\ember\target\redesign3\20-re-attacker\c\f3_mutspan_overlap.c:48
    #1 0x7ff7a4fc2b5b in main C:\Users\ism19\Code\ember\target\redesign3\20-re-attacker\c\f3_mutspan_overlap.c:57
freed by thread T0 here:
    #2 0x7ff7a4fc2e9f in refill C:\Users\ism19\Code\ember\target\redesign3\20-re-attacker\c\f3_mutspan_overlap.c:46
--- mode 2
panic: `refill(names.as_mut_span(), names.as_span())`: `dst` and `src` overlap
exit=101
```

Line 48 reads `w`; line 46 is `dst[0] = String("new")`. Mode 2 applies R4's one compare to the `MutSpan`
argument: the program stops before anything is freed.

**What would close it**: a `MutSpan` argument counts as a `ref mut` argument for R4's "two different
places" (proved, or one compare) whenever its element type owns memory (`String`, `Array`, a class); for
plain numbers an overlap frees nothing, so AN-11's and AN-14's float programs keep their meaning and cost.

### F-04 (safety and readability, fix R18-04): a function value that changes a `static mut` is on no first line, so R5's new promise is false, and R16's new sentence removes the summary AN-76's loop rule needs

**The fix.** R5: "So a reader of any first line sees everything a call of it may change." R16: "what a
call changes in a `static mut` is now written on every first line that makes the change (R5), so of E's
summaries only those of scoped tasks stay inferred." AN-76 rule 3 (unchanged): "a loop over `LOG` ...
across a call whose summary changes `LOG` is refused". R5 asks a caller to name only what "one whose
first line says `changes X`" changes. A lambda has no first line with `changes`, and `click`'s first line
(19's own AN-53 fix) is `changes handlers`. D3 (unchanged) says the opposite of R16: "A variable that a
living function value changes is treated as changed by every call that may run that function value",
which is an inferred summary.

**The program** (NEW):

```ember
static mut LOG: Array[String] = []

class Button:
    handlers: Array[fn()] = []
    fn on_click(self, owned f: fn()) changes handlers:
        self.handlers.push(f)
    fn click(self) changes handlers:
        for ref mut h in self.handlers:
            h()

fn main() changes LOG:
    for k in 0..4:
        LOG.push(String("start"))
    b = Button()
    b.on_click(fn(): LOG.push(String("clicked")))   # the module's LOG
    for entry in LOG:
        b.click()                                    # its first line: `changes handlers`
        println(entry)
```

**Either reading fails a test.** (a) R16 as written: rule 3 sees `b.click()`'s written summary, which
does not name `LOG`, so the loop is accepted and `entry` reads the buffer `LOG.push` moved:

```text
--- mode 1
==21104==ERROR: AddressSanitizer: heap-use-after-free on address 0x124e321a0448 at pc 0x7ff6ffbf30f6 bp 0x00a81beff9f0 sp 0x00a81beffa38
READ of size 8 at 0x124e321a0448 thread T0
    #0 0x7ff6ffbf30f5 in print_entry C:\Users\ism19\Code\ember\target\redesign3\20-re-attacker\c\f4_fnvalue_static_loop.c:54
    #1 0x7ff6ffbf2c5b in main C:\Users\ism19\Code\ember\target\redesign3\20-re-attacker\c\f4_fnvalue_static_loop.c:67
freed by thread T0 here:
    #1 0x7ff6ffbf25cd in realloc S:\compiler-rt\lib\asan\asan_malloc_win_thunk.cpp:101
    #2 0x7ff6ffbf2f14 in log_clicked C:\Users\ism19\Code\ember\target\redesign3\20-re-attacker\c\f4_fnvalue_static_loop.c:50
    #3 0x7ff6ffbf3029 in click C:\Users\ism19\Code\ember\target\redesign3\20-re-attacker\c\f4_fnvalue_static_loop.c:48
    #4 0x7ff6ffbf2c7d in main C:\Users\ism19\Code\ember\target\redesign3\20-re-attacker\c\f4_fnvalue_static_loop.c:66
--- mode 2
mode2 log=5
f4_fnvalue_static_loop ms=0.060 peak_kb=8212 checksum=2232
```

(`c/f4_fnvalue_static_loop.c`; line 67 is `println(entry)`, 66 is `b.click()`; mode 2 is D3's reading,
refused, with the click moved after the loop as rule 3's message says: no report.)
(b) D3 holds: the loop is refused, but then a change of `LOG` reaches callers through `b.click()` while no
first line says it (for example `fn press(ref mut b: Button): b.click()` changes `LOG` and its first line
says nothing), so R5's new sentence and AN-76 rule 2's "each link of which now says it" are false: the
owner's test fails on `press`. The model also does not say whether the lambda itself may change `LOG`
(AN-76 rule 2 asks a first line of every function that does, and a lambda has none), nor give a message.

**What would close it**: R16 keeps D3's inferred summary for calls through function values and says so,
and R5's sentence is narrowed to "every call not made through a function value"; the lambda case gets a
rule and a message. A type for function values that says what they change would need a word (the owner).

### F-05 (readability: meaning, fix R18-03): `r = first_even(...)` is now an own name, so `r = 7` no longer writes the element it names

**The fix.** R2: "A name given the result of a call (...) ... is an own name: its value may itself be a
view ..., and `=` replaces that value, as for any own name."

**Against the model's own text.** R6: "A call whose result type is `ref mut T` is a place". AN-28: after
`r = ref mut a`, `r = 20` writes `a`. The model's "means exactly" programs bind such calls and write
through the name: AN-11's `m11_mutspan.em` (`r = first_even(a.as_mut_span())`, `r = 7`), AN-22's
`m22b_pool_place.em` (`p = reg.get_mut(0, 0)`, `p += 1`), AN-23's `m23_bound_first.em` (`f = first(xs)`,
`f += 1`). Today `r = 7` there is a write through `r`: `today/t1_ref_result_write.em` (a push to `a`
between the two lines):

```text
error[E3022]: `a` is already mutably borrowed
  --> today/t1_ref_result_write.em:12:5
   |
11 |     r = first_even(a.as_mut_span())
   |                    - mutable borrow of `a` starts here
12 |     a.push(4)
   |     ^^^^^^^^^ conflicting access here
13 |     r = 7
   |         - borrow later used here
```

Under 19's R2 the same two NEW lines make `r` an own name and `r = 7` "replaces that value": `a[1]` is
not written (or the line is a type error: `7` is not a `ref mut int`). So `r = ref mut a[1]` then `r = 7`
writes `a[1]`, and `r = first_even(a.as_mut_span())` then `r = 7` does not, though `r` holds the same
changeable view of the same element: one shape, two opposite meanings chosen by an earlier line, the
defect R18-03 was meant to remove. The owner's test: on `r = first_even(...)` R2 calls `r` "an own name",
which is not what it is; no word on the line says either.

**What would close it**: a name given a call whose result type is `ref mut T` is a `ref mut` name (R6
already calls the call a place), so `=` writes through, as AN-28's `r`.

### F-06 (readability, fix R18-03): whether a later `=` on `n` compiles depends on whether its first line ended with a call

`n = p.name` then `n = q.name`: refused (E3023, "for `q`'s name use a new name: `m = q.name`").
`n = p.name.trim()` then `n = q.name.trim()`, or `n = p.display_name()` then `n = q.display_name()` (a
method whose result views `p.name`, R6): accepted, `n` re-pointed. In every case `n` only reads a
person's name. The owner's test: neither first line has a word saying "a fixed view" or "an own name
holding a view"; the reader must know which right sides end in a call. 19's reason for R18-03 ("one shape,
two opposite meanings, chosen by an earlier line") applies to its own split. **What would close it**: one
rule for every name whose value is a read-only view (all may be given another value with `=`, or none),
stated in R2; no word is needed.

### F-07 (readability: consistency, fix R18-02): the reason 19 gives for removing AN-60's second way out applies equally to AN-60's accepted loop

19's AN-60 note: "`h(self)` would then hand the handler a second name for the list that `h` is an element
of, which R4 refuses. The position loop of point 3, which copies each handler's two words out before the
call (D4), is the one form for handlers that get the Ui". But AN-60's `fire(mut self)` calls
`self.handlers[self.current](self)`: by 19's R12 that call "is a change of the place that holds it"
(`self.handlers[self.current]`), and it hands the handler `self` with `ref mut`: a second name for the
list that the called value is an element of. And the copy-out is not what distinguishes the two forms:
R12 says every stored function value is "copied out of its list or field for the call", `h()` included.
So either R4 refuses both (and its one run-time compare, which always finds `self` and its own handler
overlapping, would stop every dispatch: the model then has no accepted form for handlers that get the
Ui), or the copy-out keeps both calls off the list (and the removed read-only-Ui `for` form had no R4
conflict). **What would close it**: one sentence in R12 saying whether a call through a copied-out stored
value counts, for R4, as a change of the place it was copied from, and AN-60 made to follow it.

### F-08 (readability, fix R18-12): `with thread_shared x` on a plain local is refused while R3 still says a capture word means what it means before a value, and the code given means something else

R3: "Before a value: ... `thread_shared x` (one more owner, any thread)"; "In a lambda's capture list
...: each word means what it means before a value". 19's AN-71 refuses `with thread_shared names` when
`names` is a plain local. So either `keep = thread_shared names` on a plain local is accepted (then the
word does one thing before a value and another in a capture list), or it is refused too (then R3's
"before a value" line is wrong for plain locals and has no message). And the refusal's code E7015 is,
in section 8, "a one-thread owner (`shared`, a `weak` link of one) or a lent value reaches another
thread"; `names = Names()` is neither (R18-09's own rule, one code one meaning). **What would close it**:
R3's `thread_shared x` line limited to a value already `thread_shared`, with AN-71's message on both
forms, under a code whose meaning covers it.

### F-09 (readability, fixes R18-10 and R18-09): a stored generator freezes its list or not according to its body, not its first line; and one mistake gets two codes

- `gen fn replay(self) -> Generator[int]:` is the first line of both the `for entry in self.log:` walk,
  which now freezes `d.log` while `g` is used, and the `while i < len(self.log):` walk, which 19's message
  gives as the way out and which does not. 19 cites R6 ("a method's result points into its object"),
  but R6 makes `g` a view of `d`, not of `d.log`, and R8 (b) does not end a view of an object at a push
  to one of its lists. So whether `d.log.push(...)` compiles while `g` lives is read from `replay`'s
  body; in another package only the interface file records it (R16). The word to say it on the first
  line exists: R6's `from` (`-> Generator[int] from log`). The owner's test on `g = d.replay()`: no word
  in any line the caller reads says `d.log` may not change.
- 19 moved AN-46's `ks` (a list of views of `m`'s keys used after `m.insert`) from E3020 to E3060
  because E3020 "is for a list changed while a loop walks it", and widened E3020 to "or while a generator
  that walks it is still used" for `g` used after `d.log.push`: the same mistake (a value holding views,
  used after what it views changed) has two codes.

**What would close it**: the `for` generator writes `from log` on its first line (refused without it,
with that line), and the stored-generator refusal uses E3060.

### F-10 (readability, fixes R18-09 and R18-11): E0113's meaning covers `mut self`, which R5 and 19's own AN-70 accept

Section 8: E0113 is "`mut` on a name (a pattern name, a parameter, a lambda parameter)"; section 1.3
refuses "`mut x` on a parameter". `self` is a parameter, and 19's AN-70 fix writes `virtual fn
on_update(mut self, dt: float):` (open point 7 keeps `mut self`). Read as written, the code refuses the
model's own accepted line. **What would close it**: E0113's row excepts `mut self` (and `static mut`),
pointing at open point 7.

### F-11 (readability, error-message rule, fixes R18-05 and R18-19): two of 19's new messages name things their program does not have

- E7016's message is given "on `@parallel` above `while i < n:` in `place`", but AN-84's `place` is
  `for i in 0..idx.len():` with the names `xs`, `idx`, `i`: no `while`, no `n`, no `x`, and the help's
  lines (`for i in 0..n:`, `for ref mut x in xs:`) are not in that program. In the same pass 19 applied
  "every name in a message is in its program" to AN-46 and AN-81.
- AN-81's new help offers `for j in i + 1..` to `step`, whose `j` is a parameter, not a loop name: a help
  that does not apply (A-8). `if i != j:` around the call is the line for `step`.

**What would close it**: write the refused `place` out (its `while` form) and give the lines in its
names; keep only `if i != j:` in AN-81's help.

### F-12 (readability, fix R18-15): the comma rule covers capture lists and loop names, and only a word on the first name

R3 (19): "Where a word stands before the first of several names separated by commas, in a capture list or
among a loop's names, every name in the list writes its own word". Not covered: a pattern with several
names (`State.Moving(ref mut pos, vel):`, `if Some((ref mut a, b)) = pair:`), and a word on a later name
(`for e, ref mut pos, vel in q:`), where `vel` can be read as covered by `ref mut` exactly as in the
refused `for ref mut pos, vel in q:`. Pattern names follow loop names everywhere else (R2, R5, E3023,
E0113). So "one comma shape, one rule", 19's stated result, holds in two of the places names stand in a
comma list. No NEW block has such a pattern today, so nothing written is wrong yet. **What would close
it**: "in any list of names (capture list, loop names, pattern names), a name with no word that follows a
name with a word is refused with the same advice".

## 4. The fixes that hold, and how each was checked

- **R18-06**: `owned f: once fn()` follows R4 (the mode on the name); `m59_owned_once.em` gives `exit=0`
  (re-run). AN-59's two messages agree with it.
- **R18-07**: `owned n: shared Node` is R4's "`owned x: shared T` makes the callee one more owner".
- **R18-08**: E0109 and one text in AN-47, AN-54, AN-78; each line to write uses its program's names.
- **R18-17**: AN-05's text now states the order and the stale-position case; the alternative it gives
  (read-only handler, then `retain`) passes the owner's test (`e` in the lambda is a plain parameter).
- **R18-18**: AN-35's E3013 message gives the NEW signature's reason and one line that compiles under R2.
- **R18-20**: R4 matches today (`o20_plain_int_param.em`, `exit=1`, E3023; `m20_int_param_copy.em`,
  `exit=0`, re-run).
- **R18-21**: AN-34's forms are two functions; `items` is used after no hand-over.
- **R18-11**: AN-70's `mut self` override is accepted today (`m70_virtual_changes.em`, `exit=0`, re-run),
  apart from F-10's code clash.
- **VOC**: the spec lines spot-checked name what section 1.4 says.

## 5. Files

All under `C:\Users\ism19\Code\ember\target\redesign3\20-re-attacker\`:
- `c\f1_weak_copy_escape.c`, `c\f2_refmut_view_repoint.c`, `c\f3_mutspan_overlap.c`,
  `c\f4_fnvalue_static_loop.c`, each with its `.exe` and its `_output.txt` (the full ASan reports; the
  `.pdb` debug files were deleted to keep the disk footprint small).
- `today\t1_ref_result_write.em`, `t2_ref_result_no_use.em`, `t3_overlap_spans.em`,
  `t4_overlap_floats.em` and `today\check_output.txt` (their `ember check` output; t2 `exit=0`, t4
  `exit=1` with E3021: today refuses two spans of one list even for floats).
- `scan_rebind.py`: the scan of section 1, step 5.
- `target\`: `ember check`'s own cache.
