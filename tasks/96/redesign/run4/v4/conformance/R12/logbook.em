#$ test: parse-pass
#$ rules: R12
# Module `logbook` of ./reject_a_log_handler_in_a_button_of_a_module_that_imports_log.em and
# ./accept_a_log_handler_in_a_button_of_a_module_that_does_not_import_log.em. It declares `LOG`.

pub static mut LOG: Array[String] = []
