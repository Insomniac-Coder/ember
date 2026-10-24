/* matches.c: scoped tasks make result objects that keep a handle to a parent object (A15, a residue).
 *
 * Python:  matches = list(ex.map(lambda p: Match(p, len(p.name)), players))
 * Rust:    Vec<Match<'a>> with `player: &'a Player` (r04_results_borrow_inputs.rs): no count at all.
 * Ember (proposed, as written): refused, two tasks write "the count of some Player".
 *
 * 1,048,576 parent objects (lean, 32 bytes), shuffled handles, 8 tasks; each task makes one 24-byte Match
 * per handle in its part ({player, score}), keeps it in its own list; after the join the parent concatenates
 * the lists in spawn order (a list reduction). 16 rounds; every round frees the matches (releasing players).
 *
 *   -DMODEL=0  plain C: no counts
 *   -DMODEL=1  the task retains the player with an atomic increment (Rust's Arc::clone; the compiler emits it
 *              because the handle is a parent's)
 *   -DMODEL=2  deferred: the task stores the handle without a count; the parent, at the join, retains each
 *              stored player with a plain increment while it appends the task's list (one thread)
 *   -DHOT=1    with any model: every match points at one of 8 players (heavy sharing of a few objects)
 * Build: "C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n [-DHOT=1] matches.c -o bin/matches_n.exe */
#include "../../../../redesign/bench/harness.h"

#ifndef MODEL
#define MODEL 0
#endif
#ifndef HOT
#define HOT 0
#endif
#define N (1u << 20)
#define TASKS 8
#define ROUNDS 16

typedef struct { uint32_t strong, type_index; int32_t hp, x; int64_t more[2]; } Player;
typedef struct { Player *player; int64_t score; int64_t pad; } Match;
static Player **players;
static Match **out;                 /* the reduction's result: N handles */
static Match **part_lists[TASKS];

typedef struct { size_t lo, hi; int k; } Part;
static DWORD WINAPI work(LPVOID arg) {
    Part *pt = (Part *)arg;
    Match **mine = part_lists[pt->k];
    size_t n = 0;
    for (size_t i = pt->lo; i < pt->hi; i++) {
        Player *p = players[HOT ? (i & 7) : i];
        Match *m = (Match *)malloc(sizeof(Match));
        if (!m) abort();
        m->player = p; m->score = p->hp & 7; m->pad = 0;
#if MODEL == 1
        __atomic_fetch_add(&p->strong, 1u, __ATOMIC_RELAXED);
#endif
        mine[n++] = m;
    }
    return 0;
}

int main(void) {
    h_start();
    h_seed(3);
    Player *store = (Player *)calloc(N, sizeof(Player));
    players = (Player **)malloc(N * sizeof(Player *));
    out = (Match **)malloc(N * sizeof(Match *));
    if (!store || !players || !out) abort();
    for (int k = 0; k < TASKS; k++) { part_lists[k] = (Match **)malloc((N / TASKS) * sizeof(Match *)); if (!part_lists[k]) abort(); }
    for (uint32_t i = 0; i < N; i++) { store[i].strong = 1; store[i].hp = (int32_t)(i * 7); players[i] = &store[i]; }
    for (uint32_t i = N - 1; i > 0; i--) { uint32_t j = h_rand() % (i + 1); Player *t = players[i]; players[i] = players[j]; players[j] = t; }
    h_start();
    uint64_t sum = 0;
    for (int round = 0; round < ROUNDS; round++) {
        Part parts[TASKS];
        HANDLE th[TASKS];
        for (int k = 0; k < TASKS; k++) {
            parts[k].lo = (size_t)k * (N / TASKS); parts[k].hi = parts[k].lo + N / TASKS; parts[k].k = k;
            th[k] = CreateThread(NULL, 0, work, &parts[k], 0, NULL);
            if (!th[k]) abort();
        }
        WaitForMultipleObjects(TASKS, th, TRUE, INFINITE);
        for (int k = 0; k < TASKS; k++) CloseHandle(th[k]);
        size_t at = 0;                                     /* the join: append each task's list in spawn order */
        for (int k = 0; k < TASKS; k++) {
            Match **l = part_lists[k];
            for (size_t i = 0; i < N / TASKS; i++) {
#if MODEL == 2
                l[i]->player->strong += 1;                 /* the deferred retain */
#endif
                out[at++] = l[i];
            }
        }
        for (size_t i = 0; i < N; i++) {                   /* use, then drop the results */
            Match *m = out[i];
            sum += (uint64_t)m->score + (uint64_t)(uint32_t)m->player->hp;
#if MODEL != 0
            m->player->strong -= 1;                        /* the release: one thread, after the scope */
#endif
            free(m);
        }
    }
    for (uint32_t i = 0; i < N; i++) if (store[i].strong != 1) { fprintf(stderr, "count wrong\n"); return 1; }
    static const char *names[] = {"matches-0-plainC", "matches-1-atomic-in-task", "matches-2-deferred-at-join"};
    static const char *hot[] = {"matches-0-plainC-hot", "matches-1-atomic-in-task-hot", "matches-2-deferred-at-join-hot"};
    h_report(HOT ? hot[MODEL] : names[MODEL], sum);
    return 0;
}
