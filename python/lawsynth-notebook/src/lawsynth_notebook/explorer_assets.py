"""The self-contained JavaScript shipped inside a :class:`WorldExplorerWidget`.

``INTEGRATOR_JS`` is a single, dependency-free script body that:

* tokenises and parses the world's parameterised law expressions into an AST
  (const / var / add / sub / mul / div / pow / neg / unary functions),
* evaluates that AST over a numeric scope,
* integrates the law system with a fixed-step RK4 (or Euler) scheme,
* renders an inline-SVG trajectory chart, and
* wires parameter sliders, initial-condition inputs and play/reset so every
  interaction re-simulates and redraws entirely in the browser.

It reads two constants — ``PAYLOAD`` (the world-as-JSON) and ``ROOT_ID`` (the
container element id) — that the view injects immediately before this body.
There is no ``fetch``/``import``/network use, no external library, and no
kernel round-trip: the whole model lives in the bundle. The scheme mirrors
:mod:`lawsynth_notebook.explorer_math` sample-for-sample.
"""

from __future__ import annotations

__all__ = ["INTEGRATOR_JS"]

# NOTE: intentionally contains no "</script" sequence (the only token that can
# terminate a <script> in HTML5, so this body survives HTML/CDATA parsing) and
# no external URL of any kind.
INTEGRATOR_JS = r"""
var SERIES_COLORS = ["#2563eb", "#dc2626", "#059669", "#d97706", "#7c3aed", "#0891b2", "#db2777", "#65a30d"];
var FUNCS1 = {
  abs: Math.abs, exp: Math.exp, log: Math.log, sqrt: Math.sqrt,
  sin: Math.sin, cos: Math.cos, tan: Math.tan, neg: function (v) { return -v; }
};
var FUNCS2 = { min: Math.min, max: Math.max, pow: Math.pow };

function isIdentStart(ch) { return (ch >= "a" && ch <= "z") || (ch >= "A" && ch <= "Z") || ch === "_"; }
function isIdentPart(ch) { return isIdentStart(ch) || (ch >= "0" && ch <= "9"); }
function isDigit(ch) { return ch >= "0" && ch <= "9"; }

function tokenize(src) {
  var tokens = [];
  var i = 0;
  var n = src.length;
  while (i < n) {
    var ch = src.charAt(i);
    if (ch === " " || ch === "\t" || ch === "\n" || ch === "\r") { i += 1; continue; }
    if (isDigit(ch) || (ch === "." && isDigit(src.charAt(i + 1)))) {
      var start = i;
      while (i < n && (isDigit(src.charAt(i)) || src.charAt(i) === ".")) { i += 1; }
      if (i < n && (src.charAt(i) === "e" || src.charAt(i) === "E")) {
        i += 1;
        if (i < n && (src.charAt(i) === "+" || src.charAt(i) === "-")) { i += 1; }
        while (i < n && isDigit(src.charAt(i))) { i += 1; }
      }
      tokens.push({ t: "num", v: parseFloat(src.slice(start, i)) });
      continue;
    }
    if (isIdentStart(ch)) {
      var s = i;
      while (i < n && isIdentPart(src.charAt(i))) { i += 1; }
      tokens.push({ t: "name", v: src.slice(s, i) });
      continue;
    }
    if (ch === "*" && src.charAt(i + 1) === "*") { tokens.push({ t: "op", v: "^" }); i += 2; continue; }
    if ("+-*/^(),".indexOf(ch) !== -1) { tokens.push({ t: "op", v: ch }); i += 1; continue; }
    throw new Error("unexpected character " + ch);
  }
  tokens.push({ t: "end", v: "" });
  return tokens;
}

function parse(src) {
  var tokens = tokenize(src);
  var pos = 0;
  function peek() { return tokens[pos]; }
  function next() { var tk = tokens[pos]; pos += 1; return tk; }
  function expect(v) { var tk = next(); if (tk.v !== v) { throw new Error("expected " + v); } }

  function parsePrimary() {
    var tk = peek();
    if (tk.t === "num") { next(); return { k: "num", value: tk.v }; }
    if (tk.t === "op" && tk.v === "(") { next(); var e = parseAdd(); expect(")"); return e; }
    if (tk.t === "op" && (tk.v === "-" || tk.v === "+")) {
      next();
      var operand = parseUnary();
      return tk.v === "-" ? { k: "unary", op: "neg", operand: operand } : operand;
    }
    if (tk.t === "name") {
      next();
      if (peek().t === "op" && peek().v === "(") {
        next();
        var args = [];
        if (!(peek().t === "op" && peek().v === ")")) {
          args.push(parseAdd());
          while (peek().t === "op" && peek().v === ",") { next(); args.push(parseAdd()); }
        }
        expect(")");
        return { k: "call", name: tk.v, args: args };
      }
      return { k: "var", name: tk.v };
    }
    throw new Error("unexpected token " + tk.v);
  }

  function parsePow() {
    var base = parsePrimary();
    if (peek().t === "op" && peek().v === "^") {
      next();
      var exponent = parseUnary();
      return { k: "bin", op: "^", left: base, right: exponent };
    }
    return base;
  }

  function parseUnary() {
    var tk = peek();
    if (tk.t === "op" && (tk.v === "-" || tk.v === "+")) {
      next();
      var operand = parseUnary();
      return tk.v === "-" ? { k: "unary", op: "neg", operand: operand } : operand;
    }
    return parsePow();
  }

  function parseMul() {
    var left = parseUnary();
    while (peek().t === "op" && (peek().v === "*" || peek().v === "/")) {
      var op = next().v;
      var right = parseUnary();
      left = { k: "bin", op: op, left: left, right: right };
    }
    return left;
  }

  function parseAdd() {
    var left = parseMul();
    while (peek().t === "op" && (peek().v === "+" || peek().v === "-")) {
      var op = next().v;
      var right = parseMul();
      left = { k: "bin", op: op, left: left, right: right };
    }
    return left;
  }

  var tree = parseAdd();
  if (peek().t !== "end") { throw new Error("trailing tokens in expression"); }
  return tree;
}

function evalNode(node, scope) {
  switch (node.k) {
    case "num": return node.value;
    case "var": {
      var value = scope[node.name];
      if (value === undefined) { throw new Error("unknown symbol " + node.name); }
      return value;
    }
    case "unary": return FUNCS1[node.op](evalNode(node.operand, scope));
    case "bin": {
      var a = evalNode(node.left, scope);
      var b = evalNode(node.right, scope);
      if (node.op === "+") { return a + b; }
      if (node.op === "-") { return a - b; }
      if (node.op === "*") { return a * b; }
      if (node.op === "/") { return a / b; }
      return Math.pow(a, b);
    }
    case "call": {
      var args = node.args.map(function (arg) { return evalNode(arg, scope); });
      if (FUNCS1[node.name] && args.length === 1) { return FUNCS1[node.name](args[0]); }
      if (FUNCS2[node.name] && args.length === 2) { return FUNCS2[node.name](args[0], args[1]); }
      throw new Error("unsupported function " + node.name);
    }
  }
  throw new Error("bad node");
}

var STATES = PAYLOAD.states;
var TIME_SYMBOL = PAYLOAD.timeSymbol || "t";
var COMPILED = {};
STATES.forEach(function (state) { COMPILED[state] = parse(PAYLOAD.laws[state]); });

function derivatives(params, values, time) {
  var scope = {};
  var key;
  for (key in params) { scope[key] = params[key]; }
  for (key in values) { scope[key] = values[key]; }
  scope[TIME_SYMBOL] = time;
  var out = {};
  STATES.forEach(function (state) { out[state] = evalNode(COMPILED[state], scope); });
  return out;
}

function integrate(params, initial, method) {
  var start = PAYLOAD.time.start, end = PAYLOAD.time.end, step = PAYLOAD.time.step;
  var count = Math.floor((end - start) / step) + 1;
  var times = [];
  var series = {};
  STATES.forEach(function (s) { series[s] = []; });
  var values = {};
  STATES.forEach(function (s) { values[s] = initial[s] === undefined ? 0 : initial[s]; });
  var time = start;
  for (var idx = 0; idx < count; idx += 1) {
    times.push(time);
    STATES.forEach(function (s) { series[s].push(values[s]); });
    if (method === "euler") {
      var d = derivatives(params, values, time);
      var nextE = {};
      STATES.forEach(function (s) { nextE[s] = values[s] + step * d[s]; });
      values = nextE;
    } else {
      var k1 = derivatives(params, values, time);
      var m1 = {}; STATES.forEach(function (s) { m1[s] = values[s] + 0.5 * step * k1[s]; });
      var k2 = derivatives(params, m1, time + 0.5 * step);
      var m2 = {}; STATES.forEach(function (s) { m2[s] = values[s] + 0.5 * step * k2[s]; });
      var k3 = derivatives(params, m2, time + 0.5 * step);
      var e1 = {}; STATES.forEach(function (s) { e1[s] = values[s] + step * k3[s]; });
      var k4 = derivatives(params, e1, time + step);
      var nextR = {};
      STATES.forEach(function (s) { nextR[s] = values[s] + (step / 6) * (k1[s] + 2 * k2[s] + 2 * k3[s] + k4[s]); });
      values = nextR;
    }
    time += step;
  }
  return { time: times, values: series };
}

var root = document.getElementById(ROOT_ID);
if (root) {
  var svg = root.querySelector(".ls-chart");
  var W = 760, H = 340, ML = 52, MR = 118, MT = 16, MB = 34;
  var params = {};
  PAYLOAD.parameters.forEach(function (p) { params[p.id] = p.value; });
  var initial = {};
  STATES.forEach(function (s) { initial[s] = PAYLOAD.initial[s] === undefined ? 0 : PAYLOAD.initial[s]; });
  var method = PAYLOAD.method || "rk4";
  var marker = -1;

  function esc(text) { return String(text).split("&").join("&amp;").split("<").join("&lt;"); }
  function fmt(value) {
    if (!isFinite(value)) { return "n/a"; }
    var a = Math.abs(value);
    if (a !== 0 && (a < 0.001 || a >= 100000)) { return value.toExponential(2); }
    return (Math.round(value * 10000) / 10000).toString();
  }

  function render() {
    var traj;
    try { traj = integrate(params, initial, method); }
    catch (err) {
      svg.innerHTML = "<text x='16' y='28' fill='#dc2626' font-family='system-ui'>" + esc(err.message) + "</text>";
      return;
    }
    var lo = Infinity, hi = -Infinity;
    STATES.forEach(function (s) {
      traj.values[s].forEach(function (v) {
        if (isFinite(v)) { if (v < lo) { lo = v; } if (v > hi) { hi = v; } }
      });
    });
    if (!isFinite(lo) || !isFinite(hi)) { lo = 0; hi = 1; }
    if (hi - lo < 1e-9) { hi = lo + 1; lo = lo - 1; }
    var pad = (hi - lo) * 0.08; lo -= pad; hi += pad;
    var t0 = traj.time[0], t1 = traj.time[traj.time.length - 1];
    if (t1 - t0 < 1e-9) { t1 = t0 + 1; }
    function px(t) { return ML + (t - t0) / (t1 - t0) * (W - ML - MR); }
    function py(v) { return MT + (1 - (v - lo) / (hi - lo)) * (H - MT - MB); }

    var parts = [];
    parts.push("<rect x='" + ML + "' y='" + MT + "' width='" + (W - ML - MR) + "' height='" + (H - MT - MB) + "' fill='none' stroke='" + PAYLOAD.axis + "' />");
    for (var g = 0; g <= 4; g += 1) {
      var yv = lo + (hi - lo) * g / 4;
      var yy = py(yv);
      parts.push("<line x1='" + ML + "' y1='" + yy + "' x2='" + (W - MR) + "' y2='" + yy + "' stroke='" + PAYLOAD.grid + "' />");
      parts.push("<text x='" + (ML - 6) + "' y='" + (yy + 4) + "' text-anchor='end' font-size='11' fill='" + PAYLOAD.muted + "' font-family='system-ui'>" + esc(fmt(yv)) + "</text>");
    }
    parts.push("<text x='" + ML + "' y='" + (H - 8) + "' font-size='11' fill='" + PAYLOAD.muted + "' font-family='system-ui'>t=" + esc(fmt(t0)) + "</text>");
    parts.push("<text x='" + (W - MR) + "' y='" + (H - 8) + "' text-anchor='end' font-size='11' fill='" + PAYLOAD.muted + "' font-family='system-ui'>t=" + esc(fmt(t1)) + "</text>");

    STATES.forEach(function (s, si) {
      var color = SERIES_COLORS[si % SERIES_COLORS.length];
      var pts = [];
      for (var i = 0; i < traj.time.length; i += 1) {
        var v = traj.values[s][i];
        if (isFinite(v)) { pts.push(px(traj.time[i]).toFixed(1) + "," + py(v).toFixed(1)); }
      }
      parts.push("<polyline points='" + pts.join(" ") + "' fill='none' stroke='" + color + "' stroke-width='2' />");
      var ly = MT + 8 + si * 18;
      parts.push("<rect x='" + (W - MR + 10) + "' y='" + (ly - 8) + "' width='10' height='10' fill='" + color + "' />");
      var lastVal = traj.values[s][traj.values[s].length - 1];
      parts.push("<text x='" + (W - MR + 26) + "' y='" + (ly + 1) + "' font-size='12' fill='" + PAYLOAD.fg + "' font-family='system-ui'>" + esc(s) + " = " + esc(fmt(lastVal)) + "</text>");
    });

    if (marker >= 0 && marker < traj.time.length) {
      var mx = px(traj.time[marker]);
      parts.push("<line x1='" + mx + "' y1='" + MT + "' x2='" + mx + "' y2='" + (H - MB) + "' stroke='" + PAYLOAD.accent + "' stroke-dasharray='4 3' />");
    }
    svg.innerHTML = parts.join("");
  }

  root.querySelectorAll(".ls-param").forEach(function (input) {
    input.addEventListener("input", function () {
      params[input.getAttribute("data-id")] = parseFloat(input.value);
      var out = root.querySelector("[data-val='" + input.getAttribute("data-id") + "']");
      if (out) { out.textContent = fmt(params[input.getAttribute("data-id")]); }
      marker = -1;
      render();
    });
  });
  root.querySelectorAll(".ls-init").forEach(function (input) {
    input.addEventListener("input", function () {
      var val = parseFloat(input.value);
      if (isFinite(val)) { initial[input.getAttribute("data-state")] = val; marker = -1; render(); }
    });
  });
  var methodSelect = root.querySelector(".ls-method");
  if (methodSelect) {
    methodSelect.addEventListener("change", function () { method = methodSelect.value; marker = -1; render(); });
  }

  var timer = null;
  var playBtn = root.querySelector(".ls-play");
  var resetBtn = root.querySelector(".ls-reset");
  function stop() { if (timer) { clearInterval(timer); timer = null; } if (playBtn) { playBtn.textContent = "▶ Play"; } }
  if (playBtn) {
    playBtn.addEventListener("click", function () {
      if (timer) { stop(); return; }
      var total = Math.floor((PAYLOAD.time.end - PAYLOAD.time.start) / PAYLOAD.time.step) + 1;
      if (marker < 0 || marker >= total - 1) { marker = 0; }
      playBtn.textContent = "⏸ Pause";
      timer = setInterval(function () {
        marker += 1;
        if (marker >= total - 1) { marker = total - 1; render(); stop(); return; }
        render();
      }, 40);
    });
  }
  if (resetBtn) {
    resetBtn.addEventListener("click", function () {
      stop();
      marker = -1;
      PAYLOAD.parameters.forEach(function (p) { params[p.id] = p.value; });
      STATES.forEach(function (s) { initial[s] = PAYLOAD.initial[s] === undefined ? 0 : PAYLOAD.initial[s]; });
      root.querySelectorAll(".ls-param").forEach(function (input) {
        input.value = params[input.getAttribute("data-id")];
        var out = root.querySelector("[data-val='" + input.getAttribute("data-id") + "']");
        if (out) { out.textContent = fmt(params[input.getAttribute("data-id")]); }
      });
      root.querySelectorAll(".ls-init").forEach(function (input) {
        input.value = initial[input.getAttribute("data-state")];
      });
      render();
    });
  }

  render();
}
"""
