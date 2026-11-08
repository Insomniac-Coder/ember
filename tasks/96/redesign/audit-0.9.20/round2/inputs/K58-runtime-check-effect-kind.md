# K58-runtime-check-effect-kind

Question: Does a runtime parameter's check count as a written panic, an alias check, or both?
Severity: medium; impact rank 58
Members: J1-02, J5-10, J6-01; primary J1-02
Why together: The same question, found three times. J5-10 asks only the @no_alias_checks half, and J1-02 and J6-01 ask both contracts. All recommend A.
Critic's notes: Use J1-02's three options (J5-10 lacks the @nopanic(explicit) dimension). The answer sets J1-01's lead-fix wording and 'the one run-time alias check there is' in EFF-9, DSJ-6 and the E4030 row. All three examples use a bare `dyn` local, which G1's resolution keeps legal.

## Finding J1-02
Rules: EFF-9, EFF-12, DSJ-6, LT-48, TIER-1 table (Contract tier); file tasks/spec-0.9.9/parts/p01-overview.md lines 27, 140-142; kind undecided_case; severity medium
Quote from Hardened_11:
Which mechanism enforces a guarantee is set per property by this specification, never by the
programmer, and it is always visible (`ember inspect --safety`) and restrictable (`@no_alias_checks`,
`[EFF-12]`).

The finder's question:
Does a `runtime` parameter or variable compile inside a `@no_alias_checks` function, and inside a
`@nopanic(explicit)` one? What kind of RuntimeCheck does its stop record?

Program:
```
@no_alias_checks
@nopanic(explicit)
fn mix(p: runtime dyn Picker, ref mut xs: Array[int]):
    r = p.pick(xs)
    xs.push(owns 3)
    println(r)
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.8)
Reason:
No rule classifies the `runtime` check. EFF-9 lists RuntimeCheck(Aliasing) as only the compare of
`assert_disjoint`/`check_disjoint`, 'the one run-time alias check there is' (p10 69-70). EFF-12 says
`@no_alias_checks` 'forbids only RuntimeCheck(Aliasing), the run-time compare of assert_disjoint or
check_disjoint' (p10 196-198). DSJ-6 repeats 'the one run-time alias check' (p09 513). The Panic(Explicit) row
(p10 16) names `assert_disjoint` and `cast x as T` but not `runtime`. So whether a `@no_alias_checks` or
`@nopanic(explicit)` function may take a `runtime` parameter is undecided, and the two nearest rules point
different ways: `assert_disjoint` is both Aliasing and Explicit, while a bounds check is implicit. No verdict
covers contracts and `runtime`.

options:
[{'letter': 'a', 'summary': 'As `assert_disjoint`: an alias check the programmer asked for, refused under both
contracts', 'code': '@no_alias_checks\n@nopanic(explicit)\nfn mix(m: runtime dyn Mixer, ref mut samples:
Array[f32]):\n    gains = m.pick(samples)\n    samples.push(owns 0.0)\n    println(gains)', 'outcome': "Under
`@no_alias_checks` the `runtime` word is refused (E4030, 'compares the kind of `m` while running'). Under
`@nopanic(explicit)` it is refused too (E4040). Without the two contracts it compiles, prints for a Fixed and
stops at the push for a PassThrough. Where every kind that reaches `m` is known when compiling, nothing is
compared and both contracts accept it."}, {'letter': 'b', 'summary': 'As a bounds check: an alias check the
compiler adds, refused only under @no_alias_checks', 'code': '@nopanic(explicit)\nfn mix(m: runtime dyn Mixer,
ref mut samples: Array[f32]):\n    gains = m.pick(samples)\n    samples.push(owns 0.0)\n    println(gains)',
'outcome': 'With `@no_alias_checks` it is refused (E4030). With only `@nopanic(explicit)` it compiles, and its
stop counts as a check like an index out of bounds, not as a panic the programmer wrote.'}, {'letter': 'c',
'summary': 'As `cast s as Circle`: a kind compare the programmer wrote, refused only under
@nopanic(explicit)', 'code': '@no_alias_checks\nfn mix(m: runtime dyn Mixer, ref mut samples: Array[f32]):\n
gains = m.pick(samples)\n    samples.push(owns 0.0)\n    println(gains)', 'outcome': 'With only
`@no_alias_checks` it compiles, since the check counts as a kind test and not an alias check. With
`@nopanic(explicit)` it is refused (E4040).'}]

recommendation:
{'letter': 'a', 'reason': 'The nearest existing rule is `assert_disjoint`, a check the program asks for that
stops when two names look at the same memory. It counts as both an alias check and a panic the programmer
wrote, and `runtime` is exactly that. A written `@no_alias_checks` then means what it says, with no alias
check left running in the function.', 'checklist': '5, 6 (nearest rule `assert_disjoint`), 11 (same idea
classified the same way), 15 (a written contract means only what it says), 8'}

## Finding J5-10
Rules: DSJ-6, EFF-12, LT-48; file tasks/spec-0.9.9/parts/p09-memory.md lines 512-514; kind verdict_missing; severity medium
Quote from Hardened_11:
`@nosync` code. They are the one run-time alias check there is, so inside `@no_alias_checks` each is

The finder's question:
Does a `runtime` parameter or variable (verdicts 152, 162, 163) compile inside a `@no_alias_checks` function?

Program:
```
@no_alias_checks
fn show(p: runtime dyn Picker, ref mut xs: Array[int]):
    r = p.pick(xs)
    xs.push(owns 3)        # checked while running: stops when p is a Yours, whose pick gives back xs
    println(r)
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.65)
Reason:
Verdicts 152, 162 and 163 added a run-time check that stops when a list changes while a view looks at it. Yet
[EFF-9] (p10 69-70) and [DSJ-6] (p09 512-514) still call `assert_disjoint`/`check_disjoint` "the one run-time
alias check there is", [EFF-12] forbids only their compare, and the cost table row (p10 315) gives `runtime`
no check kind. Read literally, `@no_alias_checks` allows `runtime`; by what the attribute means, it forbids
it. No verdict covers the pair.

options:
[{'letter': 'A', 'summary': 'The `runtime` check is a run-time alias check, so inside `@no_alias_checks` it is
refused.', 'code': 'interface Picker:\n    fn pick(other: Array[int]) -> Span[int]\n\nstruct Mine:\n    items:
Array[int]\n\nextend Mine implements Picker:\n    fn pick(other: Array[int]) -> Span[int]:\n        return
this.items\n\n@no_alias_checks\nfn show(p: Mine, ref mut xs: Array[int]):    # a kind known when compiling: no
check runs\n    r = p.pick(xs)\n    xs.push(owns 3)\n    println(r)\n\nfn main():\n    xs = [1, 2]\n
show(Mine(items=[9]), ref mut xs)', 'outcome': 'The example is E4030 at `p: runtime dyn Picker`: "`show` is
`@no_alias_checks`, and `runtime` on `p` checks while running whether `xs` changes while `r` looks at it; give
`show` a kind known when compiling, or remove `@no_alias_checks`". This version compiles and prints [9]. The
specification\'s "the one run-time alias check there is" becomes two: `assert_disjoint`/`check_disjoint` and
`runtime`.'}, {'letter': 'B', 'summary': "The `runtime` check tests a value's kind, which `@no_alias_checks`
allows, as it allows a cast's check.", 'code': 'interface Picker:\n    fn pick(other: Array[int]) ->
Span[int]\n\nstruct Mine:\n    items: Array[int]\n\nextend Mine implements Picker:\n    fn pick(other:
Array[int]) -> Span[int]:\n        return this.items\n\nstruct Yours:\n    unused: int\n\nextend Yours
implements Picker:\n    fn pick(other: Array[int]) -> Span[int]:\n        return other\n\n@no_alias_checks\nfn
show(p: runtime dyn Picker, ref mut xs: Array[int]):\n    r = p.pick(xs)\n    xs.push(owns 3)\n
println(r)\n\nfn main():\n    xs = [1, 2]\n    p: dyn Picker = Mine(items=[9]) if input() == "m" else
Yours(unused=0)\n    show(p, ref mut xs)', 'outcome': 'Compiles. Given m, it prints [9]. Otherwise it stops at
`xs.push(owns 3)`: "`xs` changes while `r` looks at it: `p` is a `Yours`, whose `pick` gives back the list it
was handed".'}]

recommendation:
{'letter': 'A', 'reason': '`@no_alias_checks` promises that no aliasing check runs in the function, and
`runtime` checks exactly that: its stop message says a list changed while a view looked at it. A written
attribute means only what it says.', 'checklist': '5 ("alias check" means one thing), 6, 15 (a written
declaration means only what it says)'}

## Finding J6-01
Rules: EFF-9, EFF-16, COST-6, LT-48, DSJ-6; file tasks/spec-0.9.9/parts/p10-effects.md lines 17, 65-71, 225-232, 355-372; kind contradiction; severity medium
Quote from Hardened_11:
`Aliasing` covers the compare of `assert_disjoint` and `check_disjoint`
  (`[DSJ-1]`), the one run-time alias check there is;

The finder's question:
What effect does a `runtime` parameter's or variable's check carry? Does a `@no_alias_checks` or
`@nopanic(explicit)` function with a `runtime dyn` parameter compile, and does `ember inspect --safety` report
it as free of RuntimeCheck?

Program:
```
@nopanic(explicit)
@no_alias_checks
fn present(r: runtime dyn Renderer, ref mut scratch: Array[u8]):
    frame = r.draw(scratch)
    scratch.push(owns 1)      # stops here when r is a Software renderer (LT-48)
    println(frame.len())
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.8)
Reason:
LT-48 (p07 704-721, verdicts 152, 162, 163) adds a check that runs and stops the program when a view and a
change overlap. No rule gives it an effect: the effect table's Panic(Explicit) row (p10 16) does not list it,
EFF-9 (p10 65-71) and DSJ-6 (p09 512-513) still say assert_disjoint/check_disjoint are 'the one run-time alias
check there is', and COST-6's table of stops (p10 355-372) has no row for it, although COST-3 has its cost row
(p10 315). EFF-16 (p10 225-232) promises that a @nopanic(explicit) function free of RuntimeCheck 'has no panic
the effect model knows of', and this check breaks that. Whether @no_alias_checks or @nopanic(explicit) code
may hold a `runtime` name changes which programs compile, and no verdict says.

options:
[{'letter': 'A', 'summary': 'Both, the way `assert_disjoint` is: a stop the programmer asked for, and a
run-time alias check', 'code': '@nopanic(explicit)\n@no_alias_checks\nfn present(r: runtime dyn Renderer, ref
mut scratch: Array[u8]):\n    frame = r.draw(scratch)\n    scratch.push(owns 1)\n    println(frame.len())',
'outcome': 'Refused twice: `@nopanic(explicit)` (E4040) because `runtime` may stop the program at line 5, and
`@no_alias_checks` (E4030) because it compares while running. Without the two contracts it compiles; a Gpu
prints 2, a Software stops at the push with the approved message. COST-6 gets a row for the stop.'},
{'letter': 'B', 'summary': 'An alias check only, like a bounds check', 'code': '@nopanic(explicit)\nfn
present(r: runtime dyn Renderer, ref mut scratch: Array[u8]):\n    frame = r.draw(scratch)\n
scratch.push(owns 1)\n    println(frame.len())', 'outcome': 'Compiles under `@nopanic(explicit)` (the check
counts with bounds and overflow checks, and `ember inspect --safety` reports it); `@no_alias_checks` refuses
it (E4030).'}, {'letter': 'C', 'summary': 'A written panic only, like `cast s as Circle`', 'code':
'@no_alias_checks\nfn present(r: runtime dyn Renderer, ref mut scratch: Array[u8]):\n    frame =
r.draw(scratch)\n    scratch.push(owns 1)\n    println(frame.len())', 'outcome': 'Compiles under
`@no_alias_checks`; `@nopanic(explicit)` refuses it (E4040).'}]

recommendation:
{'letter': 'A', 'reason': "`runtime` is a word the programmer writes to ask for a stop, as `assert_disjoint`
and `cast` are, and the stop guards an overlap of a view and a change, as `assert_disjoint`'s does;
`assert_disjoint` already carries both effects, so the same thing gets the same rule and EFF-16's promise
stays true.", 'checklist': '1, 2, 3 (decided when compiling), 5, 6 (nearest rule: assert_disjoint), 11 (the
existing two-effect spelling reused), 13 (no special path), 15'}

