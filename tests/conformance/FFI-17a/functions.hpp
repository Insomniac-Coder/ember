// The C++ side of accept_a_std_function_crosses_as_a_callable.em: `std::function` results, and a
// guard each one's target holds, counted while it lives.
#pragma once
#include <functional>

inline int live = 0;

struct Guard {
    Guard() noexcept { ++live; }
    Guard(const Guard&) noexcept { ++live; }
    Guard(Guard&&) noexcept { ++live; }
    ~Guard() { --live; }
};

inline std::function<int(int)> make_adder(int n) {
    Guard guard;
    return [n, guard](int x) { return x + n; };
}
inline std::function<double(double, double)> make_mixer(double weight) {
    return [weight](double a, double b) { return a * weight + b * (1 - weight); };
}
inline std::function<int(int)> make_kept(int n) {
    Guard guard;
    return [n, guard](int x) { return x * n; };
}
inline int live_guards() noexcept { return live; }
