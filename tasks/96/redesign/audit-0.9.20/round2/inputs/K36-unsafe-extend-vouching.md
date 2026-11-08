# K36-unsafe-extend-vouching

Question: What may a program vouch for with unsafe extend: Send for a class, ThreadShared for a C wrapper?
Severity: medium; impact rank 36
Members: J7-06, J2-12; primary J7-06
Why together: Both ask about TYP-43's one hand-written vouching rule: which types it covers (struct, enum, class: J2-12) and which markers (Send only, or ThreadShared too: J7-06). One edit answers both.
Critic's notes: Pairs with K02: under J7-03 A, vouching ThreadShared is the only way a raw-pointer wrapper becomes thread_shared.

## Finding J7-06
Rules: THR-9, TYP-43, THR-8, STA-1; file tasks/spec-0.9.9/parts/p11-concurrency.md lines 54-58, 59-61, 64-65; kind inconsistency; severity medium
Quote from Hardened_11:
the author of a type that holds a raw / pointer declares it with `unsafe extend T implements Send:` and a `#
SAFETY:` note / It is inferred like `Send` and written only in bounds

The finder's question:
A type that holds a raw pointer to a thread-safe C object can be vouched `Send` but never `ThreadShared`
(E2040). Does the program below compile, and if it does not, how does a program let several threads read such
a value?

Program:
```
struct CLogger:
    raw: *mut c_logger      # c_logger locks internally

# SAFETY: every c_logger function takes its own lock
unsafe extend CLogger implements Send:

static LOGGER: CLogger = CLogger(raw=null)    # STA-1: T MUST be ThreadShared -> E7002

fn log_all(lg: CLogger, lines: Span[String]):
    with scope = thread.scope():
        for t in lines:
            scope.spawn(owns fn() ref t: c_log(lg, t))   # needs `ref CLogger` Send, i.e. CLogger ThreadShared
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.65)
Reason:
TYP-43 (p04 591-594) allows `unsafe extend` only of `Send`, and makes `ThreadShared` written by hand E2040.
THR-9 (p11 64) says raw pointers are never `ThreadShared`. So a program's wrapper of a thread-safe C handle
can be vouched `Send` but never `ThreadShared`. It cannot be a `static` (STA-1, E7002), cannot be read by
scoped tasks (`ref T` is `Send` only for a `ThreadShared` T, THR-8), and cannot usefully be `thread_shared`.
The only way out is a `Mutex` around it, an Ember lock on top of C's own. ODR-111 (p26 399, a delegation
ruling, not the owner's) had allowed `Send|Sync`; when `Sync` was renamed `ThreadShared`, the second was
dropped without a verdict. JOB.md has no verdict on vouching `ThreadShared`.

options:
[{'letter': 'A', 'summary': 'Vouch for it the way Send is vouched for', 'code': "# SAFETY: every logger_write
call takes the C logger's own lock, so threads may share one logger\nunsafe extend Logger implements
ThreadShared:", 'outcome': 'Compiles. The three tasks log at once, and `static LOGGER: Logger = ...` is
allowed too. The vouching is listed by `ember tcb` like the `Send` one, and written without `unsafe` it is
E3100.'}, {'letter': 'B', 'summary': "Keep today's rule: share it only through an Ember lock", 'code': 'from
std.sync import Mutex\n\nfn log_one(lg: Mutex[Logger], e: c_int):\n    with g = lg.lock():\n
g.write(e)\n\n# in main:\n    lg = Mutex(open_logger())\n    ...\n            scope.spawn(owns fn() copy of e:
log_one(lg, e))', 'outcome': "The original does not compile. This rewrite compiles, and every event takes an
Ember lock as well as C's own, so the tasks wait for each other for nothing."}]

recommendation:
{'letter': 'A', 'reason': "The language already spells 'I vouch for what the compiler cannot see' for `Send`,
and the sibling property needs the same words. Without them a correct C binding pays a lock it does not need
(Rust: `unsafe impl Sync`).", 'checklist': '11 (the same idea is spelled exactly as for `Send`), 12 (no case
left out because the syntax is missing), 2 (accepts a correct program), 3 (no lock added at run time),
principle 11 (the vouching lives in `unsafe`, with its note)'}

## Finding J2-12
Rules: TYP-43, THR-8; file tasks/spec-0.9.9/parts/p04-types.md lines 586-588; kind inconsistency; severity low
Quote from Hardened_11:
A program writes `Send` by
  hand, for a struct or enum of its own package that holds a raw pointer, only as `unsafe extend T
  implements Send:`

The finder's question:
May a class that holds a raw pointer (a wrapper of a foreign handle) be declared `Send` with `unsafe extend`?

Program:
```
class GpuTexture:
    handle: *mut u8
    size: int

# SAFETY: the driver's texture handles may be used from any thread
unsafe extend GpuTexture implements Send:
    pass

fn main():
    tex = GpuTexture(handle=null[*mut u8](), size=0)
    h = thread.spawn(owns fn() owns tex: tex.size)
    println(h.join())
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.6)
Reason:
TYP-43 (p04 586-588) limits the hand-written `unsafe extend T implements Send:` to 'a struct or enum of its
own package that holds a raw pointer'; THR-8 (p11 54-55) says 'the author of a type that holds a raw pointer
declares it' and calls a class object Send when its fields are (p11 48). A class wrapping a C handle is then
never Send by TYP-43 and may be by THR-8, with no reason given for leaving classes out (a `shares` owner of
the class stays one-thread whatever its fields are). No verdict.

options:
[{'letter': 'A', 'summary': "A struct, enum or class of the program's own package may be vouched Send this
way.", 'code': 'unsafe extend GpuTexture implements Send:\n    pass', 'outcome': 'Compiles; prints `0`.'},
{'letter': 'B', 'summary': 'Only a struct or enum; a class holding a raw pointer is never Send, and the
program wraps the pointer in a struct it vouches for.', 'code': "struct TexHandle:\n    raw: *mut u8\n\n#
SAFETY: the driver's texture handles may be used from any thread\nunsafe extend TexHandle implements Send:\n
pass", 'outcome': '`unsafe extend GpuTexture implements Send:` is refused; with `handle: TexHandle` in the
class it compiles and prints `0`.'}]

recommendation:
{'letter': 'A', 'reason': "The vouching is about the pointer, not about the kind of type holding it; nothing
true about classes needs the exception, and THR-8 already says 'a type'.", 'checklist': "5 (one rule for every
type), 6 (no exception without a reason), 11 (THR-8's wording kept)"}

