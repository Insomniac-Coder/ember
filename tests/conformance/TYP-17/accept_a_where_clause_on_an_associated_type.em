#$ test: run-pass
#$ rules: TYP-17, IFC-4
#$ stdout: 1
#$ 2
# `[TYP-17]` (D-676) — a `where` clause on a parameter's associated type bounds it, as one on the
# parameter itself does (D-644): `where I.Item: Display` lets the body format an item. The clause was
# read nowhere, so the body was E2040 and no call had to meet it.

fn show_all[I: Iterator](owned it: I) where I.Item: Display:
    for x in it:
        println(f"{x}")

fn main():
    show_all([1, 2].into_iter())
