"""Runnable example: read a time-series table from a DuckDB file.

    pip install duckdb
    python plugins/duckdb-source/examples/basic.py

If duckdb is not installed the example prints an explanatory message and exits
cleanly instead of crashing.
"""

from __future__ import annotations

import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from duckdb_source.plugin import DuckDBSource  # noqa: E402


def main() -> None:
    try:
        import duckdb
    except ImportError:
        print("duckdb is not installed. Install with: pip install duckdb")
        return

    with tempfile.TemporaryDirectory() as tmp:
        database = Path(tmp) / "observations.duckdb"

        # Seed a small numeric time series.
        connection = duckdb.connect(str(database))
        connection.execute("CREATE TABLE observations (time DOUBLE, x DOUBLE, y DOUBLE)")
        connection.executemany(
            "INSERT INTO observations VALUES (?, ?, ?)",
            [(0.0, 1.0, 2.0), (1.0, 1.5, 2.5), (2.0, 2.25, 3.1)],
        )
        connection.close()

        source = DuckDBSource(batch_size=2)
        result = source.invoke(
            {
                "database": str(database),
                "query": "SELECT time, x, y FROM observations ORDER BY time",
            }
        )

        print("columns  :", result["columns"])
        print("row_count:", result["row_count"])
        for row in result["records"]:
            print(row)


if __name__ == "__main__":
    main()
