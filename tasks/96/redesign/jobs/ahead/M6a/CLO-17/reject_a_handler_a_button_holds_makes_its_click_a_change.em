# Clause CLO-17.1: "A call through a function value makes the changes, reads and ends of every
# function the program can put there (`[EFF-2]`) (...) So a library's `Button` holding a program's
# handler that pushes to `LOG` makes `b.click()` a change of `LOG` in that program, inside a loop
# over `LOG` `E3020` (`[BRW-18]`)."
#$ test: compile-fail
#$ rules: CLO-17, BRW-18, EFF-2, DIA-27
# `[BRW-18]`'s message for this case, with this file's line numbers. The handler is put in the list
# directly, so "the handler stored on line N" is that line under either reading of Q2.

static mut LOG: Array[String] = []

class Button:
    handlers: Array[fn()] = []

    fn click():
        for h in this.handlers:
            h()

fn log_click():
    LOG.push(owns String("click"))

fn main():
    b = Button()
    b.handlers.push(owns log_click)
    LOG.push(owns String("start"))
    for line in LOG:
        println(line)
        b.click()    #$ error[E3020]: `b.click()` (line 28) adds to `LOG` (through `click`, which runs the handler stored on line 24, `log_click`, which pushes to `LOG`) while this loop walks `LOG` (line 26); click after the loop: move line 28, `b.click()`, below the loop
