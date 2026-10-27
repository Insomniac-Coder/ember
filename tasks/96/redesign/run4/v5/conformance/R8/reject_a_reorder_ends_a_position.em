#$ test: compile-fail
#$ rules: R8
#$ help: top = ref mut scores[0]
## R8 (a): a reorder (`sort`) may put another element in a position's place, so it ends the position
## (E3060 at a later use), for a number as for any element.

fn main():
    scores = [3, 1, 2]
    top = ref mut scores[0]
    scores.sort()
    top += 10    #$ error[E3060]: `scores.sort()` (line 10) may put another number in the place `top` names (line 9), and `top` is used on line 11; use `top` before line 10, or name the number again after it: `top = ref mut scores[0]`
