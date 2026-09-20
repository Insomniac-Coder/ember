#$ test: compile-pass
#$ rules: WK-5, WK-6, WK-7, TST-14
#$ profiles: debug, release, shipping
#$ warning[L3001]: potential reference cycle

# Two declared strong field edges form a visible ownership cycle. The warning is
# diagnostic-only: no runtime graph needs to be allocated for this declaration.
class Parent:
    child: Child

class Child:
    parent: Parent
