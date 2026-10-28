#$ test: parse-pass
#$ rules: R12
# Module `log_widgets` of ./reject_a_log_handler_in_a_button_of_a_module_that_imports_log.em. It imports
# `LOG` (its button writes a line for each click), so `LOG` is visible here and is written by its name;
# its `captured` does not cover `LOG` (verdict 76).

from logbook import LOG

pub class Button:
    handlers: Array[fn() changes captured] = []

    pub fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    pub fn click() changes handlers, captured, LOG:
        LOG.push(String("click"))
        for ref mut h in this.handlers:
            h()
