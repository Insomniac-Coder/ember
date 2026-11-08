# K57-export-threads-main-creator

Question: May an export marked creator change a module variable, as one marked main may?
Severity: medium; impact rank 57
Members: J8-10; primary J8-10
Why together: Single question.
Critic's notes: 

## Finding J8-10
Rules: FFI-33, FFI-33c, STA-1, GRM-37; file tasks/spec-0.9.9/parts/p16-ffi.md lines 660-684; kind inconsistency; severity medium
Quote from Hardened_11:
(line 1)", whose help gives the lock's lines and the `Atomic`'s). The same holds under `creator`.
* `[FFI-33c]` ... Under `threads = main`, the exported wrapper checks, in every profile, that
  the calling thread is the one that initialised the module, and panics otherwise; in exchange the body

The finder's question:
Does an export under `threads=creator` that changes a module `static mut` compile? Under `threads=main` it
does (FFI-33c), and both contracts check at run time that the caller is the thread that initialised the
module.

Program:
```
#! threads creator
static mut HITS: int = 0

# SAFETY: the host defines no other symbol named `bump`
@unsafe(export("bump"))
pub extern "C" fn bump() -> i32:
    HITS += 1       # E7012 under creator (FFI-33); allowed under `#! threads main` (FFI-33c)
    return 0
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.55)
Reason:
FFI-33 (p16-ffi.md 666-677): under `creator` a `static mut` an export reaches must be `guarded by` a lock
(E7012), and `creator` is "the thread of the first successful `ember_rt_init`" (or `ember_module_init`), with
"initialization after shutdown establish[ing] a new creator". FFI-33c (681-683): under `main` the wrapper
checks "that the calling thread is the one that initialised the module ... in exchange the body may use
thread-confined state, the module's `static mut`s among it". The two checks name the same thread, yet one
permits the `static mut` and the other refuses it. The only difference the text implies, re-initialisation on
another thread, is stated for `creator` and not for `main`. STA-1 (p05-declarations.md 478) ties a `static
mut` to "the main thread", and for a library module that thread is undefined except by FFI-33c. No verdict
covers this.

options:
[{'letter': 'A', 'summary': "Keep the difference and give its reason. An export's `main` is the first thread
that ever initialised the module, fixed for the process, and initialising again on another thread panics. The
module's `static mut`s live on that thread. `creator` may move after a re-initialisation, so a `static mut` it
reaches needs a lock.", 'code': '#! threads main\nstatic mut HITS: int = 0\n\n@unsafe(export("bump"))\npub
extern "C" fn bump() -> i32:\n    HITS += 1\n    return 0', 'outcome': 'Under `main` this compiles. Under
`creator` it is E7012, whose message gives the reason: a later initialisation may run on another thread. A
host that re-initialises a `main` module on another thread gets a panic naming both threads.'}, {'letter':
'B', 'summary': "One thread for both words: the thread of the current initialisation. The module's `static
mut`s get their first values again at each initialisation, and under either word an export may change them
with no lock.", 'code': '#! threads creator\nstatic mut HITS: int = 0\n\n@unsafe(export("bump"))\npub extern
"C" fn bump() -> i32:\n    HITS += 1\n    return 0', 'outcome': 'Compiles under both words, and a
re-initialisation starts `HITS` at 0 again. For exports `main` and `creator` then mean the same thing.'}]

recommendation:
{'letter': 'A', 'reason': 'It keeps both written rules and states the one fact that makes them differ, so
`main` keeps a meaning of its own. A forgotten lock under `creator` stays a compile-time refusal.',
'checklist': '6 (the exception says why), 5 (each word one meaning), 3 (the existing one-compare check), 8'}

