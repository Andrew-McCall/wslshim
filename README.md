# wslshim

A tiny, dependency-free Windows executable that runs the WSL command it's named. Run it while named `grep.exe` and `grep.exe C:\logs\app.log` runs `grep /mnt/c/logs/app.log` in WSL.

## Usage

Copy the built exe to `<tool>.exe` for each WSL tool you want available on the Windows `PATH`:

```
copy wslshim.exe C:\Users\you\bin\grep.exe
copy wslshim.exe C:\Users\you\bin\ssh.exe
```

Then call the tool as normal. Exit codes are passed through.

```
grep.exe -rn TODO C:\src\project
-> wsl.exe -e grep -rn TODO /mnt/c/src/project
```

### Path translation

| Argument            | Becomes               |
| ------------------- | --------------------- |
| `C:\foo\bar`        | `/mnt/c/foo/bar`      |
| `d:/data/x`         | `/mnt/d/data/x`       |
| `--file=C:\x\y`     | `--file=/mnt/c/x/y`   |
| `-la`, `D:`, `user@host:/p`, `foo\bar`, `x=C:\a` | unchanged |

Only an argument that is entirely a drive path, or an `--option=` whose value is one, is rewritten.

### `--no-translate`

If the first argument is `--no-translate`, it is consumed and every remaining argument is passed through untouched:

```
sed.exe --no-translate -n "s/a:\/b/c/p" file.txt
```

## How It Works

The tool name is the exe's file stem. The shim launches `%SystemRoot%\System32\wsl.exe -e <name> <args...>` (falling back to `C:\Windows`), waits for it, and exits with its status. If `wsl.exe` cannot be started it prints the error and exits with 127.

## Build and deploy

Requires Rust 1.88 or newer (the code uses let chains) and [cargo-xwin](https://github.com/rust-cross/cargo-xwin) to cross-compile from WSL:

```
cargo xwin build --release --target x86_64-pc-windows-msvc
```

`deploy.sh` builds, then overwrites every `.exe` in the bin directory (default `/mnt/c/Users/AndrewMcCall/bin`, or pass another path) with the new shim. A shim that is currently running cannot be overwritten, so it is renamed to `<name>.exe.old` and replaced; the leftovers are removed on the next run.

```
./deploy.sh [BIN_DIR]
```

Because it replaces every `.exe` in that directory, keep only shims there.

Tests run on any host:

```
cargo test
```
