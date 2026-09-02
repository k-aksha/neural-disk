<div align="center"><img src="https://github.com/user-attachments/assets/f5e4b290-d001-4cf4-9f52-dab65a30e441" alt="neuraldisk_logo" width="600" /></div>

**NeuralDisk** is a simple, multiplatform, fast, and free app to remove unnecessary files from your computer - with an
optional AI copilot that can drive the cleanup for you, one confirmed action at a time.

## Where this comes from

NeuralDisk is a fork and continuation of [**Czkawka**](https://github.com/qarmin/czkawka), the excellent
duplicate-file-finder and disk-cleanup toolkit created and maintained by **Rafał Mikrut ("qarmin")**. Everything in
this repository - the scanning engine, every tool listed below, the original GUI/CLI frontends, and years of careful
engineering - is built on that project's work. If you're looking for the original, actively-maintained upstream
project (with its own releases, community, and roadmap), that's the place to go: **https://github.com/qarmin/czkawka**.

This fork keeps the GPL-3.0/MIT/CC-BY-4.0 licensing exactly as upstream, and the original copyright notices remain in
the LICENSE files. NeuralDisk is an independent continuation, not an official Czkawka release, and upstream is not
involved in maintaining it.

## What's new in NeuralDisk

Starting from that foundation, this fork adds:

- **An AI copilot panel** in the primary desktop GUI - a chat assistant (backed by a locally-run [Ollama](https://ollama.com)
  server, so nothing leaves your machine) that can run any of the scanners below and propose file actions
  (delete/trash/move/rename/hardlink/symlink/etc.) in natural language. It never performs a destructive action
  directly - every proposal still opens the exact same confirmation dialog a manual click would, so you always review
  and approve before anything is touched.
- **A redesigned interface**: a collapsible icon-and-label sidebar, resizable panels (main list / preview / copilot
  chat) with persisted sizing, hairline dividers, and explicit high-contrast styling for selected/disabled controls.
- **A full rebrand** (Krokiet/Czkawka → NeuralDisk) across binaries, packaging metadata, and user-facing text, under
  the `io.neuraldisk.*` namespace.

Everything else - the scanning tools, their algorithms, the CLI, the legacy GTK GUI, the Android app - is Czkawka's
work, carried forward as-is or with minor adjustments.

## Features

- **Written in memory-safe Rust** - almost 100% unsafe code free
- **Amazingly fast** - due multithreading and efficient algorithms
- **Free, Open Source without any ads**
- **Multiplatform** - runs on Linux, Windows, macOS, FreeBSD, x86, ARM, RISC-V and even Android
- **Cache support** - second and further scans should be much faster than the first one
- **Easy to run, easy to compile** - minimal runtime and build dependencies, portable version available
- **CLI frontend** - for easy automation
- **GUI frontend** - uses Slint or GTK 4 frameworks
- **Core library** - allows to reuse functionality in other apps
- **Android app** - touch-friendly frontend for Android devices
- **AI copilot (optional)** - a local, natural-language assistant for driving the tools below
- **No spying** - NeuralDisk does not have access to the Internet, nor does it collect any user information or statistics
- **Multilingual** - support multiple languages like Polish, English or Italian
- **Multiple tools to use**:
    - **Duplicates** - Finds duplicates based on file name, size or hash
    - **Empty Folders** - Finds empty folders with the help of an advanced algorithm
    - **Big Files** - Finds the provided number of the biggest files in given location
    - **Empty Files** - Looks for empty files across the drive
    - **Temporary Files** - Finds temporary files
    - **Similar Images** - Finds images which are not exactly the same (different resolution, watermarks)
    - **Similar Videos** - Looks for visually similar videos
    - **Same Music** - Searches for similar music by tags or by reading content and comparing it
    - **Invalid Symbolic Links** - Shows symbolic links which point to non-existent files/directories
    - **Broken Files** - Finds files that are invalid or corrupted
    - **Bad Extensions** - Lists files whose content not match with their extension
    - **Exif Remover** - Removes Exif metadata from various file types
    - **Video Optimizer** - Crops from static parts and converts videos to more efficient formats
    - **Bad Names** - Finds files with names that may be not wanted (e.g., containing special characters)

## Usage, installation, compilation, requirements, license

Each tool uses different technologies, so you can find instructions for each of them in the appropriate file:

- [NeuralDisk GUI (Slint frontend)](neuraldisk/README.md)</br>
- [NeuralDisk Gui (GTK frontend, legacy)](neuraldisk_gui/README.md)</br>
- [NeuralDisk CLI](neuraldisk_cli/README.md)</br>
- [NeuralDisk Core](neuraldisk_core/README.md)</br>
- [Cedinia (Android)](cedinia/README.md)</br>

## Setting up the AI copilot (optional)

The copilot panel in NeuralDisk (the Slint GUI) is entirely optional - every scanning tool works
without it, and it's disabled by default. To use it, you need [Ollama](https://ollama.com) running
locally and the `llama3.1` model pulled (both are NeuralDisk's defaults - configurable in Settings):

```shell
# 1. Install Ollama (see https://ollama.com/download for your OS), then:
ollama pull llama3.1

# 2. Start it (skip this if the Ollama app/service is already running):
ollama serve
```

Then open NeuralDisk's Settings and enable the copilot. The installers below (macOS `.pkg`, Windows
setup `.exe`, Linux `.deb`) offer to do both of these steps for you during installation - each asks
before installing Ollama and before downloading the model (~4.7 GB), and neither step is required to
use the rest of the app.

## Installers

Prebuilt installers are attached to the [releases page](https://github.com/k-aksha/neural-disk/releases)
alongside the raw binaries: a `.pkg` for macOS, a `Setup.exe` for Windows, and a `.deb` for
Debian/Ubuntu-based Linux. None of these are code-signed (no certificate is available for this
fork), so your OS will likely warn about an unidentified/unknown publisher on first run - that's
expected, not a sign of tampering. Source for all three lives under
[`packaging/`](packaging/).

## Comparison to other tools

In this comparison remember, that even if app have same features they may work different(e.g. one app may have more
options to choose than other).

|                           | NeuralDisk  | NeuralDisk Gui | Cedinia | FSlint |     DupeGuru      |  Bleachbit  |
|:-------------------------:|:-----------:|:-----------:|:-------:|:------:|:-----------------:|:-----------:|
|         Language          |    Rust     |    Rust     |  Rust   | Python |   Python/Obj-C    |   Python    |
|  Framework base language  |    Rust     |      C      |  Rust   |   C    | C/C++/Obj-C/Swift |      C      |
|         Framework         |    Slint    |    GTK 4    |  Slint  | PyGTK2 | Qt 5 (PyQt)/Cocoa |   PyGTK3    |
|            OS             | Lin,Mac,Win | Lin,Mac,Win | Android |  Lin   |    Lin,Mac,Win    | Lin,Mac,Win |
|     Duplicate finder      |      ✔      |      ✔      |    ✔    |   ✔    |         ✔         |             |
|        Empty files        |      ✔      |      ✔      |    ✔    |   ✔    |                   |             |
|       Empty folders       |      ✔      |      ✔      |    ✔    |   ✔    |                   |             |
|      Temporary files      |      ✔      |      ✔      |    ✔    |   ✔    |                   |      ✔      |
|         Big files         |      ✔      |      ✔      |    ✔    |        |                   |             |
|      Similar images       |      ✔      |      ✔      |    ✔    |        |         ✔         |             |
|   Similar videos(audio)   |      ✔      |      ✔      |    ✔    |        |                   |             |
|  Similar videos(frames)   |      ✔      |      ✔      |         |        |                   |             |
|  Music duplicates(tags)   |      ✔      |      ✔      |    ✔    |        |         ✔         |             |
| Music duplicates(content) |      ✔      |      ✔      |    ✔    |        |                   |             |
|     Invalid symlinks      |      ✔      |      ✔      |         |   ✔    |                   |             |
|       Broken files        |      ✔      |      ✔      |    ✔    |        |                   |             |
| Invalid names/extensions  |      ✔      |      ✔      |    ✔    |   ✔    |                   |             |
|       Exif cleaner        |      ✔      |             |    ✔    |        |                   |             |
|      Video optimizer      |      ✔      |             |         |        |                   |             |
|         Bad Names         |      ✔      |             |    ✔    |        |                   |             |
|      AI copilot chat      |      ✔      |             |         |        |                   |             |
|      Names conflict       |             |             |         |   ✔    |                   |             |
|    Installed packages     |             |             |         |   ✔    |                   |             |
|          Bad ID           |             |             |         |   ✔    |                   |             |
|   Non stripped binaries   |             |             |         |   ✔    |                   |             |
|   Redundant whitespace    |             |             |         |   ✔    |                   |             |
|     Overwriting files     |             |             |         |   ✔    |                   |      ✔      |
|     Portable version      |      ✔      |      ✔      |         |        |                   |      ✔      |
|    Multiple languages     |      ✔      |      ✔      |    ✔    |   ✔    |         ✔         |      ✔      |
|       Cache support       |      ✔      |      ✔      |    ✔    |        |         ✔         |             |

## Other apps

There are many similar applications to NeuralDisk on the Internet, which do some things better and some things worse:

### GUI

- [DupeGuru](https://github.com/arsenetar/dupeguru) - Many options to customize
- [FSlint](https://github.com/pixelb/fslint) - A little outdated, but still have some tools not available in NeuralDisk
- [AntiDupl.NET](https://github.com/ermig1979/AntiDupl) - Shows a lot of metadata of compared images
- [Video Duplicate Finder](https://github.com/0x90d/videoduplicatefinder) - Finds similar videos(surprising, isn't it)

### CLI

- [Fclones](https://github.com/pkolaczk/fclones) - One of the fastest tools to find duplicates; it is written also in Rust
- [Rmlint](https://github.com/sahib/rmlint) - Nice console interface and also is feature packed
- [RdFind](https://github.com/pauldreik/rdfind) - Fast, but written in C++ ¯\\\_(ツ)\_/¯

## Acknowledgements

NeuralDisk exists because of the work of many people on the upstream Czkawka project, most of all its creator,
Rafał Mikrut, and everyone who contributed code, translations, bug reports, and packaging over the years. It also
builds on [FSlint](https://github.com/pixelb/fslint) by Pádraig Brady, which was Czkawka's own original inspiration.

## AI Policy

The upstream Czkawka project's own code was, by its maintainer's account, almost entirely written without AI
assistance. This fork's additions on top of it (the AI copilot feature, the interface redesign, and the rebrand) were
built with AI assistance (Claude). That's disclosed here in the interest of transparency, not as a claim about the
quality or provenance of the underlying project.

## License

The entire code in this repository is licensed under the [MIT](https://mit-license.org/) license.

All images and audio files are licensed under the [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) license.

The NeuralDisk Gui (GTK) and CLI applications are licensed under the [MIT](https://mit-license.org/) license, while NeuralDisk and Cedinia (due Slint license requirements) are licensed under the [GPL-3.0-only](https://www.gnu.org/licenses/gpl-3.0.en.html) license.

Original copyright notices from the upstream Czkawka project are preserved in the LICENSE files in this repository.
