#$ test: compile-fail
#$ rules: R12, R17
#$ help: handlers: Array[fn() changes captured, LOG] = []
## Verdict 76, R12 (rules.md lines 631-638): "can see by name" means declared or imported in the file where
## the type is written. `log_widgets` imports `LOG`, so its `captured` does not cover `LOG`, and a
## `fn() changes LOG` does not fit its button's list (E2113); the same handler fits `widgets`' button, whose
## file does not import `LOG` (accept_a_log_handler_in_a_button_of_a_module_that_does_not_import_log).
## rules.md section 3's line 1040 second message, in this program's names.

import log_widgets
from logbook import LOG

fn main() changes LOG:
    b = log_widgets.Button()
    b.on_click(fn() changes LOG: LOG.push(String("clicked")))    #$ error[E2113]: this function changes `LOG` (`fn() changes LOG`), and `on_click` takes `fn() changes captured`, functions that change one variable each captured; `log_widgets` imports `LOG` (`from logbook import LOG`), so its list names it: in `log_widgets.em` write `handlers: Array[fn() changes captured, LOG] = []` and `pub fn on_click(owns f: fn() changes captured, LOG) changes handlers:`
    b.click()
    println(LOG.len())
