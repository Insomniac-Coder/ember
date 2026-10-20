/* The C side of accept_anonymous_members_read_through.em: anonymous structs, one inside another. */
#ifndef ANONYMOUS_H
#define ANONYMOUS_H
struct Packet {
    unsigned char kind;
    struct {
        short x;
        short y;
        struct {
            int depth;
        };
    };
    double weight;
};
static inline struct Packet packet_make(void) {
    struct Packet p = {0};
    p.kind = 1; p.x = 2; p.y = 3; p.depth = 4; p.weight = 5.0;
    return p;
}
static inline int packet_sum(struct Packet p) { return p.kind + p.x + p.y + p.depth + (int)p.weight; }
#endif
