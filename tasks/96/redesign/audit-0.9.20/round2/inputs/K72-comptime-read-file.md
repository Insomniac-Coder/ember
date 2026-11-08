# K72-comptime-read-file

Question: What does comptime.read_file give, and can its function run at run time?
Severity: low; impact rank 72
Members: J6-05; primary J6-05
Why together: Single question.
Critic's notes: 

## Finding J6-05
Rules: CT-2, CT-1, CT-6; file tasks/spec-0.9.9/parts/p14-comptime.md lines 47-49; kind undecided_case; severity low
Quote from Hardened_11:
Two inputs are provided: `comptime.read_file(path)` (relative to the package root, and only inside the
  package directory, `E6010` otherwise, so a dependency's build cannot read the builder's files) and
  `comptime.env(name)`. Both are recorded as build inputs.

The finder's question:
What does `comptime.read_file` return, which effect does it carry (CT-1 bars Io at compile time), and what
happens when the function that calls it runs at run time?

Program:
```
fn sky_shader() -> String:
    return comptime.read_file("shaders/sky.wgsl")

const SKY_LEN: int = comptime(sky_shader()).len()

fn main():
    src = sky_shader()      # run time: read when compiling, read now, or E6010?
    println(src.len())
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.65)
Reason:
CT-2 (p14 47-49) names `comptime.read_file(path)` and `comptime.env(name)` but gives neither a type nor an
effect, and says nothing about a missing file or about the function that calls one being run at run time. CT-1
bars Io at compile time, so if read_file carries Io it cannot run there. CT-4 (p14 61-62) only says contents
are build inputs. Nothing in std/src defines them. No verdict.

options:
[{'letter': 'A', 'summary': "Always read while compiling, wherever written, like C's #embed", 'code': 'fn
sky_shader() -> StrView:\n    return comptime.read_file("shaders/sky.wgsl")', 'outcome': "Compiles;
`read_file` gives a `StrView` of the file's text kept in the program, so `sky_shader()` works while compiling
and while running, with no file read at run time and no Io. `comptime.env(name)` gives an `Option[StrView]`. A
missing file, text that is not UTF-8, or a path not known when compiling is a compile error."}, {'letter':
'B', 'summary': 'Only inside what compile time evaluates', 'code': 'fn main():\n    src = sky_shader()',
'outcome': 'E6010 at `src = sky_shader()`: "`sky_shader` reads a file with `comptime.read_file` (line 2),
which only runs while compiling; write `src = comptime(sky_shader())`". The `const` line compiles.'}]

recommendation:
{'letter': 'A', 'reason': 'The program means to embed the file, and A does that on every path with no run-time
cost; the result is a read-only view of constant data, which is how a `comptime(e)` result is already given.',
'checklist': "1, 2, 3, 6 (CT-5's read-only view), 7, 9"}

