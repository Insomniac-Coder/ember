#$ test: parse-pass
#$ rules: R12
# Module `widgets` of ./accept_a_button_of_another_module_keeps_a_log_handler_under_captured.em,
# ./reject_clicking_a_button_of_another_module_inside_a_loop_over_log.em and
# ./accept_a_log_handler_in_a_button_of_a_module_that_does_not_import_log.em. It neither declares nor
# imports their `LOG`, so its `captured` covers it (verdict 76).

pub class Button:
    handlers: Array[fn() changes captured] = []

    pub fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    pub fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()
