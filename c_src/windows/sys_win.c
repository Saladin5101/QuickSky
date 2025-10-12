#include "../sys.h"
#include <windows.h>

/*
 * Copyright (C) 2025 Saladin5101
 * This file is part of QuickSky, licensed under the AGPLv3.
 */

int sky_get_cwd(char* buf, int buf_size) {
    DWORD len = GetCurrentDirectoryA(buf_size, buf);
    return (len > 0 && len < buf_size) ? 0 : GetLastError();
}

int sky_check_file_access(const char* path) {
    // Check if the file exists and is readable/writable
    DWORD attr = GetFileAttributesA(path);
    if (attr == INVALID_FILE_ATTRIBUTES) return GetLastError();
    // Simple check: not a directory and accessible (can be stricter if needed)
    return (attr & FILE_ATTRIBUTE_DIRECTORY) ? 1 : 0;
}