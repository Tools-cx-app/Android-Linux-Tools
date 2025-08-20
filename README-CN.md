alt — Android chroot 管理器（中文版）

`alt` 是一个轻量级命令行工具，用于在 Android 设备上管理 Linux rootfs 的 chroot 环境。

使用 Rust 编写，基于 `clap` 解析参数，运行时需要 root 权限。

---

主要功能

- 安装 rootfs 压缩包（`.tar`、`.tar.gz`、`.tar.xz` 等）到任意目录  
- 一键挂载必要的伪文件系统（`/proc`、`/sys`、`/dev` …）  
- 快速进入 chroot 交互 shell  
- 安全卸载或直接删除整个 chroot 环境  
- 提供静态单文件二进制（`armv7`、`aarch64`、`x86_64`），复制即用

---

编译

```bash
# 1. 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 添加 Android 目标（示例：aarch64）
rustup target add aarch64-linux-android

# 3. 构建
cargo build --release --target aarch64-linux-android
```

生成的二进制文件位于

`target/aarch64-linux-android/release/alt`

---

安装到设备

```bash
adb push target/.../alt /data/local/tmp/alt
adb shell 'su -c "cp /data/local/tmp/alt /system/bin/alt && chmod 755 /system/bin/alt"'
```

---

快速开始

```bash
# 1. 安装 rootfs
alt install rootfs.tar.xz /data/local/chroot

# 2. 启动环境
alt start /data/local/chroot

# 3. 进入 shell
alt login /data/local/chroot

# 4. 使用完毕后
alt unmount /data/local/chroot   # 可选：卸载挂载点
alt remove /data/local/chroot    # 注意：此操作会删除全部数据
```

---

命令说明

```
Android chroot 管理器

用法: alt [选项] <命令>

命令:
  install  将 rootfs 安装到指定目录
  remove   删除 chroot 目录
  login    进入正在运行的 chroot shell
  unmount  卸载 chroot 下的所有挂载点
  help     打印此帮助或子命令的详细帮助

选项:
  -h, --help         打印帮助信息
  -V, --version      打印版本信息
```

---

使用示例

从 SD 卡安装 Debian rootfs：

```bash
alt install /sdcard/debian-bookworm-arm64.tar.xz /data/debian
```

在 Termux 目录里运行 Ubuntu：

```bash
alt install ubuntu.tar.gz $PREFIX/var/lib/ubuntu
alt start $PREFIX/var/lib/ubuntu
alt login $PREFIX/var/lib/ubuntu
```

---

许可证

MIT 或 Apache-2.0，任选其一。