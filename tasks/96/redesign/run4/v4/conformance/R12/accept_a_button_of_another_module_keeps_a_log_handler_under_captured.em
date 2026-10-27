#$ test: run-pass
#$ rules: R12, R17
#$ stdout: 1
## Verdicts 40 and 76, R12 (rules.md lines 634-642): `widgets` neither declares nor imports `LOG`, so its
## button's `changes captured` covers it, filled in from the whole program; `b.click()` is then a change
## of `LOG` in this module, and `main` names it.

import widgets

static mut LOG: Array[String] = []

fn main() changes LOG:
    b = widgets.Button()
    b.on_click(fn() changes LOG: LOG.push(String("clicked")))
    b.click()
    println(LOG.len())
