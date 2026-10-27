#$ test: compile-fail
#$ rules: R12, R8, R11, R13
#$ help: move line 33, `println(before)`, above line 31
## Verdict 86, R12 (rules.md lines 717-720): "A call through a function value makes the changes of the
## functions it may run. R8 then applies: ... no read-only view of what they change ... lives across the
## call". The handler changes a `Ui` through its `weakref ui`, which the compiler works out from its body,
## with no word; `before` reads `ui.clicks` across the call (R8 (b), (g): one path goes through a link;
## E3021 at the change, rules.md section 3's line 1043 shape).
## The nearest allowed program is accept_a_weakref_handler_changes_the_ui_and_a_copy_keeps_the_count.

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
    before = ref ui.clicks
    for ref mut h in b.handlers:
        h()    #$ error[E3021]: `h()` (line 32) may change `ui.clicks` (it may run `bump_ui`, stored on line 24, which adds to `me.clicks` through its `weakref ui`, line 23), and `before` reads `ui.clicks` (line 30) and is used on line 33; read it before the handlers run: move line 33, `println(before)`, above line 31, or keep a copy of your own: `before = copy of ui.clicks` in place of line 30
    println(before)
