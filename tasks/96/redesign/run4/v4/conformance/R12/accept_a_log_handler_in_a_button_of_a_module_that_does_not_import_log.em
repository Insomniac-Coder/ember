#$ test: run-pass
#$ rules: R12, R17
#$ stdout: 1
## Verdict 76, R12 (rules.md lines 631-642): `widgets` neither declares nor imports `LOG`, so its button's
## `captured` covers `LOG`, filled in from the whole program, and `b.click()` is a change of `LOG` that
## `main` names. The nearest allowed program to
## reject_a_log_handler_in_a_button_of_a_module_that_imports_log.

import widgets
from logbook import LOG

fn main() changes LOG:
    b = widgets.Button()
    b.on_click(fn() changes LOG: LOG.push(String("clicked")))
    b.click()
    println(LOG.len())
