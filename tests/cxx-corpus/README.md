# The C++ corpus — `[CXX-1]`

The conformance corpus of Annex C: real C++ headers and at least one third-party header-only
library, each with its expected outcome recorded and its generated thunks committed and diffed,
run under MSVC and Clang with the debug and release runtimes and RTTI on and off.

| Entry | What | Import |
|---|---|---|
| `nlohmann_json/` | nlohmann/json 3.11.3 (`third_party/nlohmann`, MIT, unmodified) | the whole header, `instantiate = ["nlohmann::basic_json<>"]` |

Each entry holds its corpus header and, per platform (`windows`, `linux`):

- `<entry>.<platform>.outcomes` — `ember bind --outcomes`: one line per declaration, in the order
  the header declares them: `value` (it crosses by value), `opaque` (a type held behind a
  pointer), `overlay` (a function whose pointers' facts only an overlay states) or `refused <code>`
  (not imported, with the diagnostic the import reports).
- `<entry>.<platform>.thunks.cpp` — `ember bind --emit-cpp`: the thunk file the import compiles.

The two platforms differ only where they must: `size_t` is `unsigned long long` on Windows and
`unsigned long` on Linux, and the header defines other compiler-detection macros for each.

`program/` is a program that uses the library: `json_api.hpp` is the few lines of C++ that reach it
by types Ember has, `json_api.overlay.em` the facts the headers cannot state (each text a
`Span[u8]`, each call safe), `main.em` the program, `expected.txt` what it prints.

## The test

`the_cpp_corpus_binds_as_recorded` (`compiler/ember_driver/tests/package.rs`) runs each
configuration under the C++ compiler the suite runs with — the CI matrix runs MSVC, clang-cl, GCC
and Clang:

| Compiler | Release runtime | Debug runtime | RTTI off |
|---|---|---|---|
| cl, clang-cl | `/MD` | `/MDd` | `/GR-` |
| GCC, Clang | `-U_GLIBCXX_DEBUG` | `-D_GLIBCXX_DEBUG` | `-fno-rtti` |

In each, the outcomes and thunks must be the recorded ones (where the header defines a macro may
follow the configuration; what each declaration becomes may not), and the program must build and
print `expected.txt`. Two configurations that disagree in one target must fail with `E9020`: a
class whose layout follows RTTI, and the corpus imported under two runtimes.

## Recording a meant change

When the importer's output changes on purpose, write the records again on both platforms and read
the diff before committing it:

```bash
python tests/cxx-corpus/record.py
```

(On Linux, give it the path of a Linux build of `ember`.)
