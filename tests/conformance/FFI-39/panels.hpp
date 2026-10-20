// The C++ side of the FFI-39 reject tests: a class with no default constructor, and one with no
// virtual destructor.
#pragma once

class Panel {
public:
    explicit Panel(int width) noexcept : width_(width) {}
    virtual ~Panel() {}
    virtual int width() const noexcept { return width_; }

private:
    int width_;
};

class Flat {
public:
    Flat() noexcept {}
    virtual int depth() const noexcept { return 0; }
};
