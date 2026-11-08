# K16-unsafe-import-cpp

Question: Does import cpp need unsafe, given that the header's code runs before main?
Severity: medium; impact rank 16
Members: J8-11; primary J8-11
Why together: Single question.
Critic's notes: TCB-4's list gains `unsafe import cpp` under A.

## Finding J8-11
Rules: FFI-51, FFI-17, FFI-3, TCB-4; file tasks/spec-0.9.9/parts/p16-ffi.md lines 133-142; p21-annex-c.md 8-13; kind inconsistency; severity medium
Quote from Hardened_11:
before `main` too; say that ... An
  import of a header alone needs no word: its functions run only when they are called, and those
  calls are fenced by `[FFI-2]`.

The finder's question:
Does `import cpp` of a header whose contents the thunk translation unit compiles into the program need
`unsafe`, as `unsafe import c` does for C compiled into the program? Examples are a header-only library, an
`inline` variable with a dynamic initializer, or `<iostream>`'s static initializer, all of which run before
`main`.

Program:
```
// json.hpp (header-only): inline std::string g_default = load_default();  // runs before main
import cpp "json.hpp" with (project="engine") as js     # no `unsafe`, no `# SAFETY:`?

fn main():
    println("start")
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.55)
Reason:
FFI-51 (p16-ffi.md 133-141) requires `unsafe import c` for an import that "compiles C into the program ...
runs C code that nothing checks, a constructor in it even before `main`". It exempts "an import of a header
alone" because "its functions run only when they are called". For C++, FFI-3 and FFI-17 (p21-annex-c.md 8-12)
compile a thunk translation unit that includes the header with the project's compiler. That unit instantiates
the header's inline variables and namespace-scope objects, whose dynamic initialisers run before `main`
uncalled, which is exactly FFI-51's reason. Header-only libraries (CXX-1) compile all their code this way. The
annex never mentions `unsafe import cpp`, TCB-4 (p21-annex-c.md 213) lists only `unsafe import c`, and no
verdict covers it.

options:
[{'letter': 'A', 'summary': 'Every C++ import is written `unsafe import cpp` with a `# SAFETY:` note, because
Ember always compiles the header into the program.', 'code': '# SAFETY: config.hpp is ours; load_default_path
only reads config/default.txt\nunsafe import cpp "config.hpp" with (project="engine") as cfg', 'outcome':
'This compiles. The plain `import cpp` is E3100, with the line above as its help. `ember tcb` lists the
import.'}, {'letter': 'B', 'summary': '`unsafe import cpp` only when the header defines something that runs
uncalled (a namespace-scope, inline or static-member variable with a dynamic initialiser). The importer finds
these in the header.', 'code': '# SAFETY: config.hpp is ours; load_default_path only reads
config/default.txt\nunsafe import cpp "config.hpp" with (project="engine") as cfg\nimport cpp "vec3.hpp" with
(project="engine") as v3     # no dynamic initialiser: no word', 'outcome': '`config.hpp` without `unsafe` is
E3100 naming `g_default_path`, and `vec3.hpp` needs nothing. A header that includes `<iostream>` (whose
standard library defines a static initialiser object) needs `unsafe` too.'}, {'letter': 'C', 'summary': 'As
today: a C++ header needs no word, as a C header does not.', 'code': 'import cpp "config.hpp" with
(project="engine") as cfg', 'outcome': 'Compiles, and `load_default_path()` runs before `main` with nothing
written about it.'}]

recommendation:
{'letter': 'A', 'reason': "By FFI-51's own test, a C++ import compiles code into the program, and C++ code can
run before `main` from nearly any header (`<iostream>` included), so a per-header test would hold for almost
every import. One word on every C++ import says plainly that its code is vouched for.", 'checklist': "5 and 6
(FFI-51's rule wherever code is compiled in), 13 (no special detection path), 2, 11 (`unsafe import` reused)"}

