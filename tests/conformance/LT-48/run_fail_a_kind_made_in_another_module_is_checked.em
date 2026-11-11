#$ test: run-fail
#$ rules: LT-48, BLD-4
#$ stdin: y
#$ panics: `xs` changes while `r` looks at it: `p` is a `support.pickers.Yours`, whose `pick` gives back the list it was handed
# `[LT-48]` (verdict 152) — the check of a `runtime` parameter's kind tells the kind wherever the value
# was made: here the `Yours` is boxed in this module and checked in `support.pickers`, each with a unit
# (and a copy of the table) of its own.

from support.pickers import Picker, Mine, Yours, Shower

fn main():
    s = Shower(n=1)
    p: owns dyn Picker = owns Mine(items=[9]) if input() == "m" else owns Yours(items=[9])
    s.show(p)
