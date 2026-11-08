# K59-runtime-placement-beyond-verdicts

Question: May a field say runtime, so its methods' lines are checked while running (and a lambda parameter)?
Severity: medium; impact rank 59
Members: J1-11; primary J1-11
Why together: Single question.
Critic's notes: Add J1-17's held-back parts (a lambda's typed parameter, and runtime before `shares`/`shares mut` forms) as further lines of the same placement question, rather than writing them in as a lead fix (reroute).

## Finding J1-11
Rules: LT-48, LEX-15, GRM-42; file tasks/spec-0.9.9/parts/p03-grammar.md lines 75, 85, 181, 204, 327; kind inconsistency; severity medium
Quote from Hardened_11:
field_decl    := ["let"] IDENT ":" type ["from" (IDENT | "static")] ["guarded" "by" IDENT]

The finder's question:
Where does `runtime` go when the value whose kind is known only while running is a field (`this.renderer`), a
lambda's typed parameter, or a type with a word before it (`owns dyn Renderer`, `ref A`)? `runtime` is allowed
on parameters and variables only, 'before a `dyn` or class type'.

Program:
```
class Scene:
    renderer: owns dyn Renderer
    scratch: Array[u8] = []

    fn render():
        frame = this.renderer.draw(this.scratch)
        this.scratch.push(owns 1)   # E3021, help: `runtime` on the parameter or the variable: there is neither
        present(frame)

fn show(owns p: runtime owns dyn Picker):   # or `owns runtime dyn Picker`, or refused?
    pass
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.55)
Reason:
LT-48 (p07 705, 719-721) and the grammar give `runtime` to a parameter and a variable only (`param` p03 75,
`var_decl` p03 204), not to `field_decl` (p03 85). E3021's help (p17a 132) offers only 'runtime on the
parameter or the variable'. So a method whose unsafe line comes from a `dyn` field cannot have the check on
the field itself. The finder's claim that it cannot be written at all is not shown: BRW-12 (p07 196-199) makes
`r = this.renderer` a read-only view, not a hand-over. But whether a local of bare `dyn` type may view a field
(TYP-45 p04 413-414 gives `dyn I` alone to parameters, while LT-48's own example gives it to a local) is
itself unclear. The owner's verdict on variables gave as its reason that 'No function is needed only to carry
the word', and that reason bears on fields.

options:
[{'letter': 'a', 'summary': "`runtime` right after a field's colon, checking every method line that uses the
field", 'code': 'class Scene:\n    renderer: runtime owns dyn Renderer\n    scratch: Array[u8] = []\n\n    fn
render():\n        frame = this.renderer.draw(this.scratch)\n        this.scratch.push(owns 1)          # kind
of `renderer` compared here\n        println(frame)', 'outcome': "Compiles. With Gpu it prints the frame, and
with Software it stops at the push ('`this.scratch` changes while `frame` looks at it: `renderer` is a
`Software` ...'). E3021's help names the field as a third place for the word."}, {'letter': 'b', 'summary':
'No `runtime` on fields; the method names the field in a local with the word', 'code': '    fn render():\n
r: runtime dyn Renderer = this.renderer   # a read-only view of the field\n        frame =
r.draw(this.scratch)\n        this.scratch.push(owns 1)\n        println(frame)', 'outcome': 'Compiles, with
the same result as (a), and the field stays `renderer: owns dyn Renderer`. Without the local line, `render` is
E3021, whose help gives this local line.'}]

recommendation:
{'letter': 'a', 'reason': 'The reason for allowing the word on a variable, so that no extra code is written
only to carry it, holds for a field. The word then stands right after the colon of every declaration that
names a value: parameter, variable, field. A local made only to carry the word is a workaround for missing
syntax.', 'checklist': '12 (add the syntax rather than a workaround), 5, 6, 3 (one compare at each guarded
line, as for a parameter)'}

