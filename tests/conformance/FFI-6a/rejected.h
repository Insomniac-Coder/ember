/* The C side of accept_other_macros_are_skipped_with_clangs_reason.em. */
#ifndef REJECTED_H
#define REJECTED_H
int rejected_next(void);
#define REJECTED_TWICE(x) ((x) * 2)
#define REJECTED_TYPE unsigned int
#define REJECTED_CALL rejected_next()
#define REJECTED_KEPT 1
#define REJECTED_FEATURE
#endif
