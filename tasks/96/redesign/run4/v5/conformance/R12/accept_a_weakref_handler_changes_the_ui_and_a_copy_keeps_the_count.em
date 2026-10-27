#$ test: run-pass
#$ rules: R12, R3, R11, R13
#$ stdout: 0 1
## Verdict 86, R12 (rules.md lines 662-670, 717-720): the handler changes a `Ui` through its `weakref ui`
## with no word; `watch` only stores it (lines 735-737: "Storing a function value is not a change"); the
## count taken before the handlers run is a copy of `main`'s own (R8 (g): "A copy of your own ... is not
## restricted by anything"), so it stays 0 while the handler adds 1 to the real `ui.clicks`. AN-60's
## `watch` and `UiButton`; the nearest allowed program to
## reject_a_view_of_the_ui_across_a_handler_that_changes_it_through_a_weakref.

class Ui:
    clicks: int = 0

class UiButton:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

fn watch(ref mut ui: Ui, ref mut b: UiButton):
    bump_ui = fn() weakref ui:
        if ui exists as ref mut me:
            me.clicks += 1
    b.on_click(bump_ui)

fn main():
    ui = Ui()
    b = UiButton()
    watch(ref mut ui, ref mut b)
    before = copy of ui.clicks
    for ref mut h in b.handlers:
        h()
    println(before, ui.clicks)
