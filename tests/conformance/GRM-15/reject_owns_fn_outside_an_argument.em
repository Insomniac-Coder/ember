#$ test: compile-fail
#$ rules: GRM-15, OWN-9
# `[GRM-15]` (verdict 158) — `owns` before `fn` is the call's word only before an argument for an
# `owns` parameter; anywhere else it is `E0109`.

fn main():
    h = owns fn(): println(1)    #$ error[E0109]: `owns` is not written before `fn`
    h()
