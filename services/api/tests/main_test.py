"""Unit tests for the WSGI import target and dev-server CLI guards."""

from __future__ import annotations

import pytest

from lawsynth_api.app import WsgiApplication
from lawsynth_api.main import _loopback, application, main


def test_module_exposes_a_wsgi_application_target():
    assert isinstance(application, WsgiApplication)


def test_loopback_recognizes_local_hosts():
    assert _loopback("127.0.0.1")
    assert _loopback("::1")
    assert _loopback("localhost")


def test_loopback_rejects_public_hosts():
    assert not _loopback("0.0.0.0")
    assert not _loopback("192.168.1.10")
    assert not _loopback("example.com")


def test_main_rejects_non_loopback_host():
    with pytest.raises(SystemExit):
        main(["--host", "0.0.0.0"])


def test_main_rejects_out_of_range_port():
    with pytest.raises(SystemExit):
        main(["--port", "70000"])
