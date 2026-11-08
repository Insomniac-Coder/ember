# K40-function-value-inline-state

Question: When does a stored function value allocate: is the one-word state rule fixed or optional?
Severity: medium; impact rank 40
Members: J3-15; primary J3-15
Why together: Single question.
Critic's notes: 

## Finding J3-15
Rules: CLO-10, PHIL-2; file tasks/spec-0.9.9/parts/p06-expressions.md lines 275-279; kind undecided_case; severity medium
Quote from Hardened_11:
  `{code, state}`. Its state is what it needs besides its code: what it captured with a word, in one
  block of its own made with the value (it carries `Alloc` and is reported by `ember inspect --alloc`),
  except that a state of one word or less may be kept in the state word itself, with no block; and

The finder's question:
Under `@noalloc`, does a function value whose state is one word (`copy of dt`), or one that uses two real
variables, compile, or is it E4001?

Program:
```
@noalloc
fn fall(ref mut ps: Array[Particle], dt: float, gravity: float):
    pull: fn(ref mut Particle) = fn(ref mut p): p.vy -= gravity * dt
    for ref mut p in ps:
        pull(ref mut p)
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.6)
Reason:
CLO-10 (p06:276-279) says captured state goes in a block that allocates, "except that a state of one word or
less may be kept in the state word itself". "May" leaves it to the compiler, and PHIL-2 (p01:51) calls it "its
inline capacity" without fixing it. CLO-10 also says the real variables a function uses cost "nothing beyond
their addresses", but not where two or more addresses are kept. So whether a `@noalloc` function holding a
function value compiles (E4001, `[EFF-?]`'s Alloc) depends on the implementation, which PHIL-2's "No heap
allocation happens that the source does not show" forbids. Nothing else fixes the capacity (grep: `state
word`, `inline capacity`).

options:
[{'letter': 'A', 'summary': 'The language fixes it: the variables a function uses are reached through one
address, and that address or one word of its own state stays inline; anything more allocates.', 'code': '
pull: fn(ref mut Particle) = fn(ref mut p): p.vy -= gravity * dt   # one address: no block\n    slow: fn(ref
mut Particle) = fn(ref mut p) copy of k: p.vy *= k    # one word: no block', 'outcome': "Both functions
compile under `@noalloc`, and the program prints the first particle's speed. A function value holding two
words of its own (`copy of k, copy of m`), or its own state plus the variables it uses, allocates a block,
carries `Alloc` and is E4001 under `@noalloc`."}, {'letter': 'B', 'summary': 'Any state of its own always goes
in a block; only the variables it uses (one address) stay inline.', 'code': '    slow: fn(ref mut Particle) =
fn(ref mut p) copy of k: p.vy *= k    # a block, even for one float', 'outcome': '`fall` compiles under
`@noalloc`; `drag` is E4001, whose help gives a type parameter (`[F: needs fn(ref mut Particle)]`) or the loop
body written out.'}, {'letter': 'C', 'summary': 'Keep the "may": the compiler decides whether a one-word state
is inline.', 'code': '    slow: fn(ref mut Particle) = fn(ref mut p) copy of k: p.vy *= k', 'outcome':
'Whether `drag` compiles under `@noalloc` depends on the compiler that builds it.'}]

recommendation:
{'letter': 'A', 'reason': 'Allocation becomes a rule of the language that the source shows, as PHIL-2 demands,
and small function values cost no more than a C function pointer plus one word.', 'checklist': '3 (decided
when compiling, cheapest form); 7 (production-ready: the same answer on every compiler, with no unstated
ceiling); 8 (a function value that grows past the limit is refused under `@noalloc`, never allocates in
secret); 9.'}

