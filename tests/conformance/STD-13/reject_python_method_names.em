#$ test: compile-fail
#$ rules: STD-13, DIA-1
#$ error[E2072]: `Array[str]` has no method named `append`
#$ help: Ember's name for it is `push`
#$ error[E2072]: `str` has no method named `strip`
#$ help: Ember's name for it is `trim`
#$ error[E2072]: `str` has no method named `upper`
#$ help: Ember's name for it is `to_upper`
#$ error[E2072]: `str` has no method named `startswith`
#$ help: Ember's name for it is `starts_with`
#$ error[E2072]: `str` has no method named `splitlines`
#$ help: Ember's name for it is `lines`
# `[STD-13]` — where Python's name for a method differs, there is no second name: the call is
# `E2072`, whose help names Ember's method.

fn main():
    names = ["ann"]
    names.append("bob")
    s = " Hi "
    t = s.strip()
    u = s.upper()
    ok = s.startswith("H")
    rows = "a b".splitlines()
