# Features not implemented because they require another feature

Each entry is a spec feature the compiler does not have yet because another
feature it is built on does not exist yet. None of these is a deviation: the
compiler does not do something *different* from the rule, it does not do the
rule yet, and it says so (an error, or nothing observable). When the feature it
waits on lands, the entry is built and removed.

Moved here from `docs/DEVIATIONS.md` on 2026-09-27 (owner: they are not
deviations). Their old numbers are kept in brackets so earlier records can be
followed.

---

There are no entries now. The last, N2 (a C++ base class Ember classes derive from, `[FFI-39]`),
was built in October 2026 as the current specification declares it: by an overlay's
`@ffi(trampoline, virtuals=[…])` entry for an imported class (tasks/90/N4-notes.md). The older
`extern class` production of amendment A9, which the specification no longer has, still parses and
is still refused with `E1010`.
