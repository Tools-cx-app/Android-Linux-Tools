alt — Android chroot manager

`alt` is a tiny CLI tool that lets you install, start, stop and remove Linux rootfs images inside an Android chroot environment.

It is implemented in Rust using `clap` for argument parsing and requires root on the target device.

---

Features

- Install a rootfs archive (`.tar`, `.tar.gz`, `.tar.xz`, …) into any directory  
- Mount required pseudo-filesystems (`/proc`, `/sys`, `/dev`, …) with one command  
- Attach to the chroot with an interactive shell  
- Cleanly unmount or safely delete the entire environment  
- Cross-compiled static binary (`armv7`, `aarch64`, `x86_64`) — just copy to `/system/bin`

---

Build

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Add Android targets (example: aarch64)
rustup target add aarch64-linux-android

# 3. Build
cargo build --release --target aarch64-linux-android
```

The resulting binary is at

`target/aarch64-linux-android/release/alt`.

---

Install on device

```bash
adb push target/.../alt /data/local/tmp/alt
adb shell 'su -c "cp /data/local/tmp/alt /system/bin/alt && chmod 755 /system/bin/alt"'
```

---

Quick start

```bash
# 1. Install a rootfs
alt install rootfs.tar.xz /data/local/chroot

# 2. Start the environment
alt start /data/local/chroot

# 3. Enter a shell
alt login /data/local/chroot

# 4. When done
alt unmount /data/local/chroot   # optional
alt remove /data/local/chroot    # be careful!
```

---

Usage

```
Android chroot manager

Usage: alt [OPTIONS] <COMMAND>

Commands:
  install  Install a rootfs into the specified target directory
  remove   Remove the chroot directory
  login    Open an interactive shell inside the running chroot
  unmount  Unmount all bind-mounts under the chroot directory
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help         Print help
  -V, --version      Print version
```

---

Examples

Install Debian rootfs from SD-card:

```bash
alt install /sdcard/debian-bookworm-arm64.tar.xz /data/debian
```

Run Ubuntu in Termux-style mount:

```bash
alt install ubuntu.tar.gz $PREFIX/var/lib/ubuntu
alt start $PREFIX/var/lib/ubuntu
alt login $PREFIX/var/lib/ubuntu
```

---

License

MIT or Apache-2.0 — your choice.