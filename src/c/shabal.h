#pragma once

#include <stdint.h>
#include <stdlib.h>

void find_best_deadline_sph(char *scoops, uint64_t nonce_count, char *gensig,
                             uint64_t *best_deadline, uint64_t *best_offset);
