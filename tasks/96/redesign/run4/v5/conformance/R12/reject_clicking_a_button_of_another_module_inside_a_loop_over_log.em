#$ test: compile-fail
#$ rules: R12, R8, R17
#$ help: move line 21, `b.click()`, below the loop
## Verdict 86, R12 (rules.md lines 665-670): "a library's `Button` holding a program's handler that pushes
## to `LOG` makes `b.click()` a change of `LOG` in that program (inside a loop over `LOG`, E3020)"; R17
## (lines 938-939): the function values a package stores are counted wherever such a value is called.
## rules.md section 3's line 1065 `b.click()` message, in this program's lines (F-12: the line and where
## it goes). The nearest allowed program is
## accept_a_button_of_another_module_clicked_after_the_loop_over_log.

import widgets

static mut LOG: Array[String] = []

fn main():
    LOG.push(String("start"))
    b = widgets.Button()
    b.on_click(fn(): LOG.push(String("clicked")))
    for entry in LOG:
        println(entry)
        b.click()    #$ error[E3020]: `b.click()` (line 21) adds to `LOG` (through `click` in `widgets.em`, line 14, which runs the handler stored on line 18, which pushes to `LOG`) while this loop walks `LOG` (line 19); click after the loop: move line 21, `b.click()`, below the loop
