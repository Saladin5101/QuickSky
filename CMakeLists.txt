cmake_minimum_required(VERSION 3.10)
project(QuickSkySys)

#
# Copyright (C) 2025 Saladin5101
# This file is part of QuickSky, licensed under the AGPLv3.
#

if (WIN32)
    set(SRC_FILES c_src/windows/sys_win.c)
else()
    set(SRC_FILES c_src/linux/sys_linux.c)
endif()

add_library(quickskysys STATIC ${SRC_FILES})
target_include_directories(quickskysys PUBLIC c_src)