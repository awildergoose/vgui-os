@echo off
setlocal

qemu-system-x86_64 ^
    -machine q35 ^
    -accel whpx ^
    -cpu host ^
    -smp 1 ^
    -m 512M ^
    -cdrom "build\vgui.iso" ^
    -boot d ^
    -vga std ^
    -serial stdio
