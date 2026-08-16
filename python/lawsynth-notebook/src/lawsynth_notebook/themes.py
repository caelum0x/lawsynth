"""Named palettes used by all generated HTML."""

from .errors import ArtifactValidationError

PALETTES = {
    "light": {"background": "#ffffff", "foreground": "#172033", "muted": "#53627a", "border": "#cbd5e1", "accent": "#155e75"},
    "dark": {"background": "#111827", "foreground": "#e5e7eb", "muted": "#a5b4cc", "border": "#374151", "accent": "#67e8f9"},
}


def palette(name: str) -> dict[str, str]:
    try:
        return dict(PALETTES[name])
    except KeyError as error:
        raise ArtifactValidationError(f"unknown notebook theme {name!r}") from error
