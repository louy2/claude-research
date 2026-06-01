"""End-to-end pipeline: download PWT -> build parquets -> render HTML report.

    python src/run.py            # use cached raw file if present
    python src/run.py --force    # re-download the PWT workbook
"""
from __future__ import annotations

import sys

import fetch
import report
import transform


def main() -> None:
    force = "--force" in sys.argv
    fetch.download(force=force)
    transform.build()
    out = report.build()
    print(f"\nDone. Open: {out}")


if __name__ == "__main__":
    main()
