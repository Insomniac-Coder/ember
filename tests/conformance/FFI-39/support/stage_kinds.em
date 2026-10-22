# A module of ../accept_a_class_derives_through_another_module.em: an Ember class deriving from a
# C++ class, which classes of another module derive from in turn, the header's module re-exported,
# and C++ running a stage.

pub import cpp "../stages.hpp" as stages

pub open class Timed(stages.Stage):
    override fn label(self) -> i32:
        return 5

pub type Labelled = Timed

pub fn run(stage: stages.Stage) -> i32:
    unsafe:  # SAFETY(ffi): a call on an object that lives for it
        return stages.run(stage)
