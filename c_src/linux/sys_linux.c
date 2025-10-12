/*
    QuickSky - 懒人友好的版本控制工具（命令：sky）
    Copyright (C) 2025  Saladin5101

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
#include "../sys.h"
#include <unistd.h>   // Linux native chdir
#include <errno.h>    // Get error codes (e.g., ENOENT means path does not exist)



int sky_chdir(const char* path) {
    // Call system chdir, return 0 on success, error code on failure (non-zero)
    if (chdir(path) == 0) {
        return 0;
    } else {
        // Return specific error code (Rust side can interpret and convert to user-friendly messages)
        return errno;
    }
}
