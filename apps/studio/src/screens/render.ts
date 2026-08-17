import type { PlotGeometry } from "@lawsynth/world-viewer";
import type {
  BandOverlay,
  ControlField,
  ScreenActions,
  ScreenModel,
  ScreenSection,
  TimelineView,
} from "./types.js";

const SVG_NS = "http://www.w3.org/2000/svg";

function el<K extends keyof HTMLElementTagNameMap>(document: Document, tag: K, className?: string, text?: string): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className !== undefined) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function svg(document: Document, tag: string, attrs: Readonly<Record<string, string | number>>): SVGElement {
  const node = document.createElementNS(SVG_NS, tag);
  for (const [key, value] of Object.entries(attrs)) node.setAttribute(key, String(value));
  return node;
}

function controlValue(input: HTMLInputElement | HTMLSelectElement, kind: ControlField["kind"]): string {
  if (kind === "toggle" && input instanceof HTMLInputElement) return input.checked ? "true" : "false";
  return input.value;
}

function renderField(document: Document, field: ControlField, actions: ScreenActions): HTMLElement {
  const wrapper = el(document, "label", "lss-scr-field");
  const caption = field.unit ? `${field.label} (${field.unit})` : field.label;
  wrapper.append(el(document, "span", "lss-scr-field-label", caption));

  if (field.kind === "select") {
    const select = el(document, "select");
    for (const option of field.options ?? []) {
      const node = el(document, "option", undefined, option.label);
      node.value = option.value;
      if (option.value === String(field.value)) node.selected = true;
      select.append(node);
    }
    if (field.disabled === true) select.disabled = true;
    select.addEventListener("change", () => actions.onControl(field.id, controlValue(select, field.kind)));
    wrapper.append(select);
  } else {
    const input = el(document, "input");
    if (field.kind === "toggle") {
      input.type = "checkbox";
      input.checked = field.value === true;
    } else {
      input.type = field.kind === "range" ? "range" : field.kind === "number" ? "number" : "text";
      input.value = String(field.value);
      if (field.min !== undefined) input.min = String(field.min);
      if (field.max !== undefined) input.max = String(field.max);
      if (field.step !== undefined) input.step = String(field.step);
    }
    if (field.disabled === true) input.disabled = true;
    input.addEventListener("change", () => actions.onControl(field.id, controlValue(input, field.kind)));
    wrapper.append(input);
  }
  if (field.help !== undefined) wrapper.append(el(document, "span", "lss-scr-field-help", field.help));
  return wrapper;
}

function renderChart(document: Document, geometry: PlotGeometry, bands: readonly BandOverlay[]): SVGElement {
  const root = svg(document, "svg", { viewBox: `0 0 ${geometry.width} ${geometry.height}`, class: "lss-scr-chart", role: "img" });
  for (const band of bands) {
    if (band.polygon) root.append(svg(document, "path", { d: band.polygon, fill: band.color, "fill-opacity": 0.18, stroke: "none" }));
    if (band.medianPath !== undefined) root.append(svg(document, "path", { d: band.medianPath, fill: "none", stroke: band.color, "stroke-width": 1, "stroke-dasharray": "4 3" }));
  }
  for (const path of geometry.paths) {
    root.append(svg(document, "path", { d: path.d, fill: "none", stroke: path.color ?? "#b54b2a", "stroke-width": 2 }));
  }
  return root;
}

function renderTimeline(document: Document, view: TimelineView, sectionId: string, actions: ScreenActions): SVGElement {
  const height = view.height + 34;
  const root = svg(document, "svg", { viewBox: `0 0 ${view.width} ${height}`, class: "lss-scr-timeline", role: "img" });
  for (const tick of view.ticks) {
    root.append(svg(document, "line", { x1: tick.x, y1: 0, x2: tick.x, y2: view.height, stroke: "#e2ded2", "stroke-width": 1 }));
    const label = svg(document, "text", { x: tick.x + 2, y: height - 6, "font-size": 10, fill: "#59635e" });
    label.textContent = tick.label;
    root.append(label);
  }
  for (const segment of view.segments) {
    const rect = svg(document, "rect", {
      x: segment.x,
      y: segment.y + 4,
      width: segment.width,
      height: segment.height,
      rx: 4,
      fill: segment.color,
      "fill-opacity": segment.selected ? 0.9 : 0.55,
      stroke: segment.selected ? "#18201d" : "none",
      "stroke-width": segment.selected ? 2 : 0,
    });
    rect.addEventListener("click", () => actions.onSelect(sectionId, segment.regime));
    rect.style.cursor = "pointer";
    root.append(rect);
    const caption = `${segment.label}${segment.confidence === undefined ? "" : ` · ${Math.round(segment.confidence * 100)}%`}`;
    const text = svg(document, "text", { x: segment.x + 6, y: segment.y + segment.height / 2 + 8, "font-size": 11, fill: "#fffdf7" });
    text.textContent = caption;
    root.append(text);
  }
  for (const boundary of view.boundaries) {
    root.append(svg(document, "line", { x1: boundary.x, y1: 0, x2: boundary.x, y2: view.height, stroke: "#18201d", "stroke-width": 1, "stroke-dasharray": "2 2" }));
  }
  return root;
}

function renderSection(document: Document, section: ScreenSection, actions: ScreenActions): HTMLElement {
  const container = el(document, "section", "lss-scr-section");
  const title = "title" in section ? section.title : undefined;
  if (title !== undefined) container.append(el(document, "h2", "lss-scr-heading", title));

  switch (section.kind) {
    case "notices":
      for (const notice of section.notices) container.append(el(document, "p", `lss-scr-notice lss-tone-${notice.tone}`, notice.message));
      break;
    case "metrics": {
      const grid = el(document, "div", "lss-scr-metrics");
      for (const metric of section.metrics) {
        const card = el(document, "article", `lss-scr-metric${metric.tone === undefined ? "" : ` lss-tone-${metric.tone}`}`);
        card.append(el(document, "span", "lss-scr-metric-label", metric.label), el(document, "strong", "lss-scr-metric-value", metric.value));
        grid.append(card);
      }
      container.append(grid);
      break;
    }
    case "controls": {
      const form = el(document, "div", "lss-scr-controls");
      for (const field of section.fields) form.append(renderField(document, field, actions));
      container.append(form);
      break;
    }
    case "actions": {
      const bar = el(document, "div", "lss-scr-actions");
      for (const button of section.buttons) {
        const node = el(document, "button", `lss-scr-btn${button.tone === undefined ? "" : ` lss-tone-${button.tone}`}`, button.label);
        node.type = "button";
        if (button.disabled === true) node.disabled = true;
        node.addEventListener("click", () => actions.onAction(button.id));
        bar.append(node);
      }
      container.append(bar);
      break;
    }
    case "table": {
      const table = el(document, "table", "lss-scr-table");
      const head = el(document, "tr");
      for (const column of section.columns) head.append(el(document, "th", column.align === "end" ? "lss-end" : undefined, column.label));
      const thead = el(document, "thead");
      thead.append(head);
      table.append(thead);
      const body = el(document, "tbody");
      for (const row of section.rows) {
        const tr = el(document, "tr", `${row.selected === true ? "lss-selected " : ""}${row.emphasis === true ? "lss-emphasis" : ""}`.trim() || undefined);
        row.cells.forEach((cell, index) => tr.append(el(document, "td", section.columns[index]?.align === "end" ? "lss-end" : undefined, cell)));
        tr.addEventListener("click", () => actions.onSelect(section.id, row.id));
        (tr as HTMLElement).style.cursor = "pointer";
        body.append(tr);
      }
      table.append(body);
      if (section.rows.length === 0 && section.empty !== undefined) container.append(el(document, "p", "lss-scr-empty", section.empty));
      else container.append(table);
      break;
    }
    case "chart":
      container.append(renderChart(document, section.geometry, section.bands ?? []));
      break;
    case "timeline":
      container.append(renderTimeline(document, section.timeline, section.id, actions));
      break;
    case "equations": {
      const list = el(document, "div", "lss-scr-equations");
      for (const block of section.equations) {
        const article = el(document, "article", `lss-scr-equation${block.selected ? " lss-selected" : ""}${block.enabled ? "" : " lss-disabled"}`);
        const header = el(document, "button", "lss-scr-equation-head", block.heading);
        header.type = "button";
        header.addEventListener("click", () => actions.onSelect(section.id, block.id));
        article.append(header, el(document, "code", "lss-scr-equation-text", block.text));
        if (block.selected && block.terms.length > 0) {
          const terms = el(document, "ul", "lss-scr-terms");
          for (const term of block.terms) {
            const item = el(document, "li");
            item.append(el(document, "span", "lss-scr-term-sign", term.sign), el(document, "code", undefined, term.text));
            if (term.symbols.length > 0) item.append(el(document, "span", "lss-scr-term-symbols", term.symbols.join(", ")));
            terms.append(item);
          }
          article.append(terms);
        }
        list.append(article);
      }
      container.append(list);
      break;
    }
  }
  return container;
}

/** Turns a screen render description into an interactive DOM subtree. */
export function renderScreenModel(document: Document, model: ScreenModel, actions: ScreenActions): HTMLElement {
  const root = el(document, "div", "lss-scr");
  const header = el(document, "div", "lss-scr-header");
  header.append(el(document, "h1", undefined, model.title));
  if (model.subtitle !== undefined) header.append(el(document, "p", "lss-scr-subtitle", model.subtitle));
  root.append(header);
  for (const section of model.sections) root.append(renderSection(document, section, actions));
  return root;
}
