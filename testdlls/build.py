#!/usr/bin/env python3
"""Build bench (cdylib) and runner64 (exe) in release mode and stage them into testdlls/target/.

Order matters: bench must be built first so its import library can be copied into
runner64/ as bench64.dll.lib before runner64 is linked.
"""
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
BENCH_DIR = ROOT / "bench64"
RUNNER_DIR = ROOT / "runner64"
OUT_DIR = ROOT / "target"


def cargo_target_dir(crate_dir: Path) -> Path:
    """Resolve the (possibly workspace-level) target directory cargo will use."""
    out = subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=crate_dir,
        shell=True,
    )
    return Path(json.loads(out)["target_directory"])


def cargo_build(crate_dir: Path, extra_env: dict | None = None) -> None:
    print(f"==> cargo build --release ({crate_dir.name})")
    env = os.environ.copy()
    if extra_env:
        env.update(extra_env)
    subprocess.run(
        ["cargo", "build", "--release"],
        cwd=crate_dir,
        check=True,
        shell=True,
        env=env,
    )


def copy(src: Path, dst: Path) -> None:
    if not src.exists():
        sys.exit(f"missing expected build artifact: {src}")
    dst.parent.mkdir(parents=True, exist_ok=True)
    # Remove dst first: it may be a hardlink to src (cargo creates hardlinks),
    # which would make shutil.copy2 raise SameFileError.
    if dst.exists():
        dst.unlink()
    shutil.copy2(src, dst)
    print(f"    {src} -> {dst}")


def main() -> None:
    workspace_target = cargo_target_dir(BENCH_DIR)
    bench_release = workspace_target / "release"
    runner_release = cargo_target_dir(RUNNER_DIR) / "release"

    cargo_build(BENCH_DIR)

    # Stage import library where runner64 expects it (its .cargo/config.toml has
    # `rustflags = ["-L", "."]` for resolving bench64.dll.lib).
    copy(bench_release / "bench64.dll.lib", RUNNER_DIR / "bench64.dll.lib")

    # That `-L .` resolves relative to cargo's cwd, which inside a workspace is the
    # workspace root — not runner64/. Override with an absolute -L so the linker
    # actually finds bench64.dll.lib.
    cargo_build(RUNNER_DIR, extra_env={"RUSTFLAGS": f"-L {RUNNER_DIR}"})

    # Stage final artifacts: runner64.exe needs bench64.dll alongside it at runtime.
    copy(runner_release / "runner64.exe", OUT_DIR / "runner64.exe")
    copy(bench_release / "bench64.dll", OUT_DIR / "bench64.dll")

    # Tear down cargo's workspace-level target dir now that artifacts are staged.
    print(f"==> rm -rf {workspace_target}")
    shutil.rmtree(workspace_target, ignore_errors=True)

    print(f"\nbuild complete -> {OUT_DIR}")


if __name__ == "__main__":
    main()
