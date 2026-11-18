#$ test: compile-fail
#$ rules: FN-4, GRM-41
# `[FN-4]` — a function's first line says its parameters, `owns this`, its result and `from`, in
# that order, and nothing about what the function changes or reads (`[GRM-41]`).

fn tick() changes counter:    #$ error[E0100]
    pass
