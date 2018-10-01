#include "shabal.h"
#include <string.h>
#include "common.h"
#include "sph_shabal.h"

sph_shabal_context global_64;

void init_shabal_sph() {
    sph_shabal256_init(&global_64);
}

void find_best_deadline_sph(char* scoops, uint64_t nonce_count, char* gensig,
                             uint64_t* best_deadline, uint64_t* best_offset) {
    uint64_t dl = 0;

    char sig[32 + 64];
	char res[32];
	memcpy_s(sig, sizeof(sig), gensig, sizeof(char) * 32);

	sph_shabal_context x, init_x;
    memcpy(&init_x, &global_64, sizeof(global_64)); 

	for (uint64_t i = 0; i < nonce_count; i++){
		memcpy_s(&sig[32], sizeof(sig)-32, &scoops[i * 64], sizeof(char)* 64);
		
		memcpy(&x, &init_x, sizeof(init_x));
		//sph_shabal256(&x, (const unsigned char*)sig, 64 + 32);
		//sph_shabal256_close(&x, res);
		sph_shabal_openclose_fast(&x, (const unsigned char*)sig, 64 + 32, res);

		unsigned long long dl = *((unsigned long long*)res);
  
        SET_BEST_DEADLINE(dl, i);
    }
}