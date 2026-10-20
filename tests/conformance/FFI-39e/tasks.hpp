// The C++ side of the FFI-39e tests: a class whose base implementation throws, and C++ that calls a
// virtual Ember overrides.
#pragma once
#include <stdexcept>

class Task {
public:
    Task() noexcept {}
    virtual ~Task() {}
    virtual void act() { throw std::runtime_error("the task refused"); }
    virtual int step() noexcept { return 1; }
};

inline int run(Task& task) noexcept { return task.step(); }
