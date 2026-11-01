# Owner answers, ODR round of 2026-11-01 (not yet verdicts in JOB.md)

Every answer is in the owner's words, or is my sentence the owner answered "yes" to (marked).
Nothing here is widened. They go into JOB.md (145+), decisions.md and a new hardening after the round.

1. An argument for an `owns` parameter carries `owns` at the call: `take(owns xs)`; `take(xs)` is an error.
   (my question; owner: "YES")
2. A copy handed to an `owns` parameter: `take(owns copy of xs)` (owner: "I think owns copy of will make
   things more clear"); a literal too: `take(owns [1, 2])` (my question; owner: "yes").
3. `copy of`'s return type stays `Array[int]` (owner: "keep it Array[int]"); `return owns xs` loses the
   word (owner: "remove owns").
4. A named function passed to a std function that takes a callable is borrowed (my question "SO IT BORROWS
   RIGHT?" was his; my answer yes; owner: "YES GOOD"). ODR-240 stands.
5. `push` too: `names.push(owns p.name)` (owner: "yes owns at the call for push too").
6. N31: (a), `owns` on any value `=` copies hands it over, as for a number (`f = owns flag` ends `flag`)
   (owner: "yes noext question"). N32: (a), `show(owns xs)` is an error, "remove `owns`" (owner: "yes pick a,
   next question"); my reason given with it, "`owns` is never written before an argument", was overturned by 1.
7. dyn, the Picker example (tasks: keep this example):
   - `show(Mine(items=[9]))` works; `show(Yours(items=[9]))` fails at `xs.push(owns 3)` (owner: "MAKE MINE
     WORK AND YOURS FAIL BUT AT XS.PUSH").
   - a kind picked while running needs a new word (owner: "add a way to check it at runtime but with a
     word", "THAT IS NOT WHAT I MEANT I WANT A NEW WORD").
   - the word is `runtime`, on the function's parameter (owner: "go with 3, keep runtime"):
     ```ember
     fn show(p: runtime dyn Picker):
     ```
   - what it does (my sentences, owner: "yes"): a Mine prints [9]; a Yours stops the program at
     `xs.push(owns 3)` with a message that `r` looks at `xs`; without the word, a kind not known when
     compiling is a compile error that says to write the word; where the kind is visible no word is needed.
   - `fn show(p: runtime dyn Picker)` called as `show(Yours(items=[9]))`, a kind the compiler can see: it
     does not compile, the error is at `xs.push(owns 3)` as without the word; `runtime` only matters when
     the compiler can't tell the kind (my option (a); owner: "a").
8. A hasher may be `Copy` (owner: "COPYING SHOULD NOT BE AN ISSUE MAKE IT AN ODR"): decisions.md N37;
   `[HASH-1]` loses "that is not `Copy`" (my words from ODR-033, never the owner's).
9. GRM-40: a blank line or a comment line between `[T]` and its `fn` is an error (`E0100`): `[T]` must be
   on the line right above (my option (a); owner: "A"). Today both compile.
10. GRM-40: every line that starts with `[` and is not directly above a function, struct or class is `E0100`,
    `[10, 20].len()` as a line of its own included (my option (a); owner: "a"). A `[T: needs Add]` above an
    `enum` gets one `E0100` in place of today's four errors.
11. LEX-14: every keyword used as a name is `E0005` (my option (a); owner: "a"), with the words shown in (a):
    "`type` is a keyword", help "write `r#type` to use the word as a name". Test change shown and accepted
    with it: `LEX-15b/reject_yield_as_a_name.em` expects `E0005` for `yield = 1` (was `E0100`). Not decided:
    the future words' own message ("`async` is reserved for v2", help "... as an identifier") stays as is.
12. Not an ODR (tooling, owner rule 2026-10-30): N30, the upgrader writes `owns copy of` where 0.9.10 code
    reuses a number after giving it to an `owns` parameter. Told the owner in one line.
13. N33, `this` in a function outside any struct or class (owner: "ok use that message"):
    ```
    error[E1010]: `this` can only be used in a method, and `double` is not a method
      = help: to make `double` a method, move it into the struct or class it belongs to
    ```
    The help line was in the block shown; the owner approved "that message".
14. N33, `this` in a `static fn` (my option (b); owner: "b"): "`this` can only be used in a method, and
    `starting_at` is a `static fn`, not a method", help "build the counter from `k`: `return Counter(n=k)`"
    (replaces [FN-4]'s example words in the next hardening). When the help is given in general (only where
    the compiler can see what to build) was in my earlier proposal, not asked again.
15. N34, `E2118`'s help (my option (a); owner: "a"): the new name is the parameter's name with `_param` added:
    "rename the parameter: for a view of the field write `fn next_word(src_param: StrView) -> StrView from
    src:`, for a view of the parameter `fn next_word(src_param: StrView) -> StrView from src_param:`"
    (replaces [LT-1a]'s `line` in the next hardening).
16. N35, `E3062`'s help where the result is a view of the whole object (`return ref this.at` under
    `from owner`) (my option (b); owner: "vb"): "return a view of `owner`, or remove `from owner`".
17. The owns-at-the-call messages (owner: "ok approve"), with `fn take(owns xs: Array[int])`, `fn show(xs:
    Array[int])`:
    - `take(xs)`: "error[E3028]: `take` takes `xs` over (its first line says `owns xs: Array[int]`); say so at
      the call: `take(owns xs)`", help "to keep `xs`, give it a copy: `take(owns copy of xs)`"
    - `show(owns xs)`: "error[E3028]: `show` only reads its argument; remove `owns`: `show(xs)`"
    - `return owns xs`: "error[E0109]: `return` always hands the value over; remove `owns`: `return xs`"
18. The two `runtime` messages (owner: "approve"), Picker example, `xs.push(owns 3)` at 17:5:
    - no `runtime` on `show`, kind known only while running: "error[E3021]: `xs` changes while `r` may still
      look at it: `p` may be a `Yours`, and which kind it is is only known while running", help "to check it
      while running, write `runtime` on the parameter: `fn show(p: runtime dyn Picker):`"
    - with `runtime`, a Yours at run time: "panic at main.em:17:5: `xs` changes while `r` looks at it: `p` is
      a `Yours`, whose `pick` gives back the list it was handed"

The ODR round is complete (2026-11-01 evening). Next: the test changes for the owner, then JOB.md 145+,
decisions.md, the next hardening, the build.

19. A function written in a call to a std function says `owns` (verdict 147 applied): `for_each(owns fn(x: int):
    println(x))`, `filter(owns fn ...)`, `map(owns fn ...)`; a named one stays `for_each(h)` (owner: "a"). JOB.md 158.
20. A constructor argument takes no word, as `=`: `Words(names=xs)` hands `xs` over (owner: "b"). JOB.md 159.
    Read by the lead to cover an enum variant's constructor the same way (`Some(t)`), told to the owner.
21. A value made with `shares`/`weakref` in the call takes no `owns`: `setup(shares mut game)`,
    `ROSTER.push(weakref this)` (owner: "yes go with b"). JOB.md 160.
