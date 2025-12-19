#!/bin/bash
set -euo pipefail

PROJECT_NAME="siligpu"
TARGET_DIR="target/release"
RELEASE_DIR="$TARGET_DIR"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SANDBOX_PREFIX="/tmp/siligpu-sandbox"

log() {
    printf "[build-clean] %s\n" "$*"
}

error() {
    printf "[build-clean] ERROR: %s\n" "$*" >&2
}

ensure_command() {
    local cmd="$1"
    if ! command -v "$cmd" >/dev/null 2>&1; then
        error "Required command '$cmd' is not available"
        exit 1
    fi
}

ensure_toolchain() {
    if ! rustup show active-toolchain | grep -q nightly; then
        log "Switching to nightly toolchain"
        rustup override set nightly
    fi
}

ensure_llvm_objcopy() {
    if command -v llvm-objcopy >/dev/null 2>&1; then
        LLVM_OBJCOPY=$(command -v llvm-objcopy)
    else
        log "llvm-objcopy not on PATH; installing llvm"
        brew install llvm >/dev/null
        local prefix
        prefix=$(brew --prefix llvm)
        LLVM_OBJCOPY="$prefix/bin/llvm-objcopy"
    fi

    if [ ! -x "$LLVM_OBJCOPY" ]; then
        error "llvm-objcopy could not be located; install llvm manually"
        exit 1
    fi
}

select_remap_flag() {
    if rustc -Z help 2>/dev/null | grep -q "remap-path-prefix"; then
        export RUSTFLAGS="-Zremap-path-prefix=${HOME}=."
        log "Using -Zremap-path-prefix to cleanse build paths"
        return
    fi

    if rustc -Z help 2>/dev/null | grep -q "remap-cwd-prefix"; then
        export RUSTFLAGS="-Zremap-cwd-prefix=."
        log "Using -Zremap-cwd-prefix because remap-path-prefix is unavailable"
        return
    fi

    log "rustc lacks remap flags; proceeding without path remapping"
    unset RUSTFLAGS
}

copy_workspace_to_sandbox() {
    local sandbox_dir="$1"
    rsync -a --delete --exclude target --exclude release --exclude .git "$SCRIPT_DIR/" "$sandbox_dir/"
}

run_sandboxed_build() {
    local sandbox_dir
    sandbox_dir="$(mktemp -d "${SANDBOX_PREFIX}-XXXXXX")"
    cleanup_sandbox() {
        rm -rf "$sandbox_dir"
    }
    trap cleanup_sandbox EXIT

    log "Copying workspace into sandbox ($sandbox_dir)"
    copy_workspace_to_sandbox "$sandbox_dir"

    local sandbox_home="$sandbox_dir/home"
    local sandbox_rustup="$sandbox_dir/rustup"
    local sandbox_cargo="$sandbox_dir/cargo"
    mkdir -p "$sandbox_home" "$sandbox_rustup" "$sandbox_cargo"

    local env_vars=(
        "HOME=$sandbox_home"
        "RUSTUP_HOME=$sandbox_rustup"
        "CARGO_HOME=$sandbox_cargo"
    )
    if [ -n "${RUSTFLAGS:-}" ]; then
        env_vars+=("RUSTFLAGS=$RUSTFLAGS")
    fi

    log "Running sanitized build in sandbox"
    (
        cd "$sandbox_dir"
        env "${env_vars[@]}" rustup override set nightly
        env "${env_vars[@]}" cargo clean
        env "${env_vars[@]}" cargo build --release
        env "${env_vars[@]}" strip "target/release/$PROJECT_NAME"
        env "${env_vars[@]}" "$LLVM_OBJCOPY" --strip-all "target/release/$PROJECT_NAME"
    )

    mkdir -p "$SCRIPT_DIR/$RELEASE_DIR"
    cp "$sandbox_dir/$TARGET_DIR/$PROJECT_NAME" "$SCRIPT_DIR/$RELEASE_DIR/$PROJECT_NAME"

    trap - EXIT
    cleanup_sandbox
}

verify_clean() {
    local binary="$1"
    log "Checking for local paths or usernames in $binary"
    local occurrences
    occurrences=$(strings "$binary" | grep -E --color=never -n "${USER}|/Users/|${HOME}" || true)
    if [ -n "$occurrences" ]; then
        error "Local paths or usernames still present in $binary"
        printf "%s\n" "$occurrences"
        return 1
    fi
    log "Binary is clean"
}

main() {
    ensure_command rustup
    ensure_command cargo
    ensure_command strip
    ensure_command rsync

    ensure_toolchain
    ensure_llvm_objcopy
    select_remap_flag

    run_sandboxed_build

    STRIPPED_BIN="$SCRIPT_DIR/$RELEASE_DIR/$PROJECT_NAME"
    verify_clean "$STRIPPED_BIN"

    log "Clean binary is available in $RELEASE_DIR"
}

main "$@"
