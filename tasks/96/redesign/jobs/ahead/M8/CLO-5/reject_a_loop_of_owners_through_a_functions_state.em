# Clause CLO-5.2: "A loop of owners through function values' state is refused as every loop of owners
# is (`[WK-1]`)"
#$ test: compile-fail
#$ rules: CLO-5, WK-1
#$ error[E3032]
# The `Ui` owns the handler, whose state owns the `Ui`. Code only, claimed for the file. Needs M7's
# loop-of-owners check (`E3032`).

class Ui:
    handlers: Array[fn()] = []

fn main():
    ui = shares mut Ui()
    ui.handlers.push(owns fn() shares mut ui: println(ui.handlers.len()))
