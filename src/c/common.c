#include "common.h"
#include <string.h>

void write_seed(char seed[32], uint64_t numeric_id) {
    numeric_id = bswap_64(numeric_id);
    memmove(&seed[0], &numeric_id, 8);
    memset(&seed[8], 0, 8);
    seed[16] = -128;  // shabal message termination bit
    memset(&seed[17], 0, 15);
}

void write_term(char term[32]) {
    term[0] = -128;  // shabal message termination bit
    memset(&term[1], 0, 31);
}
