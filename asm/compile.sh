#!/usr/bin/env bash

arm-none-eabi-as -march=armv7-a --gdwarf2 -o $1.s.o $1.s
arm-none-eabi-ld -e _start -u _start -o $1.s.elf $1.s.o