#$ test: run-pass
#$ rules: MOD-8
#$ stdout: words loud
## `[MOD-8]` — both globs bind `describe`, but the explicit import names one,
## and an explicit import or a local declaration shadows a glob: `shout` is
## this module's own.

from support.shapes import *
from support.words import *
from support.words import describe

fn shout(s: str) -> str:
    return "loud"

fn main():
    println(describe(), shout("quiet"))
