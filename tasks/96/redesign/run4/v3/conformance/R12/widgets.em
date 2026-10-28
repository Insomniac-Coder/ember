#$ test: parse-pass
#$ rules: R12
# Module `widgets` of ./accept_a_button_of_another_module_keeps_a_log_handler_under_captured.em and
# ./reject_clicking_a_button_of_another_module_inside_a_loop_over_log.em. It cannot see their `LOG`.

pub class Button:
    handlers: Array[fn() changes captured] = []

    pub fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    pub fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()
