from __future__ import annotations
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
from _performance import script_main
if __name__ == "__main__":
    raise SystemExit(script_main(Path(__file__).resolve().parent, "generate"))
