#$ test: compile-fail
#$ rules: LEX-24, TYP-8

const TOO_LOW: i8 = -129i8    #$ error[E2010]: does not fit
const UNSIGNED: u8 = -1    #$ error[E2010]: does not fit
