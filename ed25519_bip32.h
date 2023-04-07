#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

extern "C" {

uint8_t *derive_key_c(const uint8_t *seed,
                      uintptr_t seedlen,
                      const uint8_t *path,
                      uintptr_t pathlen);

} // extern "C"
