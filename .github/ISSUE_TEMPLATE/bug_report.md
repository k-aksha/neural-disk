---
name: Bug report
about: Create a report to help us improve app
title: ''
labels: bug
assignees: ''

---

<!-- NeuralDisk Gui (GTK) is no longer supported - version 12.0 was the last release, no new binaries or fixes will be provided. Issues reported exclusively against NeuralDisk Gui (GTK) (not reproducible in NeuralDisk, Cedinia, the CLI, or NeuralDisk Core) will be closed. Please switch to NeuralDisk: https://github.com/k-aksha/neural-disk/tree/master/neuraldisk -->

**Bug Description**

**Steps to reproduce:**
<!-- Please describe what you expected to see and what you saw instead. Also include screenshots or screencasts if needed. -->

**Terminal output** (optional):

```
<!--
Add terminal output only if needed - if there are some errors or warnings or you have performance/freeze issues.  
Very helpful in this situation will be logs from NeuralDisk run with RUST_LOG environment variable set e.g. 
`RUST_LOG=debug ./neuraldisk` or `flatpak run --env=RUST_LOG=debug io.neuraldisk.neuraldisk_gui` if you use flatpak, which will print more detailed info about executed function.
-->

<details>
<summary>Debug log</summary>

# UNCOMMENT DETAILS AND PUT LOGS HERE

</details>
```

**System**

<!-- OS and NeuralDisk version and other OS info - you can copy it from the logs if you run the app from a terminal or locate the log files manually
(Linux: `/home/username/.cache/neuraldisk`,
macOS: `/Users/Username/Library/Caches/io.neuraldisk.NeuralDisk`,
Windows: `C:\Users\Username\AppData\Local\neuraldisk\NeuralDisk\cache`).
Note: the exact path depends on the installation method(you can open config/cache path from gui). -->
<!-- Example of logs: -->
<!-- NeuralDisk version: 1.0.0, debug mode, rust 1.94.1 (2025-06-23), os Ubuntu 25.4.0 (x86_64 64-bit), 24 cpu/threads, features(1): [fast_image_resize], app cpu version: x86-64-v3 (AVX2) or x86-64-v4 (AVX-512), os cpu version: x86-64-v4 (AVX-512) -->
<!-- Config folder set to "/home/user/.config/neuraldisk" and cache folder set to "/home/user/.cache/neuraldisk" -->

- NeuralDisk version: <!--  e.g. 1.0.0 cli/gui -->
- OS version: <!--  e.g. Ubuntu 22.04, Windows 11, Mac 15.1 ARM -->
- Installation method: <!-- e.g. github binaries, flatpak, msys2 -->

<!-- If you use flatpak, please include the result of `flatpak info io.neuraldisk.neuraldisk_gui`. -->
