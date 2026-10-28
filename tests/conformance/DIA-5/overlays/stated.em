# The overlay of ../accept_an_import_with_its_facts_stated_reports_nothing.em and
# ../reject_an_error_names_the_header_and_the_c_declaration.em: every fact the calls need.

unsafe overlay c "labels.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn label_length(text: borrowed nul_terminated)
    fn label_twice(value)
