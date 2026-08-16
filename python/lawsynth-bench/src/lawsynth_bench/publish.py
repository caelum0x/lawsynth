"""Safe local publication of immutable benchmark report documents."""
from pathlib import Path
import json, os, tempfile
from typing import Mapping

def write_report(report: Mapping[str, object], destination: str | Path) -> Path:
    path = Path(destination); path.parent.mkdir(parents=True, exist_ok=True)
    temporary: Path | None = None
    try:
        with tempfile.NamedTemporaryFile("w", encoding="utf-8", dir=path.parent, delete=False) as handle:
            temporary = Path(handle.name)
            json.dump(report, handle, sort_keys=True, indent=2, allow_nan=False); handle.write("\n")
        os.replace(temporary, path)
    except Exception:
        if temporary is not None: temporary.unlink(missing_ok=True)
        raise
    return path
