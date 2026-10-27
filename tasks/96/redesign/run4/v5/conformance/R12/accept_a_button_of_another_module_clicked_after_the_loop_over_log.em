#$ test: run-pass
#$ rules: R12, R17
#$ stdout: start
#$ stdout: 2
## Verdict 86, R12 (rules.md lines 665-670), R17: `widgets` neither declares nor imports `LOG`, and its
## button keeps this module's handler that pushes to `LOG`; `b.click()` is a change of `LOG` here, worked
## out from the whole program, so it runs after the loop over `LOG`. The nearest allowed program to
## reject_clicking_a_button_of_another_module_inside_a_loop_over_log.

import widgets

static mut LOG: Array[String] = []

fn main():
    LOG.push(String("start"))
    b = widgets.Button()
    b.on_click(fn(): LOG.push(String("clicked")))
    for entry in LOG:
        println(entry)
    b.click()
    println(LOG.len())
