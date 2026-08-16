import pytest

from lawsynth_server.database import Database


def test_transaction_rolls_back():
    db = Database()
    db.connection.execute("CREATE TABLE x (value INTEGER)")
    with pytest.raises(RuntimeError):
        with db.transaction() as connection:
            connection.execute("INSERT INTO x VALUES (1)")
            raise RuntimeError("abort")
    assert db.connection.execute("SELECT count(*) FROM x").fetchone() == (0,)
    db.close()


def test_non_sqlite_database_requires_a_deployment_adapter():
    with pytest.raises(ValueError):
        Database("postgresql://server/lawsynth")
