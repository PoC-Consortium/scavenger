#include "shabal.h"
#include <string.h>
#include "common.h"
#include "sph_shabal.h"

void find_best_deadline_sph(char *scoops, uint64_t nonce_count, char *gensig,
                             uint64_t *best_deadline, uint64_t *best_offset) {
    uint64_t dl = 0;
	for (uint64_t i = 0; i < nonce_count; i++){
		sph_shabal_deadline_fast(&scoops[i * 64], gensig, &dl);
        SET_BEST_DEADLINE(dl, i);
    }
}