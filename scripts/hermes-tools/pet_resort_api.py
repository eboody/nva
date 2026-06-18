#!/usr/bin/env python3
"""Small stdlib HTTP client for Hermes-to-pet-resort app bridge scripts."""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any
from urllib import error, parse, request

DEFAULT_API_URL = "http://127.0.0.1:3001"


class BridgeError(Exception):
    """Safe, user-facing bridge error."""


def api_base_url() -> str:
    return os.environ.get("PET_RESORT_API_URL", DEFAULT_API_URL).rstrip("/")


def auth_headers() -> dict[str, str]:
    token = os.environ.get("PET_RESORT_API_TOKEN")
    if not token:
        return {}
    return {"Authorization": f"Bearer {token}"}


def endpoint_url(path: str, query: dict[str, str] | None = None) -> str:
    url = f"{api_base_url()}/{path.lstrip('/')}"
    if query:
        return f"{url}?{parse.urlencode(query)}"
    return url


def request_json(method: str, path: str, *, query: dict[str, str] | None = None, body: Any = None) -> Any:
    data = None
    headers = {"Accept": "application/json", **auth_headers()}
    if body is not None:
        data = json.dumps(body, separators=(",", ":")).encode("utf-8")
        headers["Content-Type"] = "application/json"

    req = request.Request(endpoint_url(path, query), data=data, headers=headers, method=method)
    try:
        with request.urlopen(req, timeout=30) as response:
            return _decode_json_response(response.read())
    except error.HTTPError as exc:
        # Do not print URL, headers, tokens, or raw upstream body; any of those may contain secrets/PII.
        raise BridgeError(f"pet resort api request failed: HTTP {exc.code} from app API") from None
    except error.URLError as exc:
        reason = exc.reason.__class__.__name__
        raise BridgeError(f"pet resort api request failed: could not reach app API ({reason})") from None
    except TimeoutError:
        raise BridgeError("pet resort api request failed: app API timed out") from None


def _decode_json_response(raw: bytes) -> Any:
    try:
        return json.loads(raw.decode("utf-8"))
    except json.JSONDecodeError as exc:
        raise BridgeError(f"pet resort api request failed: app API returned invalid JSON at byte {exc.pos}") from None


def read_json_payload(path: str | None = None) -> Any:
    try:
        if path:
            text = Path(path).read_text(encoding="utf-8")
        else:
            text = sys.stdin.read()
        return json.loads(text)
    except OSError as exc:
        raise BridgeError(f"could not read JSON payload file: {exc.__class__.__name__}") from None
    except json.JSONDecodeError as exc:
        raise BridgeError(f"invalid JSON payload at byte {exc.pos}") from None


def write_json(payload: Any) -> None:
    sys.stdout.write(json.dumps(payload, indent=2, sort_keys=True))
    sys.stdout.write("\n")


def fail_safe(exc: Exception) -> int:
    message = str(exc) if isinstance(exc, BridgeError) else exc.__class__.__name__
    print(message, file=sys.stderr)
    return 1


def add_payload_file_argument(parser: argparse.ArgumentParser, name: str) -> None:
    parser.add_argument(
        "--payload-file",
        "--draft-file" if name == "draft" else "--outcome-file",
        dest="payload_file",
        help=f"Read {name} JSON from a file instead of stdin.",
    )
