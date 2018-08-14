#pragma once

#define SET_BEST_DEADLINE(d, o) \
    if ((d) < *best_deadline) { \
        *best_deadline = (d);   \
        *best_offset = (o);     \
    }
