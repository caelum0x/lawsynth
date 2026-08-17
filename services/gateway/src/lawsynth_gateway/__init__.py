"""LawSynth's transport-independent, in-process WSGI gateway."""

from .app import GatewayApplication, InProcessWsgiBackend, create_gateway
from .settings import GatewaySettings
from .sso import Principal, SsoAuthenticator, SsoError

__all__ = [
    "GatewayApplication",
    "GatewaySettings",
    "InProcessWsgiBackend",
    "Principal",
    "SsoAuthenticator",
    "SsoError",
    "create_gateway",
]
