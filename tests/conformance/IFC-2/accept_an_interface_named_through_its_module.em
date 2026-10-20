#$ test: run-pass
#$ rules: IFC-2, MOD-2
#$ stdout: 7
# An implementation may name its interface through the module that declares it, as a type path
# names a type (D-750: `extend Blob implements core.Default` was `E1010`, "a `dyn` bound must
# name an interface").

import std.core

struct Blob:
    n: int

extend Blob implements core.Default:
    fn default() -> Blob:
        return Blob(n=7)

fn main():
    println(Blob.default().n)
