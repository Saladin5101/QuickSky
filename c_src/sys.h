#ifndef QUICKSKY_SYS_H
#define QUICKSKY_SYS_H

// Get the current working directory (buf: output buffer, buf_size: buffer size, returns 0=success)
int sky_get_cwd(char* buf, int buf_size);

// Check if a file is readable and writable (path: file path, returns 0=readable and writable)
int sky_check_file_access(const char* path);

#endif  // QUICKSKY_SYS_H