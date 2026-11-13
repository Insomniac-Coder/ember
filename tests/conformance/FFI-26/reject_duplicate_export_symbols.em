#$ test: compile-fail
#$ rules: FFI-26, MNG-2
#$ profiles: debug
#$ error[E0104]: C symbol `api_tick` is defined by more than one Ember function
#$ error[E0104]: C symbol `plain_tick` is defined by more than one Ember function

# SAFETY: nothing else in the process defines this name
@unsafe(export("api_tick"))
fn first() -> i32:
    return 1

# SAFETY: nothing else in the process defines this name
@unsafe(export("api_tick"))
fn second() -> i32:
    return 2

pub extern "C" fn plain_tick() -> i32:
    return 3

# SAFETY: nothing else in the process defines this name
@unsafe(export("plain_tick"))
fn alias_tick() -> i32:
    return 4

fn main():
    pass
