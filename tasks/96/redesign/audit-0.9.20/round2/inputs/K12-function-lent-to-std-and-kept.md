# K12-function-lent-to-std-and-kept

Question: A function from a variable given to std that keeps it (thread.spawn, push, a returned adapter): lent, handed over, or owns?
Severity: high; impact rank 12
Members: J3-02, J6-12; primary J3-02
Why together: Both ask what verdict 150's loan means when the std function keeps the function past the call (spawn in J3-02, push in J6-12). Their recommendations draw the line in different places (see conflicts), so one rule must answer both.
Critic's notes: Show J6-12's `b.handlers.push(beep)` as the second line of J3-02's program. J3-02 A turns verdict 51's own line `a = thread.spawn(push_a)` into `thread.spawn(owns push_a)`, so say so. Option C reverses verdict 51 ('both ways should be permissible'), so label it that way. J3-03 (the owner's own std/program split, verdicts 147 and 150) may be shown beside it. K31 (J8-01) follows this answer.

## Finding J3-02
Rules: CLO-12, GRM-28, FN-2a, GRM-15, THR-10, CLO-7; file tasks/spec-0.9.9/parts/p06-expressions.md lines 331-333; kind contradiction; severity high
Quote from Hardened_11:
and may be passed by its name, so a long lambda may be written inside the call
  (`a = thread.spawn(owns fn() thread_shared log: log.push("a"))`) or as a local function
  (`fn push_a() thread_shared log:` with the body `log.push("a")`, then `a = thread.spawn(push_a)`).

The finder's question:
Is `thread.spawn(push_a)` (a local function with state, passed by name) a hand-over with no word, a loan, or
E3028? Can `push_a()` still be called after it?

Program:
```
import std.thread

fn main():
    log = thread_shared Log()
    fn push_a() thread_shared log:
        log.push("a")
    a = thread.spawn(push_a)
    push_a()
    a.join()
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.7)
Reason:
Nothing settles it. FN-2a (p05:132-134) and STD-19 (p15:316-317) lend a function from a variable to any
standard-library function, with no word. THR-10 (p11:173-175) says the spawned thread "may outlive the
function that started it", so a loan of `main`'s `push_a` to it cannot be kept, and CLO-4/CLO-7 refuse a
function value that outlives what it uses. No rule lets `thread.spawn(owns push_a)` hand a named function
over: FN-2a says such a function "takes no word". Yet CLO-12 (p06:333) and GRM-28 (p03:253) show `a =
thread.spawn(push_a)` as working code, and verdict 51 says both forms are allowed. So no spelling is decided
to compile. The same gap hits a std adapter kept past its line (`return xs.iter().map(h)`). Verdict 150
decided the loan for `for_each`, which runs the function during the call. It does not say what happens when
the std function keeps it.

options:
[{'letter': 'A', 'summary': "A loan stays the default; `owns` before a named function hands it over to any
`owns` parameter, the library's included.", 'code': '    a = thread.spawn(owns push_a)             # handed
over: the thread owns it now\n    a.join()\n\n# for_each still borrows with no word:\n#     h = fn(x: int):
println(x)\n#     [1, 2].iter().for_each(h)\n#     h(5)', 'outcome': 'Compiles and prints `a`. A later
`push_a()` is E3040, because `push_a` was handed over. `thread.spawn(push_a)` with no word is refused, because
the thread may outlive `push_a`, and the help gives `thread.spawn(owns push_a)`. `for_each(h)` keeps borrowing
and `h(5)` prints 5. `return xs.iter().map(owns h)` works the same way.'}, {'letter': 'B', 'summary': 'With no
word, a named function is handed over when the library function keeps it past the call.', 'code': '    a =
thread.spawn(push_a)                  # no word: handed over, since spawn keeps it\n    a.join()', 'outcome':
'Compiles and prints `a`. A `push_a()` after the spawn is E3040, though nothing in that line says `push_a` was
given away. `for_each(h)` still borrows, so the same-looking call means two things depending on what the
library function does with it.'}, {'letter': 'C', 'summary': 'A named function is always lent, so
`thread.spawn` cannot take one; write the function inside the call.', 'code': '    a = thread.spawn(owns fn()
thread_shared log: log.add("a"))   # the only form that compiles\n    a.join()', 'outcome':
"`thread.spawn(push_a)` is refused, because the thread may outlive `push_a`, and its help writes the function
into the call. A local function can never start a thread, and the spec's `thread.spawn(push_a)` examples
go."}]

recommendation:
{'letter': 'A', 'reason': 'One rule: a lent function is like a view and cannot outlive the variable, and
`owns` at a call hands over, as it does for every other argument. Both ways of writing a long thread body keep
working, and the hand-over is written in the line.', 'checklist': '2 (the local-function form, allowed by the
owner, compiles); 4 and 5 (the hand-over is written, and `owns` means one thing at every call); 6 (no new
exception: the loan with no word stays as decided); 9 (the refusal gives a line that compiles); 13 (no hidden
rule that depends on what the callee does with the function).'}

## Finding J6-12
Rules: STD-19, FN-2a, OWN-9; file tasks/spec-0.9.9/parts/p15-stdlib.md lines 312-317; kind inconsistency; severity high
Quote from Hardened_11:
(`return xs.iter().map(owns fn(x) => x * 2)`), what the lambda holds with it. A function from a
  variable is lent to the call and takes no word: after `xs.iter().for_each(h)`, `h` is the caller's
  still and `h(5)` runs it (ODR-240).

The finder's question:
For an `owns` parameter of a function type, do `b.on_click(beep)` (the program's own method) and
`b.handlers.push(beep)` (std's `push`, which keeps what it is given) compile without `owns`? Is `beep` still
the caller's afterwards?

Program:
```
type Handler = fn()

class Button:
    handlers: Array[Handler] = []

    fn on_click(owns f: Handler):
        this.handlers.push(owns f)

fn main():
    b = Button()
    beep = fn(): println("beep")
    b.handlers.push(beep)   # std function given a function from a variable: lent?
    b.on_click(beep)        # same parameter in the program's own code: E3028?
    beep()
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.7)
Reason:
Part decided: the owner ruled that a function from a variable given to a standard-library function is lent
(verdict 150, ODR-240, FN-2a p05 132-134, STD-19 p15 315-317), and that the program's own `owns` parameters
take `owns` at the call (verdict 147); that split is the owner's and is not reopened here. Still open: std's
`push` keeps what it is given and takes `owns` at the call (verdict 149: "`push` too ... `names.push(p.name)`
is an error"), while verdict 150 lends a function from a variable to std. For `handlers.push(beep)` the two
verdicts give different answers, and a list cannot hold a lent function past the call. ODR-240's 'std's
callable-taking functions' does not say whether push on a list of functions is one.

options:
[{'letter': 'A', 'summary': 'push keeps it, so it is handed over like any value; lending stays for functions
given as a function parameter', 'code': "b.handlers.push(owns beep)              # beep is the list's
now\nb.handlers.push(owns copy of beep)      # or keep beep and give the list a copy", 'outcome':
'`b.handlers.push(beep)` is an error: "`push` keeps what it is given; hand `beep` over: `push(owns beep)`, or
keep it: `push(owns copy of beep)`". With `owns beep`, a later `beep()` is E3040; with `owns copy of beep`, it
prints "beep" and the loop prints "beep". A function from a variable is still lent, with no word, to a
standard function that takes a function as its function parameter (`for_each(h)`, `map(h)`,
`retain(keep)`).'}, {'letter': 'B', 'summary': 'Lent, and the list holds a view of it', 'code':
'b.handlers.push(beep)', 'outcome': 'Compiles; the list holds a view of `beep` and may not outlive it (as a
list of `StrView`s may not outlive their text): returning or storing `b` past `beep`\'s scope is E3063.
`beep()` prints "beep".'}]

recommendation:
{'letter': 'A', 'reason': 'A list owns its elements, so its `push` hands over a function as it does every
other value, with `owns` written at the call; lending stays exactly where the library already takes a function
as a function parameter (STD-19), so no line keeps a value silently.', 'checklist': "1, 4 (the word at the
call says it), 5, 6 (push's existing rule), 9, 11"}

