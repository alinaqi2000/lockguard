# Installation

LockGuard is distributed through several channels. Pick whichever fits your environment.

## Cargo

The simplest method if you have Rust installed:

```sh
cargo install lockguard
```

This downloads, compiles, and installs the binary to `~/.cargo/bin/lockguard`. Make sure `~/.cargo/bin` is on your `PATH`.

To upgrade to a newer version later, run the same command again. Cargo will overwrite the existing binary.

## One-liner install script (Linux)

If you don't have Rust and just want the binary:

```sh
curl -fsSL https://github.com/alinaqi2000/lockguard/releases/latest/download/install.sh | sh
```

This detects your architecture, downloads the latest release binary, and installs it to `/usr/local/bin/lockguard`. It requires `sudo` for the final install step.

To install a specific version, replace `latest` with the tag:

```sh
curl -fsSL https://github.com/alinaqi2000/lockguard/releases/download/v0.1.4/install.sh | sh
```

## Debian / Ubuntu

Download the `.deb` package from the [releases page](https://github.com/alinaqi2000/lockguard/releases) and install with `dpkg`:

```sh
wget https://github.com/alinaqi2000/lockguard/releases/latest/download/lockguard_0.1.4_amd64.deb
sudo dpkg -i lockguard_0.1.4_amd64.deb
```

After installation, `lockguard` is available at `/usr/bin/lockguard`.

To verify the package integrity, check the SHA256 checksum:

```sh
sha256sum -c lockguard_0.1.4_amd64.deb.sha256
```

## Fedora / RHEL / openSUSE

Download the `.rpm` package and install with `rpm`:

```sh
sudo rpm -i https://github.com/alinaqi2000/lockguard/releases/latest/download/lockguard-0.1.4-1.x86_64.rpm
```

To verify:

```sh
sha256sum -c lockguard-0.1.4-1.x86_64.rpm.sha256
```

## Arch Linux (AUR)

An AUR package `lockguard-bin` is available. Install with your AUR helper:

```sh
yay -S lockguard-bin
```

## Build from source

If you need a debug build, want to modify the code, or are on an architecture without pre-built binaries:

```sh
git clone https://github.com/alinaqi2000/lockguard.git
cd lockguard
cargo build --release
```

The binary will be at `target/release/lockguard`. Copy it to wherever you keep local binaries:

```sh
cp target/release/lockguard ~/.local/bin/
```

## Verify the installation

Regardless of how you installed it, verify it works:

```sh
lockguard --version
```

Should print:

```
lockguard 0.1.4
```
