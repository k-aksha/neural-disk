# Packages an all-backends neuraldisk binary together with per-backend launcher
# batch scripts into a zip archive.
#
# Usage: pack_all_backends.ps1 -Binary <path> -OutputZip <path>
param(
    [Parameter(Mandatory)][string]$Binary,
    [Parameter(Mandatory)][string]$OutputZip
)

$pkgDir = Join-Path $env:TEMP ([System.IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Force $pkgDir | Out-Null

Copy-Item $Binary (Join-Path $pkgDir "neuraldisk.exe")

$backends = [ordered]@{
    "neuraldisk_winit_femtovg.bat"     = "winit-femtovg"
    "neuraldisk_winit_skia_opengl.bat" = "winit-skia-opengl"
    "neuraldisk_winit_skia_vulkan.bat" = "winit-skia-vulkan"
    "neuraldisk_winit_software.bat"    = "winit-software"
    "neuraldisk_femtovg_wgpu.bat"      = "femtovg-wgpu"
}
foreach ($file in $backends.Keys) {
    $backend = $backends[$file]
    "@echo off`r`nset SLINT_BACKEND=$backend`r`n`"%~dp0neuraldisk.exe`" %*`r`n" |
        Set-Content -Encoding ASCII (Join-Path $pkgDir $file)
}

Compress-Archive -Path (Join-Path $pkgDir "*") -DestinationPath $OutputZip -Force

Remove-Item -Recurse -Force $pkgDir
