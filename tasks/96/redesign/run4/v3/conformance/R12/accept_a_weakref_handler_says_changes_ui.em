#$ test: run-pass
#$ rules: R12, R11, R13
#$ stdout: 1
## R12 (rules.md lines 507-509, 583-588): the handler names `Ui`, the type of what it changes through its
## link, and is kept in a list whose type says so; `watch` only stores it, which is not a change, so it
## names nothing; `main` runs it and names `Ui`. AN-60's `watch` and `UiButton`.

class Ui:
    clicks: int = 0

class UiButton:
    handlers: Array[fn() changes Ui] = []

    fn on_click(owns f: fn() changes Ui) changes handlers:
        this.handlers.push(f)

fn watch(ref mut ui: Ui, ref mut b: UiButton):
    bump_ui = fn() with weakref ui changes Ui:
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
