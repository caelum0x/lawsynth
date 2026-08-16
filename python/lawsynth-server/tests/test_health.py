from lawsynth_server.database import Database
from lawsynth_server.health import check
from lawsynth_server.storage import FileObjectStore


def test_health_checks_local_adapters(tmp_path):
    db = Database()
    assert check(db, FileObjectStore(tmp_path, max_bytes=1)).status == "ok"
    db.close()
