// The C++ side of the FFI-17c and FFI-17d tests: a class Ember classes derive from, whose
// destructor records itself, and C++ that keeps one node and deletes it.
#pragma once

inline int recorded_events = 0;
inline void record(int event) noexcept { recorded_events = recorded_events * 10 + event; }
inline int recorded() noexcept { return recorded_events; }

class Node {
public:
    Node() noexcept {}
    virtual ~Node() { record(3); }
    virtual int value() const noexcept { return 0; }
};

inline Node* kept_node = nullptr;
inline void keep(Node* node) noexcept { kept_node = node; }
inline int kept_value() noexcept { return kept_node != nullptr ? kept_node->value() : -1; }
inline void release() noexcept {
    delete kept_node;
    kept_node = nullptr;
}
