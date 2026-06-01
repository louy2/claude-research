"""Download the Penn World Table 11.0 workbook.

DataverseNL's `/api/access/datafile/<id>` endpoint returns a 303 redirect to a
time-limited signed object-store URL. We resolve the redirect and fetch the
target in a single short-lived step, with exponential-backoff retries for the
occasional upstream reset.
"""
from __future__ import annotations

import time
import urllib.request
from urllib.error import URLError, HTTPError

from config import PWT_DATAFILE_URL, RAW_DIR, RAW_XLSX

_XLSX_MAGIC = b"PK\x03\x04"  # xlsx is a zip container


def _resolve_signed_url(url: str) -> str:
    """Follow the DataverseNL 303 to its signed object-store location."""
    req = urllib.request.Request(url, method="HEAD")

    class _NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *a, **k):  # noqa: D401
            return None

    opener = urllib.request.build_opener(_NoRedirect)
    try:
        opener.open(req, timeout=30)
        return url  # no redirect -> use as-is
    except HTTPError as e:
        loc = e.headers.get("Location")
        if e.code in (301, 302, 303, 307, 308) and loc:
            return loc
        raise


def download(force: bool = False, retries: int = 5) -> str:
    RAW_DIR.mkdir(parents=True, exist_ok=True)
    if RAW_XLSX.exists() and not force:
        print(f"[fetch] cached: {RAW_XLSX}")
        return str(RAW_XLSX)

    last_err: Exception | None = None
    for attempt in range(1, retries + 1):
        try:
            signed = _resolve_signed_url(PWT_DATAFILE_URL)
            with urllib.request.urlopen(signed, timeout=180) as r:
                data = r.read()
            if not data.startswith(_XLSX_MAGIC):
                raise ValueError(f"unexpected payload ({len(data)} bytes, not xlsx)")
            RAW_XLSX.write_bytes(data)
            print(f"[fetch] saved {len(data):,} bytes -> {RAW_XLSX}")
            return str(RAW_XLSX)
        except (URLError, HTTPError, ValueError, TimeoutError) as e:
            last_err = e
            wait = 2 ** attempt
            print(f"[fetch] attempt {attempt} failed ({e}); retry in {wait}s")
            time.sleep(wait)
    raise RuntimeError(f"failed to download PWT after {retries} attempts: {last_err}")


if __name__ == "__main__":
    download()
