#$ test: compile-fail
#$ rules: R12, R11, R13
#$ help: bump_ui = fn() with weakref ui changes Ui:
## R12 (rules.md lines 507-509): a function type names "the types of the objects it may change through a
## link it holds", so a handler that changes a `Ui` through `with weakref ui` says `changes Ui`.
## AN-60 part 6's E2105 message, word for word.

class Ui:
    clicks: int = 0

class UiButton:
    handlers: Array[fn() changes Ui] = []

    fn on_click(owns f: fn() changes Ui) changes handlers:
        this.handlers.push(f)

fn watch(ref mut ui: Ui, ref mut b: UiButton):
    bump_ui = fn() with weakref ui:    #$ error[E2105]: this function changes `me`, a `Ui` reached through its `weakref ui`; say so on its first line: `fn() with weakref ui changes Ui:`
        if ui exists as ref mut me:
            me.clicks += 1
    b.on_click(bump_ui)

fn main() changes Ui:
    ui = Ui()
    b = UiButton()
    watch(ref mut ui, ref mut b)
    for ref mut h in b.handlers:
        h()
    println(ui.clicks)
