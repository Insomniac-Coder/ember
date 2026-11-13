// The C++ side of accept_adopt_ends_through_the_unique_ptrs_deleter.em: a class only its friends make,
// whose `std::unique_ptr` names the deleter `Recycler`.
#pragma once
#include <memory>

inline int recycled = 0;

class Gizmo;

struct Recycler {
    void operator()(Gizmo* gizmo) const noexcept;
};

class Gizmo {
public:
    ~Gizmo() {}
    int value() const noexcept { return value_; }
    friend Gizmo* raw_gizmo(int value) noexcept;
    friend std::unique_ptr<Gizmo, Recycler> make_gizmo(int value) noexcept;

private:
    explicit Gizmo(int value) noexcept : value_(value) {}
    int value_;
};

inline void Recycler::operator()(Gizmo* gizmo) const noexcept { ++recycled; delete gizmo; }
inline Gizmo* raw_gizmo(int value) noexcept { return new Gizmo(value); }
inline std::unique_ptr<Gizmo, Recycler> make_gizmo(int value) noexcept { return std::unique_ptr<Gizmo, Recycler>(new Gizmo(value)); }
inline int recycled_count() noexcept { return recycled; }
