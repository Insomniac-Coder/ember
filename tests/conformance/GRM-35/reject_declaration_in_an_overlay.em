#$ test: parse-fail
#$ rules: GRM-35
#$ error[E0100]: expected an overlay item
# `[GRM-35]` — an overlay holds `fn`, `type`, `enum`, `rename`, `hide` and
# `extend` items; a declaration of another kind is not one.

unsafe overlay c "string.h":
    const LIMIT = 4

fn main():
    pass
