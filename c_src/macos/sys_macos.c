#include "../sys.h"
#include <unistd.h>
#include <errno.h>



int sky_chdir(const char* path) {
    if (chdir(path) == 0) {
        return 0;
    } else {
        return errno;
    }
}