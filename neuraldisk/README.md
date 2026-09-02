
![neuraldisk_logo](https://github.com/user-attachments/assets/f5e4b290-d001-4cf4-9f52-dab65a30e441)

NeuralDisk is a new frontend for NeuralDisk Core, written in Slint.

![NeuralDisk](https://github.com/user-attachments/assets/720e98c3-598a-41aa-a04b-0c0c1d8a28e6)

It aims to provide a more consistent experience across all platforms (Linux, Windows, macOS) compared to the previous GTK 4 frontend.

## How to install?
Prebuilt binaries are available for Windows 10/11, Mac and Ubuntu 22.04(base)/24.04(with additional libraries) - or distros with same/newer glibc/libraries versions.

Check this repository's [releases page](https://github.com/k-aksha/neural-disk/releases) for prebuilt binaries and recommendations on which variant to use for your platform.

## Requirements

Prebuilt binaries have no mandatory runtime dependencies.

Optional features require native libraries installed before compilation:

| Feature   | Library               | Purpose                  |
|-----------|-----------------------|--------------------------|
| `heif`    | `libheif`             | HEIF/HEIC image support  |
| `libraw`  | `libraw`              | RAW camera image support |
| `libavif` | `libavif`, `libdav1d` | AVIF image support       |

The similar videos tool requires **ffmpeg** at runtime.

### Linux (Ubuntu / Debian)

```shell
# Runtime: similar videos
sudo apt install ffmpeg

# Optional build + runtime: extra image formats
sudo apt install libheif-dev libraw-dev libavif-dev libdav1d-dev
```

### macOS

```shell
brew install ffmpeg libraw libheif libavif dav1d
```

### Windows

- ffmpeg: `choco install ffmpeg` or download from [ffmpeg.org](https://ffmpeg.org/download.html#build-windows) and place `ffmpeg.exe` in your `PATH`.
- `heif` and `libraw` features are very hard to set up on Windows and are not available in prebuild binaries(there are some unofficial builds, that enables this features)

## Compilation

Another option is to compile it yourself.

Compilation with `cargo build --release` (run from inside the `krokiet` directory) should produce a working binary,
that without any additional dependencies should run on user os.

(`cargo install neuraldisk` would work the same way once/if this fork is published to crates.io under that name -
it isn't yet, so building from a local clone is the reliable path for now.)

To enable support for extra image formats, compile with optional features:

```shell
cargo build --release --bin neuraldisk --features "heif,libraw,libavif"
```

## Additional Renderers

By default, only femtovg (OpenGL) and the software renderer are enabled, but you can enable more renderers by compiling the app
with additional features.

Most users will want to use the app with a windowing system/compositor, so features starting with `winit` in the name are
recommended.

For example:

```
cargo build --release --features "winit_skia_opengl"
cargo build --release --features "winit_software"
```

To run the app with a different renderer, set the `SLINT_BACKEND` environment variable(but app must be compiled with the appropriate feature):

```
SLINT_BACKEND=winit-femtovg ./target/release/neuraldisk
SLINT_BACKEND=software ./target/release/neuraldisk
SLINT_BACKEND=skia ./target/release/neuraldisk
```

If you use an invalid or non-existing backend, the app will show a warning:

```
slint winit: unrecognized renderer skia, falling back to FemtoVG
```

To check which backend is actually used, add the `SLINT_DEBUG_PERFORMANCE=refresh_lazy,console,overlay` environment variable:

```
SLINT_DEBUG_PERFORMANCE=refresh_lazy,console,overlay cargo run
```

You should see output like:

```
Slint: Build config: debug; Backend: software
```


## Why create a new frontend instead of improving the existing NeuralDisk Gui (GTK 4) app?

For many, it might seem surprising to abandon the existing GTK 4 frontend (NeuralDisk Gui) especially considering that GTK is one of the most popular GUI frameworks and replace it with a new one based on Slint, which is still relatively unknown.

This decision was driven by several key factors:
- **GTK on Windows and macOS performs poorly** - There are random bugs that don't appear on Linux or on other systems with similar environments. Slint, on the other hand, behaves consistently and reliably across all platforms.
- **Complicated compilation and cross-compilation** - Due to GTK's complexity on Windows, the easiest way to compile the application is by using a Docker image with Linux. This makes testing and debugging on Windows much more difficult.
- **External dependencies** - GTK apps rarely work right after downloading without a separate installation step. On Linux and macOS, several dynamically linked libraries must be installed first, and they may exist in different versions across systems. On Windows, DLLs often have to be bundled manually, since the GTK team doesn't officially distribute them or maintain a list of required files - leaving distributors to compile everything themselves or rely on external Docker images. With Slint, a single binary file runs out of the box on almost any system.
- **GTK version fragmentation across platforms** - On Linux, GTK is dynamically linked, and different versions may introduce unique bugs or inconsistencies. On Windows, bundled libraries can lag behind since newer ones aren't always available in the build environment, and some versions crash on some OSes. macOS (with Homebrew) is in the best position here, as it usually keeps GTK up to date. With Slint, each release is bundled with the latest Slint version, ensuring consistency across all systems and reducing platform-specific issues.
- **Cambalache is the only no-code GUI tool** - While Cambalache itself works reasonably well, it isn't officially supported or maintained by the GTK team, but by an independent developer. In contrast, while the Slint GUI is mostly created via code, it offers live previews in VS Code/VSCodium, which is extremely convenient.
- **Difficult to modify built-in widgets** - GTK enforces a specific visual style, which can be very restrictive to tweak internal widget parameters to achieve a desired look. Slint takes the opposite approach: its built-in widgets are quite limited, which often makes it easier to build fully custom components from scratch.
- **GTK is still C code** - Even though the library is wrapped and provides a relatively safe Rust interface, you still occasionally have to work with low-level structures, which can cause issues and crashes. Another downside is the large number of warnings printed to the console, even with correct code, due to internal GTK issues. These warnings are often unhelpful and rarely assist in identifying actual bugs.

## License

The code is licensed under the MIT license, but the entire project is licensed under GPL-3.0 due to Slint license restrictions.

All icons and images are licensed under the [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) license.

## Name

This frontend was originally named Krokiet (Polish for "croquette") under the upstream Czkawka project, and is
rebranded here as NeuralDisk. See the root [README](../README.md) for where this fork comes from.
