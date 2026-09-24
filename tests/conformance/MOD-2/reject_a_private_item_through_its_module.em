#$ test: compile-fail
#$ rules: MOD-2
# `[MOD-2]` — an item is private to its module unless marked. Calling a
# private function through the module that declares it, or importing it by
# name, is `E1052`, naming where it is declared; the public one is fine.

import support.tools
from support.tools import helper    #$ error[E1052]: `helper` is private to `support.tools`

fn main():
    println(tools.api())
    println(tools.helper())         #$ error[E1052]: `helper` is private to `support.tools`
