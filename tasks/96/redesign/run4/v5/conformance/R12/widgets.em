#$ test: parse-pass
#$ rules: BRW-18, CTL-2
## design rules: R12
# Module `widgets` of ./reject_clicking_a_button_of_another_module_inside_a_loop_over_log.em and
# ./accept_a_button_of_another_module_clicked_after_the_loop_over_log.em. It neither declares nor imports
# their `LOG`; what `click` changes is worked out from the functions a program stores in `handlers`,
# across the module's edge (R12, R17).

pub class Button:
    handlers: Array[fn()] = []

    pub fn on_click(owns f: fn()):
        this.handlers.push(f)

    pub fn click():
        for ref mut h in this.handlers:
            h()
