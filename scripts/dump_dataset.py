"""dump_dataset.py – Run the crawler once and save the raw rows as JSON.

Usage:
    .venv/bin/python scripts/dump_dataset.py [output]

Default output: data/sample_dataset.json (gitignored – contains personal data).
"""

from __future__ import annotations

import json
import logging
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from run_hour import collect, _load_dotenv  # noqa: E402


def main() -> None:
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s [%(levelname)s] %(name)s – %(message)s",
        stream=sys.stdout,
    )
    _load_dotenv()
    output = Path(sys.argv[1] if len(sys.argv) > 1 else "data/sample_dataset.json")
    data = collect()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(data, ensure_ascii=False, indent=2, default=str), encoding="utf-8")
    print(f"Wrote dataset sample to {output}")
    for key, value in data.items():
        state = "FAILED" if value is None else f"{len(value)} rows"
        print(f"  {key}: {state}")


if __name__ == "__main__":
    main()
