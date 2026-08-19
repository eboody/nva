#!/usr/bin/env python3
"""Convert a minicov profile captured from SpaceTimeDB WASM into LCOV.

Rust's wasm32 linker currently drops ``__llvm_prf_names`` while retaining
``__llvm_covfun`` records.  LLVM therefore cannot read the linked module
without reconstructing the profile-name section and discarding coverage
records whose translation-unit filename table was linker-collected.
"""

from __future__ import annotations

import argparse
import hashlib
import re
import subprocess
import sys
import tempfile
import zlib
from pathlib import Path


FUNCTION_LINE = re.compile(r"^  (.+):$")
MISSING_FILENAME = re.compile(
    r"no filename found for function with hash=0x([0-9a-fA-F]+)"
)


def read_uleb(data: bytes, offset: int) -> tuple[int, int]:
    value = 0
    shift = 0
    while True:
        byte = data[offset]
        offset += 1
        value |= (byte & 0x7F) << shift
        if byte < 0x80:
            return value, offset
        shift += 7


def write_uleb(value: int) -> bytes:
    encoded = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        encoded.append(byte | (0x80 if value else 0))
        if not value:
            return bytes(encoded)


def custom_section(name: bytes, payload: bytes) -> bytes:
    contents = write_uleb(len(name)) + name + payload
    return b"\0" + write_uleb(len(contents)) + contents


def coverage_records(section: bytes) -> list[bytes]:
    records: list[bytes] = []
    offset = 0
    while offset + 28 <= len(section):
        mapping_size = int.from_bytes(section[offset + 8 : offset + 12], "little")
        end = offset + 28 + mapping_size
        if end > len(section):
            raise ValueError("truncated __llvm_covfun record")
        padded_end = min((end + 7) // 8 * 8, len(section))
        records.append(section[offset:padded_end])
        offset = padded_end
    if offset != len(section):
        raise ValueError("trailing bytes in __llvm_covfun section")
    return records


def extract_custom_section(wasm: bytes, wanted: bytes) -> bytes:
    offset = 8
    while offset < len(wasm):
        section_id = wasm[offset]
        offset += 1
        size, payload_start = read_uleb(wasm, offset)
        payload_end = payload_start + size
        payload = wasm[payload_start:payload_end]
        if section_id == 0:
            name_size, name_start = read_uleb(payload, 0)
            name_end = name_start + name_size
            if payload[name_start:name_end] == wanted:
                return payload[name_end:]
        offset = payload_end
    raise ValueError(f"WASM custom section {wanted.decode()} is absent")


def rewrite_coverage_sections(
    wasm: bytes, profile_names: bytes, covfun: bytes
) -> bytes:
    if wasm[:8] != b"\0asm\1\0\0\0":
        raise ValueError("input is not a WebAssembly v1 module")

    output = bytearray(wasm[:8])
    offset = 8
    replaced = False
    while offset < len(wasm):
        section_start = offset
        section_id = wasm[offset]
        offset += 1
        size, payload_start = read_uleb(wasm, offset)
        payload_end = payload_start + size
        payload = wasm[payload_start:payload_end]
        if section_id == 0:
            name_size, name_start = read_uleb(payload, 0)
            name_end = name_start + name_size
            name = payload[name_start:name_end]
            if name == b"__llvm_covfun":
                output.extend(custom_section(b"__llvm_prf_names", profile_names))
                output.extend(custom_section(name, covfun))
                replaced = True
                offset = payload_end
                continue
        output.extend(wasm[section_start:payload_end])
        offset = payload_end

    if not replaced:
        raise ValueError("WASM custom section __llvm_covfun is absent")
    return bytes(output)


def llvm_tool(toolchain: str, name: str) -> Path:
    result = subprocess.run(
        ["rustc", f"+{toolchain}", "--print", "target-libdir"],
        check=True,
        capture_output=True,
        text=True,
    )
    path = Path(result.stdout.strip()).parent / "bin" / name
    if not path.is_file():
        raise FileNotFoundError(
            f"{name} is unavailable in {toolchain}; install the llvm-tools component"
        )
    return path


def profile_function_names(llvm_profdata: Path, profdata: Path) -> list[str]:
    result = subprocess.run(
        [str(llvm_profdata), "show", "--all-functions", str(profdata)],
        check=True,
        capture_output=True,
        text=True,
    )
    names = [
        match.group(1)
        for line in result.stdout.splitlines()
        if (match := FUNCTION_LINE.match(line)) is not None
    ]
    if not names:
        raise ValueError("captured profile contains no instrumented functions")
    return names


def encoded_profile_names(names: list[str]) -> bytes:
    raw = b"\x01".join(name.encode() for name in names)
    compressed = zlib.compress(raw, 9)
    return write_uleb(len(raw)) + write_uleb(len(compressed)) + compressed


def extract_profraw(sql_output: Path, profraw: Path) -> None:
    candidates = re.findall(r"(?i)\b0x([0-9a-f]{1000,})\b", sql_output.read_text())
    if not candidates:
        raise ValueError("SpaceTimeDB SQL output contains no coverage-profile bytes")
    profraw.write_bytes(bytes.fromhex(max(candidates, key=len)))


def export_lcov(
    *, wasm_path: Path, profraw: Path, output: Path, toolchain: str
) -> None:
    llvm_profdata = llvm_tool(toolchain, "llvm-profdata")
    llvm_cov = llvm_tool(toolchain, "llvm-cov")

    with tempfile.TemporaryDirectory(prefix="nva-wasm-coverage-") as temporary:
        temp = Path(temporary)
        profdata = temp / "coverage.profdata"
        subprocess.run(
            [str(llvm_profdata), "merge", "-sparse", str(profraw), "-o", str(profdata)],
            check=True,
        )
        names = profile_function_names(llvm_profdata, profdata)
        known_name_hashes = {
            int.from_bytes(hashlib.md5(name.encode()).digest()[:8], "little")
            for name in names
        }
        original_wasm = wasm_path.read_bytes()
        records = [
            record
            for record in coverage_records(
                extract_custom_section(original_wasm, b"__llvm_covfun")
            )
            if int.from_bytes(record[:8], "little") in known_name_hashes
        ]
        if not records:
            raise ValueError("no linked coverage records match the captured profile")

        profile_names = encoded_profile_names(names)
        rewritten_wasm = temp / "coverage-map.wasm"
        removed_filename_hashes: set[int] = set()
        for _ in range(len(records) + 1):
            rewritten_wasm.write_bytes(
                rewrite_coverage_sections(
                    original_wasm, profile_names, b"".join(records)
                )
            )
            with output.open("w") as lcov:
                result = subprocess.run(
                    [
                        str(llvm_cov),
                        "export",
                        "-format=lcov",
                        f"-instr-profile={profdata}",
                        str(rewritten_wasm),
                    ],
                    stdout=lcov,
                    stderr=subprocess.PIPE,
                    text=True,
                )
            if result.returncode == 0:
                print(
                    "SpaceTimeDB WASM coverage: "
                    f"{len(records)} function records, "
                    f"{len(removed_filename_hashes)} linker-collected filename tables skipped",
                    file=sys.stderr,
                )
                return
            missing = MISSING_FILENAME.search(result.stderr)
            if missing is None:
                raise RuntimeError(result.stderr.strip())
            filename_hash = int(missing.group(1), 16)
            retained = [
                record
                for record in records
                if int.from_bytes(record[20:28], "little") != filename_hash
            ]
            if len(retained) == len(records):
                raise RuntimeError(result.stderr.strip())
            records = retained
            removed_filename_hashes.add(filename_hash)

    raise RuntimeError("could not reconcile linked WASM coverage records")


def main() -> int:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)

    extract = subparsers.add_parser("extract-profraw")
    extract.add_argument("--sql-output", type=Path, required=True)
    extract.add_argument("--output", type=Path, required=True)

    export = subparsers.add_parser("export-lcov")
    export.add_argument("--wasm", type=Path, required=True)
    export.add_argument("--profraw", type=Path, required=True)
    export.add_argument("--output", type=Path, required=True)
    export.add_argument("--toolchain", default="nightly-2026-08-15")

    args = parser.parse_args()
    if args.command == "extract-profraw":
        extract_profraw(args.sql_output, args.output)
    else:
        export_lcov(
            wasm_path=args.wasm,
            profraw=args.profraw,
            output=args.output,
            toolchain=args.toolchain,
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
