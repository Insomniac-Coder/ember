#$ test: compile-fail
#$ rules: GRM-2
#$ help: handlers: Array[fn()] = []
## design rules: R12, R3
## Verdict 86, R12 (rules.md lines 662-665): "A function type says its parameters, its result, and
## whether any thread may run its functions; what they change is worked out from the program", and R3
## (lines 205-208): a function type says "nothing about what they change". A `changes` clause in a
## function type is E0100 (rules.md section 3, line 1051). The nearest allowed program is
## accept_named_functions_kept_in_a_list_of_fn.

static mut LOG: Array[String] = []

fn log_click():
    LOG.push(String("clicked"))

class Button:
    handlers: Array[fn() changes LOG] = []    #$ error[E0100]: `changes LOG` is not Ember: a function type says its parameters and result, and the compiler works out what the functions kept here change; write `handlers: Array[fn()] = []`

fn main():
    b = Button()
    b.handlers.push(log_click)
    for ref mut h in b.handlers:
        h()
    println(LOG.len())
