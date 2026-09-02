# Installer packaging

Source for the three native installers built and attached to
[GitHub Releases](https://github.com/k-aksha/neural-disk/releases) by CI. All
three target the primary `neuraldisk` (Slint GUI) app only.

None of these bundle Ollama itself - they detect whether it's installed and,
with the user's yes/no consent, guide them through installing it and pulling
the default `llama3.1` model. Declining either step still leaves a fully
working NeuralDisk install; only the optional AI copilot needs them.

- **`macos/`** - `.pkg` installer. `build_pkg.sh <binary> <version> <output-dir>`
  generates an `.icns` from `neuraldisk/icons/neuraldisk_logo_flag.png`,
  assembles `NeuralDisk.app`, and wraps it via `pkgbuild`/`productbuild`. The
  Ollama/model prompts run from `scripts/postinstall` as GUI dialogs in the
  logged-in user's session (postinstall itself runs as root).
  Built and tested locally with real macOS tooling - see
  `.github/workflows/mac.yml`'s "Build .pkg installer" step for how CI invokes it.

- **`windows/`** - `neuraldisk.iss`, an Inno Setup script compiled by
  `ISCC.exe` in CI (`.github/workflows/windows.yml`'s `windows-installer`
  job). The Ollama/model prompts are `[Code]` section `MsgBox`/`Exec` calls.
  Not compilable in this repo's usual dev environment (Inno Setup is
  Windows-only) - verified via CI.

- **`linux/debian/`** - `templates` + `postinst` for `cargo-deb` (see
  `neuraldisk/Cargo.toml`'s `[package.metadata.deb]`, specifically
  `maintainer-scripts`). Builds via `cargo deb -p neuraldisk`. Ollama/model
  prompts use debconf (`db_input`/`db_go`), not a raw TTY prompt - required
  by Debian policy, and it correctly no-ops under unattended/non-interactive
  installs instead of hanging.
