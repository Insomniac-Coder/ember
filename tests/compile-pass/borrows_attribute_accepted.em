#$ test: compile-pass
#$ rules: LT-1a

## The form the rule is written for: two view-typed parameters, and the
## attribute saying which one the result points into, so the caller may keep
## using the other.

fn pick(ref a: String, ref b: String) -> ref String from a:
    return a

fn main():
    x: String = "1".to_string()
    y: String = "2".to_string()
    p: ref String = ref x
    q: ref String = ref y
    println(pick(p, q))
