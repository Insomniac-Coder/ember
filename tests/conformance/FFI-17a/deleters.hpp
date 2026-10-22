// The C++ side of accept_a_unique_ptr_with_its_own_deleter.em: `std::unique_ptr`s whose deleter is
// a class of the header, and one whose deleter holds state.
#pragma once
#include <memory>

inline int recycled = 0;

class Widget {
public:
    explicit Widget(int value) noexcept : value_(value) {}
    ~Widget() {}
    int value() const noexcept { return value_; }

private:
    int value_;
};

struct Recycler {
    void operator()(Widget* widget) const noexcept {
        ++recycled;
        delete widget;
    }
};

inline std::unique_ptr<Widget, Recycler> make_widget(int value) noexcept { return std::unique_ptr<Widget, Recycler>(new Widget(value)); }
inline int take_widget(std::unique_ptr<Widget, Recycler> widget) noexcept { return widget->value(); }
inline int recycled_count() noexcept { return recycled; }

inline void free_widget(Widget* widget) noexcept { delete widget; }
inline std::unique_ptr<Widget, void (*)(Widget*)> make_tracked(int value) noexcept {
    return std::unique_ptr<Widget, void (*)(Widget*)>(new Widget(value), free_widget);
}
