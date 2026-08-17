#!/usr/bin/env python3
"""Generate LawSynth's brand raster assets deterministically.

These images are not decorative filler: the hero/social art renders the Lorenz
system that LawSynth discovers, drawn from a fixed-seed RK4 integration, and the
palette/typography come straight from ``assets/brand/palette.json`` and
``typography.md``. Re-running this script reproduces byte-stable art (no clock,
no RNG), so the committed assets can always be regenerated.

Requires Pillow. Fonts resolve to the system Georgia (brand display serif) and
Helvetica (interface), with graceful fallback to Pillow's default bitmap font.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[2]
README = ROOT / "assets" / "readme"
SOCIAL = ROOT / "assets" / "social"

# Brand tokens (assets/brand/palette.json).
INK = (24, 32, 29)
PAPER = (243, 240, 232)
SURFACE = (255, 253, 247)
LINE = (200, 198, 186)
MUTED = (89, 99, 94)
ACCENT = (181, 75, 42)
ACCENT_SOFT = (229, 195, 180)
SUCCESS = (47, 111, 79)

GEORGIA = "/System/Library/Fonts/Supplemental/Georgia.ttf"
GEORGIA_BOLD = "/System/Library/Fonts/Supplemental/Georgia Bold.ttf"
HELVETICA = "/System/Library/Fonts/Helvetica.ttc"


def font(path: str, size: int) -> ImageFont.FreeTypeFont:
    try:
        return ImageFont.truetype(path, size)
    except OSError:
        return ImageFont.load_default()


# --- Lorenz system (the law LawSynth is built to recover) --------------------

def lorenz(n: int, dt: float = 0.008, sigma: float = 10.0, rho: float = 28.0,
           beta: float = 8.0 / 3.0) -> list[tuple[float, float, float]]:
    """Fixed-seed RK4 integration of the Lorenz attractor — pure Python."""

    def deriv(s: tuple[float, float, float]) -> tuple[float, float, float]:
        x, y, z = s
        return (sigma * (y - x), x * (rho - z) - y, x * y - beta * z)

    def step(s: tuple[float, float, float]) -> tuple[float, float, float]:
        k1 = deriv(s)
        k2 = deriv(tuple(s[i] + 0.5 * dt * k1[i] for i in range(3)))
        k3 = deriv(tuple(s[i] + 0.5 * dt * k2[i] for i in range(3)))
        k4 = deriv(tuple(s[i] + dt * k3[i] for i in range(3)))
        return tuple(s[i] + (dt / 6.0) * (k1[i] + 2 * k2[i] + 2 * k3[i] + k4[i])
                     for i in range(3))

    state = (1.0, 1.0, 1.0)
    points = [state]
    for _ in range(n):
        state = step(state)
        points.append(state)
    return points


def project(points: list[tuple[float, float, float]], w: int, h: int,
            pad: int) -> list[tuple[float, float]]:
    """Project the x/z plane of the attractor into padded pixel space."""
    xs = [p[0] for p in points]
    zs = [p[2] for p in points]
    x0, x1 = min(xs), max(xs)
    z0, z1 = min(zs), max(zs)
    sx = (w - 2 * pad) / (x1 - x0)
    sz = (h - 2 * pad) / (z1 - z0)
    scale = min(sx, sz)
    cx = (w - scale * (x1 - x0)) / 2
    cz = (h - scale * (z1 - z0)) / 2
    return [(cx + (x - x0) * scale, h - (cz + (z - z0) * scale)) for x, _, z in points]


def blend(a: tuple[int, int, int], b: tuple[int, int, int], t: float) -> tuple[int, int, int]:
    return tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))


def draw_attractor(draw: ImageDraw.ImageDraw, pts: list[tuple[float, float]],
                   count: int, base: tuple[int, int, int], head: tuple[int, int, int],
                   width: int = 2) -> None:
    """Draw the first `count` segments, fading the tail toward the accent head."""
    n = max(count, 2)
    for i in range(1, n):
        t = i / n
        draw.line([pts[i - 1], pts[i]], fill=blend(base, head, t * t), width=width)


# --- Static compositions -----------------------------------------------------

def wordmark(draw: ImageDraw.ImageDraw, x: int, y: int, size: int) -> None:
    law = font(GEORGIA_BOLD, size)
    draw.text((x, y), "Law", font=law, fill=INK)
    law_w = draw.textlength("Law", font=law)
    draw.text((x + law_w, y), "Synth", font=law, fill=ACCENT)


def hero(path: Path) -> None:
    w, h = 1600, 900
    img = Image.new("RGB", (w, h), PAPER)
    d = ImageDraw.Draw(img)
    pts = project(lorenz(6000), w, h, 80)
    draw_attractor(d, pts, len(pts), blend(PAPER, INK, 0.72), ACCENT, width=2)
    # Editorial framing.
    d.rectangle([40, 40, w - 40, h - 40], outline=LINE, width=2)
    wordmark(d, 96, 96, 132)
    d.text((100, 250), "Turn time-series observations into executable worlds.",
           font=font(GEORGIA, 46), fill=INK)
    d.text((100, 320), "Deterministic discovery · simulation · .lsworld bundles",
           font=font(HELVETICA, 30), fill=MUTED)
    img.save(path, "WEBP", quality=90, method=6)


def studio(path: Path) -> None:
    w, h = 1600, 1000
    img = Image.new("RGB", (w, h), PAPER)
    d = ImageDraw.Draw(img)
    # A calm, stylised Studio layout (not a screenshot — a brand-accurate mock).
    d.rectangle([0, 0, w, 72], fill=SURFACE)
    d.line([0, 72, w, 72], fill=LINE, width=2)
    wordmark(d, 28, 18, 40)
    for i, label in enumerate(("Data", "Discovery", "World", "Regimes", "Export")):
        d.text((360 + i * 150, 26), label, font=font(HELVETICA, 24),
               fill=ACCENT if i == 1 else MUTED)
    # Left rail.
    d.rectangle([0, 72, 280, h], fill=SURFACE)
    d.line([280, 72, 280, h], fill=LINE, width=2)
    for i in range(6):
        y = 110 + i * 54
        d.rectangle([24, y, 256, y + 36], fill=blend(PAPER, ACCENT_SOFT, 0.4 if i == 1 else 0.0),
                    outline=LINE)
    # Canvas: the discovered attractor on a surface card.
    d.rectangle([320, 110, w - 40, h - 220], fill=SURFACE, outline=LINE, width=2)
    pts = project(lorenz(5000), w - 40 - 320, h - 220 - 110, 60)
    pts = [(px + 320, py + 110) for px, py in pts]
    draw_attractor(d, pts, len(pts), blend(SURFACE, INK, 0.7), ACCENT, width=2)
    # Bottom equation strip.
    d.rectangle([320, h - 200, w - 40, h - 40], fill=SURFACE, outline=LINE, width=2)
    d.text((344, h - 184), "recovered law", font=font(HELVETICA, 22), fill=MUTED)
    mono = font(HELVETICA, 30)
    d.text((344, h - 148), "x' = 10.0 (y - x)", font=mono, fill=INK)
    d.text((344, h - 108), "y' = x (28.0 - z) - y", font=mono, fill=INK)
    d.text((344, h - 68), "z' = x y - 2.667 z", font=mono, fill=INK)
    img.save(path, "WEBP", quality=90, method=6)


def social_card(path: Path, size: tuple[int, int], title_size: int, tagline: str,
                fmt: str = "PNG") -> None:
    w, h = size
    img = Image.new("RGB", (w, h), PAPER)
    d = ImageDraw.Draw(img)
    # Attractor motif on the right third.
    motif_w = w // 2
    pts = project(lorenz(4500), motif_w, h, 60)
    pts = [(px + (w - motif_w), py) for px, py in pts]
    draw_attractor(d, pts, len(pts), blend(PAPER, INK, 0.6), ACCENT, width=2)
    d.rectangle([28, 28, w - 28, h - 28], outline=LINE, width=2)
    wordmark(d, 72, int(h * 0.30), title_size)
    d.text((76, int(h * 0.30) + title_size + 24), tagline,
           font=font(GEORGIA, max(24, title_size // 3)), fill=MUTED)
    d.text((76, h - 90), "Apache-2.0 · local-first · reproducible",
           font=font(HELVETICA, max(18, title_size // 5)), fill=INK)
    img.save(path, fmt)


def lorenz_gif(path: Path) -> None:
    w, h = 640, 400
    pts = project(lorenz(4000), w, h, 44)
    frames = []
    total = len(pts)
    steps = 60
    base = blend(PAPER, INK, 0.62)
    for f in range(steps + 1):
        count = max(2, round(total * (f / steps)))
        img = Image.new("RGB", (w, h), PAPER)
        d = ImageDraw.Draw(img)
        d.rectangle([12, 12, w - 12, h - 12], outline=LINE, width=2)
        draw_attractor(d, pts, count, base, ACCENT, width=2)
        # Accent head marking the current state.
        hx, hy = pts[count - 1]
        d.ellipse([hx - 4, hy - 4, hx + 4, hy + 4], fill=ACCENT)
        wm = font(GEORGIA_BOLD, 26)
        d.text((24, h - 44), "Law", font=wm, fill=INK)
        d.text((24 + d.textlength("Law", font=wm), h - 44), "Synth", font=wm, fill=ACCENT)
        frames.append(img.convert("P", palette=Image.ADAPTIVE, colors=64))
    # Hold the final frame a little longer before looping.
    durations = [60] * steps + [1200]
    frames[0].save(path, save_all=True, append_images=frames[1:], loop=0,
                   duration=durations, optimize=True, disposal=2)


def main() -> None:
    README.mkdir(parents=True, exist_ok=True)
    SOCIAL.mkdir(parents=True, exist_ok=True)
    hero(README / "hero.webp")
    studio(README / "studio.webp")
    lorenz_gif(README / "lorenz-demo.gif")
    social_card(SOCIAL / "github-card.png", (1280, 640), 120,
                "Executable mathematical worlds from data.")
    social_card(SOCIAL / "announcement.png", (1200, 675), 108,
                "Now open source — discover the laws in your data.")
    social_card(SOCIAL / "demo-thumbnail.png", (1280, 720), 128,
                "Watch a world get discovered.")
    print("generated brand assets under assets/readme and assets/social")


if __name__ == "__main__":
    main()
