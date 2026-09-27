#$ test: compile-fail
#$ rules: FFI-26, MNG-2
#$ profiles: debug
#$ error[E0104]: C symbol `api_tick` is defined by more than one Ember function
#$ error[E0104]: C symbol `plain_tick` is defined by more than one Ember function

@export("api_tick")
fn first() -> i32:
    return 1

@export("api_tick")
fn second() -> i32:
    return 2

pub extern "C" fn plain_tick() -> i32:
    return 3

@export("plain_tick")
fn alias_tick() -> i32:
    return 4

fn main():
    pass
