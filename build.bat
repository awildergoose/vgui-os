@echo off
setlocal

set ISO=build\iso

@REM build
cargo build

@REM copy
if NOT EXIST build (
    mkdir build          >nul
    mkdir %ISO%\boot     >nul
    mkdir %ISO%\EFI\BOOT >nul

    copy /y "limine\BOOTX64.EFI"        "%ISO%\EFI\BOOT\" >nul
    copy /y "limine\limine-bios-cd.bin" "%ISO%\"          >nul
    copy /y "limine\limine-uefi-cd.bin" "%ISO%\"          >nul
    copy /y "limine\limine-bios.sys"    "%ISO%\"          >nul
)

copy /y "target\x86_64-unknown-none\debug\vgui" "%ISO%\boot\vgui"   >nul
copy /y "limine.conf"                             "%ISO%\limine.conf" >nul

@REM iso
xorriso.exe -as mkisofs ^
    -R -r -J ^
    -b limine-bios-cd.bin ^
    -no-emul-boot ^
    -boot-load-size 4 ^
    -boot-info-table ^
    --efi-boot limine-uefi-cd.bin ^
    -efi-boot-part ^
    --efi-boot-image ^
    "build/iso" ^
    -o "build/vgui.iso" 2>nul
