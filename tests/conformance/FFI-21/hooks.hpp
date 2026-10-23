// The C++ side of accept_cpp_callback_contracts.em: a callback whose user data is the second of two
// pointers (ODR-245), hooks C++ may be given none of (ODR-253), and hooks kept until `cancel` with
// the id `watch` returned (ODR-246).
#pragma once
#include <vector>

using Pick = int (*)(void* left, void* right);
inline int pick_with(Pick pick, void* right) noexcept {
    static int marker = 0;
    return pick(&marker, right);
}

using Hook = int (*)(void* data, int value);

// -1 when no hook is given; `given_both()` says whether the hook and its data came together.
template <int Registry = 0> int& last_given() {
    static int given = 0;
    return given;
}
inline int maybe_hook(Hook hook, void* data, int value) noexcept {
    last_given<>() = (hook == nullptr) == (data == nullptr) ? 1 : 0;
    return hook != nullptr ? hook(data, value) : -1;
}
inline int given_both() noexcept { return last_given<>(); }

struct Kept {
    Hook hook;
    void* data;
};
template <int Registry = 0> std::vector<Kept>& kept() {
    static std::vector<Kept> all(8, Kept{nullptr, nullptr});
    return all;
}
// Keeps `hook` until `cancel(id)`; a null one is kept nowhere and its id is 0.
inline int watch(Hook hook, void* data) noexcept {
    if (hook == nullptr) return 0;
    auto& all = kept<>();
    for (int slot = 0; slot < static_cast<int>(all.size()); ++slot) {
        if (all[slot].hook == nullptr) {
            all[slot] = Kept{hook, data};
            return slot + 1;
        }
    }
    return 0;
}
inline void cancel(int id) noexcept {
    auto& all = kept<>();
    if (id >= 1 && id <= static_cast<int>(all.size())) all[id - 1] = Kept{nullptr, nullptr};
}
inline int poke(int value) noexcept {
    int sum = 0;
    for (auto& entry : kept<>()) {
        if (entry.hook != nullptr) sum += entry.hook(entry.data, value);
    }
    return sum;
}
inline int watching() noexcept {
    int count = 0;
    for (auto& entry : kept<>()) count += entry.hook != nullptr;
    return count;
}

class Panel {
public:
    Panel() noexcept {}
    ~Panel() {}
    // Calls `hook(data, size)` when there is a hook; -1 when there is none.
    int on_resize(Hook hook, void* data, int size) const noexcept { return hook != nullptr ? hook(data, size) : -1; }
};
