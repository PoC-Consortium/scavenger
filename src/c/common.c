#include "common.h"
#include <string.h>

void write_term(char term[32]) {
    term[0] = -128;  // shabal message termination bit
    memset(&term[1], 0, 31);
}