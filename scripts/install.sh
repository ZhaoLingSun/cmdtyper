#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_binary="$repo_root/target/release/cmdtyper"
bin_dir="${HOME:?HOME must be set}/.local/bin"
installed_binary="$bin_dir/cmdtyper"
data_parent="$HOME/.local/share/cmdtyper"
installed_data="$data_parent/data"
required_data_dirs=(commands lessons symbols system)

staging_data=""
binary_tmp=""
backup_data=""
backup_binary=""
data_publish_attempted=0
binary_publish_attempted=0
transaction_complete=0

path_exists() {
    [[ -e "$1" || -L "$1" ]]
}

validate_data_tree() {
    local root=$1
    local label=$2
    local directory
    local -a toml_files

    for directory in "${required_data_dirs[@]}"; do
        if [[ ! -d "$root/$directory" ]]; then
            printf 'ERROR: required %s data directory is missing: %s\n' \
                "$label" "$root/$directory" >&2
            return 1
        fi

        toml_files=("$root/$directory"/*.toml)
        if (( ${#toml_files[@]} == 0 )); then
            printf 'ERROR: required %s data directory has no TOML files: %s\n' \
                "$label" "$root/$directory" >&2
            return 1
        fi
    done
}

cleanup() {
    local status=$?
    local rollback_failed=0
    trap - EXIT INT TERM
    set +e

    if (( status != 0 && transaction_complete == 0 )); then
        # Remove a newly published binary before restoring the old data set.
        if (( binary_publish_attempted == 1 )) && path_exists "$installed_binary"; then
            rm -rf "$installed_binary"
            if path_exists "$installed_binary"; then
                printf 'ERROR: could not remove failed binary install: %s\n' \
                    "$installed_binary" >&2
                rollback_failed=1
            fi
        fi

        if [[ -n "$backup_binary" ]] && path_exists "$backup_binary"; then
            if path_exists "$installed_binary"; then
                printf 'ERROR: could not restore previous binary; backup retained at %s\n' \
                    "$backup_binary" >&2
                rollback_failed=1
            elif mv "$backup_binary" "$installed_binary"; then
                backup_binary=""
            else
                printf 'ERROR: could not restore previous binary; backup retained at %s\n' \
                    "$backup_binary" >&2
                rollback_failed=1
            fi
        fi

        if (( data_publish_attempted == 1 )) && path_exists "$installed_data"; then
            rm -rf "$installed_data"
            if path_exists "$installed_data"; then
                printf 'ERROR: could not remove failed data install: %s\n' \
                    "$installed_data" >&2
                rollback_failed=1
            fi
        fi

        if [[ -n "$backup_data" ]] && path_exists "$backup_data"; then
            if path_exists "$installed_data"; then
                printf 'ERROR: could not restore previous data; backup retained at %s\n' \
                    "$backup_data" >&2
                rollback_failed=1
            elif mv "$backup_data" "$installed_data"; then
                backup_data=""
            else
                printf 'ERROR: could not restore previous data; backup retained at %s\n' \
                    "$backup_data" >&2
                rollback_failed=1
            fi
        fi
    fi

    if [[ -n "$staging_data" ]] && path_exists "$staging_data"; then
        rm -rf "$staging_data"
    fi
    if [[ -n "$binary_tmp" ]] && path_exists "$binary_tmp"; then
        rm -rf "$binary_tmp"
    fi

    if (( transaction_complete == 1 )); then
        if [[ -n "$backup_binary" ]] && path_exists "$backup_binary"; then
            if rm -rf "$backup_binary"; then
                backup_binary=""
            else
                printf 'WARNING: old binary backup remains at %s\n' "$backup_binary" >&2
            fi
        fi
        if [[ -n "$backup_data" ]] && path_exists "$backup_data"; then
            if rm -rf "$backup_data"; then
                backup_data=""
            else
                printf 'WARNING: old data backup remains at %s\n' "$backup_data" >&2
            fi
        fi
    elif (( status != 0 && rollback_failed == 1 )); then
        printf 'ERROR: installation rollback was incomplete; retained backup paths are shown above.\n' >&2
    fi

    exit "$status"
}

trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM

shopt -s nullglob
validate_data_tree "$repo_root/data" "source"

printf 'Building release binary from %s\n' "$repo_root"
cargo build --release --manifest-path "$repo_root/Cargo.toml"

if [[ ! -f "$source_binary" || ! -x "$source_binary" ]]; then
    printf 'ERROR: release build did not produce an executable binary: %s\n' \
        "$source_binary" >&2
    exit 1
fi

install -d "$bin_dir" "$data_parent"

# Stage and validate the complete release data set before changing any install path.
staging_data="$(mktemp -d "$data_parent/.data-staging.XXXXXX")"
for directory in "${required_data_dirs[@]}"; do
    cp -a "$repo_root/data/$directory" "$staging_data/$directory"
done
for directory in practice scenarios sequences; do
    if [[ -d "$repo_root/data/$directory" ]]; then
        cp -a "$repo_root/data/$directory" "$staging_data/$directory"
    fi
done
for metadata in command_aliases.toml command_contexts.toml; do
    if [[ -f "$repo_root/data/$metadata" ]]; then
        cp -a "$repo_root/data/$metadata" "$staging_data/$metadata"
    fi
done
validate_data_tree "$staging_data" "staged"

binary_tmp="$(mktemp "$bin_dir/.cmdtyper-staging.XXXXXX")"
install -m 755 "$source_binary" "$binary_tmp"

# Publish data first. The new binary is not exposed until the full data tree is in place.
if path_exists "$installed_data"; then
    backup_data="$(mktemp "$data_parent/.data-backup.XXXXXX")"
    rm -f "$backup_data"
    mv "$installed_data" "$backup_data"
fi

data_publish_attempted=1
mv "$staging_data" "$installed_data"
staging_data=""

# Only after data is committed do we replace the binary. Any failure rolls both paths back.
if path_exists "$installed_binary"; then
    backup_binary="$(mktemp "$bin_dir/.cmdtyper-backup.XXXXXX")"
    rm -f "$backup_binary"
    mv "$installed_binary" "$backup_binary"
fi

binary_publish_attempted=1
mv -f "$binary_tmp" "$installed_binary"
binary_tmp=""

if [[ ! -f "$installed_binary" || ! -x "$installed_binary" ]]; then
    printf 'ERROR: installed binary validation failed: %s\n' "$installed_binary" >&2
    exit 1
fi
validate_data_tree "$installed_data" "installed"

printf 'Installed release binary:\n'
printf '  source:      %s\n' "$source_binary"
printf '  destination: %s\n' "$installed_binary"
printf 'Installed release data:\n'
printf '  source:      %s/data/{commands,lessons,symbols,system,practice,scenarios,sequences}\n' "$repo_root"
printf '  destination: %s\n' "$installed_data"
printf '  excluded:    %s/data/reviews\n' "$repo_root"

transaction_complete=1
