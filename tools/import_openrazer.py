#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later

"""Generate RazeRS's evidence-only device catalog from a pinned OpenRazer tree."""

from __future__ import annotations

import argparse
import ast
import json
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any


SCHEMA_VERSION = 1
SOURCE_REPOSITORY = "openrazer/openrazer"
SOURCE_COMMIT = "6820f9da169d354bc7e6e93a0aa8683a6bb75792"
SOURCE_LICENSE = "GPL-2.0-or-later"
HARDWARE_PATH = Path("daemon/openrazer_daemon/hardware")
SOURCE_FILES = (
    "accessory.py",
    "core.py",
    "headsets.py",
    "keyboards.py",
    "monitor.py",
    "mouse.py",
    "mouse_mat.py",
)


@dataclass(frozen=True)
class Device:
    name: str
    kind: str
    vid: int
    pid: int
    source_path: str
    source_symbol: str
    upstream_features: tuple[str, ...]
    methods: tuple[str, ...]
    matrix: tuple[int, int] | None
    max_dpi: int | None
    poll_rates_hz: tuple[int, ...]


class ModuleResolver:
    """Resolve the small, literal subset used by OpenRazer device classes."""

    def __init__(self, module: ast.Module):
        self.classes = {
            node.name: node for node in module.body if isinstance(node, ast.ClassDef)
        }
        self.cache: dict[tuple[str, str], Any] = {}

    def value(self, class_name: str, attribute: str, seen: tuple[str, ...] = ()) -> Any:
        key = (class_name, attribute)
        if key in self.cache:
            return self.cache[key]
        if class_name in seen:
            return None

        class_node = self.classes.get(class_name)
        if class_node is None:
            return None

        next_seen = (*seen, class_name)
        for statement in class_node.body:
            expression = None
            if isinstance(statement, ast.Assign) and any(
                isinstance(target, ast.Name) and target.id == attribute
                for target in statement.targets
            ):
                expression = statement.value
            elif (
                isinstance(statement, ast.AnnAssign)
                and isinstance(statement.target, ast.Name)
                and statement.target.id == attribute
            ):
                expression = statement.value

            if expression is not None:
                result = self._expression(expression, next_seen)
                self.cache[key] = result
                return result

        for base in class_node.bases:
            if isinstance(base, ast.Name):
                result = self.value(base.id, attribute, next_seen)
                if result is not None:
                    self.cache[key] = result
                    return result
        return None

    def _expression(self, expression: ast.expr, seen: tuple[str, ...]) -> Any:
        try:
            return ast.literal_eval(expression)
        except (ValueError, TypeError):
            pass

        if (
            isinstance(expression, ast.Attribute)
            and isinstance(expression.value, ast.Name)
        ):
            return self.value(expression.value.id, expression.attr, seen)

        if isinstance(expression, ast.BinOp) and isinstance(expression.op, ast.Add):
            left = self._expression(expression.left, seen)
            right = self._expression(expression.right, seen)
            if isinstance(left, (list, tuple)) and isinstance(right, (list, tuple)):
                return [*left, *right]
        return None


def display_name(class_node: ast.ClassDef) -> str:
    docstring = ast.get_docstring(class_node) or ""
    first_line = docstring.strip().splitlines()[0] if docstring.strip() else ""
    match = re.fullmatch(r"Class for (?:the )?(.+)", first_line)
    if match:
        return match.group(1).rstrip(".")

    words = re.sub(r"(?<=[a-z0-9])(?=[A-Z])", " ", class_node.name)
    words = re.sub(r"(?<=[A-Z])(?=[A-Z][a-z])", " ", words)
    return words


def device_kind(filename: str, methods: tuple[str, ...]) -> str:
    advertised = next(
        (method.removeprefix("get_device_type_") for method in methods if method.startswith("get_device_type_")),
        None,
    )
    if advertised:
        return {"mousemat": "mouse-mat", "core": "core"}.get(advertised, advertised)
    return {
        "accessory.py": "accessory",
        "core.py": "core",
        "headsets.py": "headset",
        "keyboards.py": "keyboard",
        "monitor.py": "monitor",
        "mouse.py": "mouse",
        "mouse_mat.py": "mouse-mat",
    }[filename]


def upstream_features(methods: tuple[str, ...]) -> tuple[str, ...]:
    features = {"identity"}
    if any("dpi" in method for method in methods):
        features.add("dpi")
    if any("poll_rate" in method for method in methods):
        features.add("polling-rate")
    if any(
        "effect" in method
        or "brightness" in method
        or method in {"set_key_row", "set_logo_on", "set_scroll_on"}
        for method in methods
    ):
        features.add("lighting")
    if any(method in {"get_battery", "is_charging"} for method in methods):
        features.add("battery")
    if any("macro" in method for method in methods):
        features.add("macro")
    if any("game_mode" in method for method in methods):
        features.add("game-mode")
    if any("scroll_mode" in method or "scroll_acceleration" in method for method in methods):
        features.add("scroll-mode")
    if any("keyboard_layout" in method for method in methods):
        features.add("layout")
    return tuple(sorted(features))


def load_devices(source: Path) -> list[Device]:
    hardware = source / HARDWARE_PATH
    if not hardware.is_dir():
        raise SystemExit(f"OpenRazer hardware directory not found: {hardware}")

    devices = []
    for filename in SOURCE_FILES:
        path = hardware / filename
        module = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        resolver = ModuleResolver(module)
        for class_name, class_node in resolver.classes.items():
            pid = resolver.value(class_name, "USB_PID")
            if pid is None:
                continue
            vid = resolver.value(class_name, "USB_VID")
            methods = resolver.value(class_name, "METHODS")
            if not isinstance(vid, int) or not isinstance(pid, int):
                raise SystemExit(f"{filename}:{class_name} has an unresolved USB identity")
            if not isinstance(methods, (list, tuple)) or not all(
                isinstance(method, str) for method in methods
            ):
                raise SystemExit(f"{filename}:{class_name} has unresolved METHODS")

            matrix_value = resolver.value(class_name, "MATRIX_DIMS")
            matrix = None
            if matrix_value is not None:
                if (
                    not isinstance(matrix_value, (list, tuple))
                    or len(matrix_value) != 2
                    or not all(isinstance(value, int) for value in matrix_value)
                ):
                    raise SystemExit(f"{filename}:{class_name} has invalid MATRIX_DIMS")
                matrix = (matrix_value[0], matrix_value[1])

            max_dpi = resolver.value(class_name, "DPI_MAX")
            if max_dpi is not None and not isinstance(max_dpi, int):
                raise SystemExit(f"{filename}:{class_name} has invalid DPI_MAX")

            poll_rates = resolver.value(class_name, "POLL_RATES") or ()
            if not isinstance(poll_rates, (list, tuple)) or not all(
                isinstance(rate, int) for rate in poll_rates
            ):
                raise SystemExit(f"{filename}:{class_name} has invalid POLL_RATES")

            method_tuple = tuple(dict.fromkeys(methods))
            devices.append(
                Device(
                    name=display_name(class_node),
                    kind=device_kind(filename, method_tuple),
                    vid=vid,
                    pid=pid,
                    source_path=str(HARDWARE_PATH / filename),
                    source_symbol=class_name,
                    upstream_features=upstream_features(method_tuple),
                    methods=method_tuple,
                    matrix=matrix,
                    max_dpi=max_dpi,
                    poll_rates_hz=tuple(poll_rates),
                )
            )

    devices.sort(key=lambda device: (device.kind, device.name.casefold(), device.vid, device.pid))
    identities = [(device.vid, device.pid) for device in devices]
    if len(identities) != len(set(identities)):
        raise SystemExit("OpenRazer source contains duplicate VID/PID identities")
    return devices


def verify_source(source: Path) -> None:
    result = subprocess.run(
        ["git", "-C", str(source), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    )
    actual_commit = result.stdout.strip()
    if actual_commit != SOURCE_COMMIT:
        raise SystemExit(
            f"OpenRazer checkout is at {actual_commit}; expected pinned commit {SOURCE_COMMIT}"
        )


def toml_string(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def toml_strings(values: tuple[str, ...]) -> str:
    return "[" + ", ".join(toml_string(value) for value in values) + "]"


def render(devices: list[Device]) -> str:
    lines = [
        "# SPDX-License-Identifier: GPL-2.0-or-later",
        "# Generated by tools/import_openrazer.py; do not edit by hand.",
        "# This is upstream evidence, not a RazeRS hardware-support claim.",
        f"schema_version = {SCHEMA_VERSION}",
        "",
        "[source]",
        'name = "OpenRazer"',
        f"repository = {toml_string(SOURCE_REPOSITORY)}",
        f"commit = {toml_string(SOURCE_COMMIT)}",
        f"path = {toml_string(str(HARDWARE_PATH))}",
        f"license = {toml_string(SOURCE_LICENSE)}",
        'generated_by = "tools/import_openrazer.py"',
    ]

    for device in devices:
        lines.extend(
            [
                "",
                "[[devices]]",
                f"name = {toml_string(device.name)}",
                f"kind = {toml_string(device.kind)}",
                f"vid = 0x{device.vid:04x}",
                f"pid = 0x{device.pid:04x}",
                f"source_path = {toml_string(device.source_path)}",
                f"source_symbol = {toml_string(device.source_symbol)}",
                f"upstream_features = {toml_strings(device.upstream_features)}",
                f"methods = {toml_strings(device.methods)}",
            ]
        )
        if device.matrix is not None:
            lines.append(f"matrix = [{device.matrix[0]}, {device.matrix[1]}]")
        if device.max_dpi is not None:
            lines.append(f"max_dpi = {device.max_dpi}")
        if device.poll_rates_hz:
            rates = ", ".join(str(rate) for rate in device.poll_rates_hz)
            lines.append(f"poll_rates_hz = [{rates}]")

    return "\n".join(lines) + "\n"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, required=True, help="OpenRazer checkout")
    parser.add_argument("--output", type=Path, required=True, help="catalog output path")
    args = parser.parse_args()

    verify_source(args.source)
    devices = load_devices(args.source)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(render(devices), encoding="utf-8")
    print(f"imported {len(devices)} OpenRazer device identities into {args.output}")


if __name__ == "__main__":
    main()
