// The C++ side of the FFI-48 tests: inheritance the importer does not model.
#pragma once

struct Named { int id = 0; };
struct Sized { int size = 0; };

class Both : public Named, public Sized {
public:
    Both() noexcept {}
};

class Shared : public virtual Named {
public:
    Shared() noexcept {}
};

inline int answer() noexcept { return 42; }
