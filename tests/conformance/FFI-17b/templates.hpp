// The C++ side of the FFI-17b tests: a class template and function templates.
#pragma once

namespace geo {
template <class T>
struct Pair {
    T first;
    T second;
    T sum() const noexcept { return first + second; }
    void scale(T by) noexcept { first *= by; second *= by; }
    static Pair twin(T value) noexcept { return Pair{value, value}; }
    T larger() const { return first < second ? second : first; }
};

// Opaque: its object is behind a pointer, freed through its destructor.
template <class T>
class Stack {
public:
    Stack() noexcept {}
    ~Stack() {}
    void push(T value) { items_[count_++] = value; }
    T top() const noexcept { return items_[count_ - 1]; }
    int size() const noexcept { return count_; }
    static Stack* make() { return new Stack(); }

private:
    T items_[8] = {};
    int count_ = 0;
};
}  // namespace geo

template <class T>
T biggest(T a, T b) noexcept { return a < b ? b : a; }

template <class T>
T smallest(T a, T b) { return a < b ? a : b; }
