#$ test: run-fail
#$ profiles: debug, release, shipping
#$ rules: CELL-9, CELL-5, PRF-1
# `[CELL-9]` keeps runtime borrow checks in every profile. The package-level
# `exclusivity = "unchecked"` switch applies only to classes and is not yet a
# reachable compiler feature; this test pins the profile-independent part of
# the rule without pretending that absent package syntax is conformance.

fn main():
    cell: RefCell[i32] = RefCell(1)
    with _mutable = cell.borrow_mut():
        with _shared = cell.borrow():
            println(0)
#$ panics: RefCell already mutably borrowed (borrowed at
