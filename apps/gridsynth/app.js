const MODEL_VERSION = "gridsynth-linear-temperature-v1";
const HEADER_ALIASES = {
  timestamp: ["timestamp", "time", "date_time", "datetime", "tarih_saat", "zaman"],
  load: ["load_mw", "load", "demand_mw", "demand", "power_mw", "yuk_mw", "yuk"],
  voltage: ["voltage_kv", "voltage", "grid_voltage", "gerilim_kv", "gerilim"],
  temperature: ["temperature_c", "temperature", "temp_c", "temp", "weather_temp", "sicaklik_c", "sicaklik"],
  outage: ["outage_minutes", "outage", "interruption_minutes", "kesinti_dakika", "kesinti"],
};

const state = {
  rows: [],
  sourceName: "",
  model: null,
  anomalies: [],
  baselineForecast: [],
  forecast: [],
  contract: null,
  rawDataset: null,
  workflowStatus: "awaiting-file",
  workflowMessage: "GridSynth örnek veri yüklemez. Analizi başlatmak için CSV seç.",
  scenarioChanged: false,
  reportExported: false,
};

const elements = {
  csvInput: document.querySelector("#csvInput"),
  sourceBadge: document.querySelector("#sourceBadge"),
  sourceNote: document.querySelector("#sourceNote"),
  analysisStatus: document.querySelector("#analysisStatus"),
  statusLabel: document.querySelector("#statusLabel"),
  statusMessage: document.querySelector("#statusMessage"),
  routeSteps: [...document.querySelectorAll("[data-route-step]")],
  mappingLink: document.querySelector("[data-mapping-link]"),
  routeExportButton: document.querySelector("#routeExportButton"),
  mappingPanel: document.querySelector("#mappingPanel"),
  mappingSummary: document.querySelector("#mappingSummary"),
  mappingForm: document.querySelector("#mappingForm"),
  mappingError: document.querySelector("#mappingError"),
  timestampColumn: document.querySelector("#timestampColumn"),
  loadColumn: document.querySelector("#loadColumn"),
  voltageColumn: document.querySelector("#voltageColumn"),
  temperatureColumn: document.querySelector("#temperatureColumn"),
  outageColumn: document.querySelector("#outageColumn"),
  loadUnit: document.querySelector("#loadUnit"),
  voltageUnit: document.querySelector("#voltageUnit"),
  temperatureUnit: document.querySelector("#temperatureUnit"),
  currentLoad: document.querySelector("#currentLoad"),
  peakLoad: document.querySelector("#peakLoad"),
  averageVoltage: document.querySelector("#averageVoltage"),
  anomalyCount: document.querySelector("#anomalyCount"),
  dataRange: document.querySelector("#dataRange"),
  usedColumns: document.querySelector("#usedColumns"),
  rowQuality: document.querySelector("#rowQuality"),
  intervalQuality: document.querySelector("#intervalQuality"),
  unitConversions: document.querySelector("#unitConversions"),
  modelVersion: document.querySelector("#modelVersion"),
  skippedRowsDetails: document.querySelector("#skippedRowsDetails"),
  skippedRowsSummary: document.querySelector("#skippedRowsSummary"),
  skippedRowsList: document.querySelector("#skippedRowsList"),
  chart: document.querySelector("#loadChart"),
  chartEmpty: document.querySelector("#chartEmpty"),
  equation: document.querySelector("#equation"),
  modelQuality: document.querySelector("#modelQuality"),
  demandShift: document.querySelector("#demandShift"),
  temperatureShift: document.querySelector("#temperatureShift"),
  generationShare: document.querySelector("#generationShare"),
  demandOutput: document.querySelector("#demandOutput"),
  temperatureOutput: document.querySelector("#temperatureOutput"),
  generationOutput: document.querySelector("#generationOutput"),
  scenarioForm: document.querySelector("#scenarioForm"),
  scenarioReset: document.querySelector("#scenarioForm button[type='reset']"),
  scenarioPeak: document.querySelector("#scenarioPeak"),
  riskLabel: document.querySelector("#riskLabel"),
  riskFill: document.querySelector("#riskFill"),
  scenarioReason: document.querySelector("#scenarioReason"),
  findingList: document.querySelector("#findingList"),
  exportButton: document.querySelector("#exportButton"),
};

const STATUS_LABELS = {
  "awaiting-file": "CSV bekleniyor",
  reading: "CSV okunuyor",
  "mapping-required": "Eşleme gerekiyor",
  "invalid-file": "CSV doğrulanamadı",
  "file-read-error": "Dosya okunamadı",
  "analysis-error": "Analiz uygulanamadı",
  analyzed: "Analiz hazır",
};

function retainedResultText() {
  return state.model
    ? `Ekranda ${state.sourceName} sonuçları korunuyor.`
    : "Ekranda analiz sonucu yok.";
}

function hasAnalysis() {
  return Boolean(state.model && state.contract && state.rows.length);
}

function routeState(step) {
  const status = state.workflowStatus;
  if (status === "awaiting-file") {
    return step === "source" ? "current" : "pending";
  }
  if (status === "reading" || status === "invalid-file" || status === "file-read-error") {
    return step === "source" ? "current" : "pending";
  }
  if (status === "mapping-required" || status === "analysis-error") {
    if (step === "source") return "complete";
    return step === "mapping" ? "current" : "pending";
  }
  if (step === "source" || step === "mapping" || step === "contract") return "complete";
  if (step === "scenario") return state.scenarioChanged ? "complete" : "current";
  return state.reportExported ? "complete" : state.scenarioChanged ? "current" : "pending";
}

function renderWorkflowStatus() {
  elements.analysisStatus.dataset.state = state.workflowStatus;
  elements.analysisStatus.setAttribute("aria-busy", String(state.workflowStatus === "reading"));
  elements.statusLabel.textContent = STATUS_LABELS[state.workflowStatus];
  elements.statusMessage.textContent = state.workflowMessage;
  if (hasAnalysis()) {
    elements.sourceBadge.textContent = "YÜKLENEN CSV";
    elements.sourceBadge.dataset.source = "uploaded";
  } else if (state.workflowStatus === "reading") {
    elements.sourceBadge.textContent = "CSV OKUNUYOR";
    elements.sourceBadge.dataset.source = "pending";
  } else if (state.workflowStatus === "mapping-required" || state.workflowStatus === "analysis-error") {
    elements.sourceBadge.textContent = "CSV EŞLEMEDE";
    elements.sourceBadge.dataset.source = "pending";
  } else {
    elements.sourceBadge.textContent = "CSV GEREKLİ";
    elements.sourceBadge.dataset.source = "waiting";
  }
  elements.routeSteps.forEach((step) => {
    const nextState = routeState(step.dataset.routeStep);
    step.dataset.state = nextState;
    if (nextState === "current") {
      step.setAttribute("aria-current", "step");
    } else {
      step.removeAttribute("aria-current");
    }
  });

  const mappingAvailable = state.workflowStatus === "mapping-required" || state.workflowStatus === "analysis-error";
  elements.mappingLink.textContent = mappingAvailable ? "Eşlemeye git" : "CSV okununca açılır";
  elements.mappingLink.setAttribute("aria-disabled", String(!mappingAvailable));
  elements.mappingLink.tabIndex = mappingAvailable ? 0 : -1;

  const analysisAvailable = hasAnalysis();
  elements.exportButton.disabled = !analysisAvailable;
  elements.routeExportButton.disabled = !analysisAvailable;
  elements.demandShift.disabled = !analysisAvailable;
  elements.temperatureShift.disabled = !analysisAvailable;
  elements.generationShare.disabled = !analysisAvailable;
  elements.scenarioReset.disabled = !analysisAvailable;
}

function setWorkflowStatus(status, message) {
  state.workflowStatus = status;
  state.workflowMessage = message;
  renderWorkflowStatus();
}

function normalizeHeader(value) {
  return value
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "");
}

function parseCsvTable(text) {
  if (!text.trim()) throw new Error("CSV dosyası boş.");

  const parsed = [];
  let row = [];
  let field = "";
  let quoted = false;

  for (let index = 0; index < text.length; index += 1) {
    const character = text[index];
    const next = text[index + 1];
    if (character === '"') {
      if (quoted && next === '"') {
        field += '"';
        index += 1;
      } else {
        quoted = !quoted;
      }
    } else if (character === "," && !quoted) {
      row.push(field.trim());
      field = "";
    } else if ((character === "\n" || character === "\r") && !quoted) {
      if (character === "\r" && next === "\n") index += 1;
      row.push(field.trim());
      parsed.push(row);
      row = [];
      field = "";
    } else {
      field += character;
    }
  }

  if (quoted) throw new Error("CSV içinde kapanmamış tırnak bulundu.");
  if (field.length || row.length) {
    row.push(field.trim());
    parsed.push(row);
  }
  while (parsed.length && parsed.at(-1).every((value) => value === "")) {
    parsed.pop();
  }
  if (parsed.length < 3) throw new Error("CSV en az iki veri satırı içermeli.");

  const headers = parsed[0].map((value) => value.trim());
  if (headers.some((header) => !header)) {
    throw new Error("CSV başlık satırında boş sütun adı bulunuyor.");
  }
  const normalized = headers.map(normalizeHeader);
  if (new Set(normalized).size !== normalized.length) {
    throw new Error("CSV başlıkları benzersiz olmalı.");
  }

  return {
    headers,
    rows: parsed.slice(1).map((values, index) => ({ line: index + 2, values })),
  };
}

function guessColumn(headers, field) {
  const aliases = HEADER_ALIASES[field];
  return headers.find((header) => aliases.includes(normalizeHeader(header))) ?? "";
}

function populateColumnSelect(select, headers, field, optional = false) {
  select.replaceChildren();
  select.append(new Option(optional ? "Kullanılmıyor" : "Sütun seç", ""));
  headers.forEach((header) => select.append(new Option(header, header)));
  select.value = guessColumn(headers, field);
}

function showColumnMapping(table, sourceName) {
  state.rawDataset = { ...table, sourceName };
  populateColumnSelect(elements.timestampColumn, table.headers, "timestamp");
  populateColumnSelect(elements.loadColumn, table.headers, "load");
  populateColumnSelect(elements.voltageColumn, table.headers, "voltage");
  populateColumnSelect(elements.temperatureColumn, table.headers, "temperature");
  populateColumnSelect(elements.outageColumn, table.headers, "outage", true);
  elements.loadUnit.value = "mw";
  elements.voltageUnit.value = "kv";
  elements.temperatureUnit.value = "c";
  elements.mappingSummary.textContent = `${sourceName}: ${table.headers.length} sütun ve ${table.rows.length} veri satırı bulundu. Zorunlu alanları ve kaynak birimlerini seç.`;
  elements.mappingError.textContent = "";
  elements.mappingPanel.hidden = false;
  elements.sourceNote.textContent = `${sourceName} okundu. Ekrandaki sonuçlar eşleme uygulanana kadar önceki kaynağa aittir.`;
  setWorkflowStatus(
    "mapping-required",
    `${sourceName} için sütunları ve birimleri doğrula. ${retainedResultText()}`,
  );
  elements.mappingPanel.scrollIntoView({ behavior: "smooth", block: "start" });
}

function selectedMapping() {
  return {
    timestamp: elements.timestampColumn.value,
    load: elements.loadColumn.value,
    voltage: elements.voltageColumn.value,
    temperature: elements.temperatureColumn.value,
    outage: elements.outageColumn.value,
  };
}

function selectedUnits() {
  return {
    load: elements.loadUnit.value,
    voltage: elements.voltageUnit.value,
    temperature: elements.temperatureUnit.value,
  };
}

function validateMapping(mapping) {
  const required = [mapping.timestamp, mapping.load, mapping.voltage, mapping.temperature];
  if (required.some((column) => !column)) {
    return "Zaman, yük, gerilim ve sıcaklık sütunlarını seç.";
  }
  const selected = [...required, mapping.outage].filter(Boolean);
  if (new Set(selected).size !== selected.length) {
    return "Her veri alanını farklı bir CSV sütununa bağla.";
  }
  return "";
}

function convertLoad(value, unit) {
  if (unit === "kw") return value / 1_000;
  if (unit === "w") return value / 1_000_000;
  return value;
}

function convertVoltage(value, unit) {
  return unit === "v" ? value / 1_000 : value;
}

function convertTemperature(value, unit) {
  return unit === "f" ? ((value - 32) * 5) / 9 : value;
}

function unitConversionNotes(units) {
  const notes = [];
  if (units.load === "kw") notes.push("kW → MW");
  if (units.load === "w") notes.push("W → MW");
  if (units.voltage === "v") notes.push("V → kV");
  if (units.temperature === "f") notes.push("°F → °C");
  return notes.length ? notes : ["Dönüşüm yok; MW, kV ve °C korundu"];
}

function measureIntervals(rows) {
  const differences = rows.slice(1).map((row, index) => ({
    from: rows[index].timestamp,
    to: row.timestamp,
    milliseconds: row.timestamp.getTime() - rows[index].timestamp.getTime(),
  }));
  const positive = differences.filter((item) => item.milliseconds > 0);
  if (!positive.length) {
    return { expectedMilliseconds: null, irregular: differences };
  }

  const counts = new Map();
  positive.forEach((item) => {
    counts.set(item.milliseconds, (counts.get(item.milliseconds) ?? 0) + 1);
  });
  const expectedMilliseconds = [...counts.entries()].sort((a, b) => b[1] - a[1])[0][0];
  const irregular = differences.filter(
    (item) => Math.abs(item.milliseconds - expectedMilliseconds) > 1_000,
  );
  return { expectedMilliseconds, irregular };
}

function parseMappedDataset(table, mapping, units) {
  const required = [mapping.timestamp, mapping.load, mapping.voltage, mapping.temperature];
  if (required.some((column) => !column)) {
    throw new Error("Zaman, yük, gerilim ve sıcaklık sütunlarını seç.");
  }
  const selected = [...required, mapping.outage].filter(Boolean);
  if (new Set(selected).size !== selected.length) {
    throw new Error("Her veri alanı farklı bir CSV sütununa bağlanmalı.");
  }

  const columnIndex = Object.fromEntries(
    table.headers.map((header, index) => [header, index]),
  );
  const skippedRows = [];
  const rows = [];

  table.rows.forEach(({ line, values }) => {
    const source = (column) => values[columnIndex[column]]?.trim() ?? "";
    const requiredValues = {
      timestamp: source(mapping.timestamp),
      load: source(mapping.load),
      voltage: source(mapping.voltage),
      temperature: source(mapping.temperature),
    };
    const missing = Object.entries(requiredValues)
      .filter(([, value]) => value === "")
      .map(([field]) => field);
    if (missing.length) {
      skippedRows.push({ line, reason: `Eksik zorunlu alan: ${missing.join(", ")}` });
      return;
    }

    const timestamp = new Date(requiredValues.timestamp);
    const loadValue = Number(requiredValues.load);
    const voltageValue = Number(requiredValues.voltage);
    const temperatureValue = Number(requiredValues.temperature);
    const outageRaw = mapping.outage ? source(mapping.outage) : "";
    const outageMinutes = outageRaw === "" ? 0 : Number(outageRaw);
    if (
      !Number.isFinite(timestamp.getTime()) ||
      ![loadValue, voltageValue, temperatureValue, outageMinutes].every(Number.isFinite)
    ) {
      skippedRows.push({ line, reason: "Tarih veya sayısal değer okunamadı" });
      return;
    }

    rows.push({
      timestamp,
      load: convertLoad(loadValue, units.load),
      voltage: convertVoltage(voltageValue, units.voltage),
      temperature: convertTemperature(temperatureValue, units.temperature),
      outageMinutes,
      sourceLine: line,
    });
  });

  rows.sort((a, b) => a.timestamp - b.timestamp);
  if (rows.length < 2) {
    throw new Error("Eşleme sonrasında model için en az iki geçerli satır gerekli.");
  }

  const intervals = measureIntervals(rows);
  return {
    rows,
    contract: {
      modelVersion: MODEL_VERSION,
      mapping,
      sourceUnits: units,
      conversions: unitConversionNotes(units),
      inputRows: table.rows.length,
      acceptedRows: rows.length,
      skippedRows,
      dateRange: {
        start: rows[0].timestamp.toISOString(),
        end: rows.at(-1).timestamp.toISOString(),
      },
      expectedIntervalMinutes:
        intervals.expectedMilliseconds === null
          ? null
          : intervals.expectedMilliseconds / 60_000,
      irregularIntervals: intervals.irregular.map((item) => ({
        from: item.from.toISOString(),
        to: item.to.toISOString(),
        minutes: item.milliseconds / 60_000,
      })),
    },
  };
}

function mean(values) {
  return values.reduce((sum, value) => sum + value, 0) / values.length;
}

function fitLinearModel(rows) {
  const temperatures = rows.map((row) => row.temperature);
  const loads = rows.map((row) => row.load);
  const meanTemp = mean(temperatures);
  const meanLoad = mean(loads);
  const numerator = rows.reduce((sum, row) => sum + (row.temperature - meanTemp) * (row.load - meanLoad), 0);
  const denominator = rows.reduce((sum, row) => sum + (row.temperature - meanTemp) ** 2, 0);
  const temperatureCoefficient = denominator === 0 ? 0 : numerator / denominator;
  const intercept = meanLoad - temperatureCoefficient * meanTemp;
  const predictions = rows.map((row) => intercept + temperatureCoefficient * row.temperature);
  const residuals = rows.map((row, index) => row.load - predictions[index]);
  const residualStd = Math.sqrt(mean(residuals.map((value) => value ** 2))) || 1;
  const totalVariance = rows.reduce((sum, row) => sum + (row.load - meanLoad) ** 2, 0);
  const residualVariance = residuals.reduce((sum, value) => sum + value ** 2, 0);
  const rSquared = totalVariance === 0 ? 1 : Math.max(0, 1 - residualVariance / totalVariance);

  return { intercept, temperatureCoefficient, predictions, residuals, residualStd, rSquared };
}

function deriveAnomalies(rows, model) {
  return rows
    .map((row, index) => ({ row, index, residual: model.residuals[index], score: Math.abs(model.residuals[index]) / model.residualStd }))
    .filter((item) => item.score >= 1.75 || item.row.outageMinutes > 0)
    .sort((a, b) => b.score - a.score);
}

function scenarioValues() {
  return {
    demandShift: Number(elements.demandShift.value),
    temperatureShift: Number(elements.temperatureShift.value),
    generationShare: Number(elements.generationShare.value),
  };
}

function buildForecast(rows, model, scenario, contract = state.contract) {
  const recent = rows.slice(-6);
  const contractInterval = contract?.expectedIntervalMinutes;
  const interval = Number.isFinite(contractInterval) && contractInterval > 0
    ? contractInterval * 60_000
    : rows.length > 1
      ? rows.at(-1).timestamp.getTime() - rows.at(-2).timestamp.getTime()
      : 3_600_000;
  return recent.map((row, index) => {
    const temperature = row.temperature + scenario.temperatureShift;
    const baseline = model.intercept + model.temperatureCoefficient * temperature;
    const demandFactor = 1 + scenario.demandShift / 100;
    const generationFactor = 1 - scenario.generationShare / 100;
    return {
      timestamp: new Date(rows.at(-1).timestamp.getTime() + interval * (index + 1)),
      load: Math.max(0, baseline * demandFactor * generationFactor),
      temperature,
    };
  });
}

function formatNumber(value, digits = 1) {
  return new Intl.NumberFormat("tr-TR", { minimumFractionDigits: digits, maximumFractionDigits: digits }).format(value);
}

function formatTime(date) {
  return new Intl.DateTimeFormat("tr-TR", { day: "2-digit", month: "short", hour: "2-digit", minute: "2-digit" }).format(date);
}

function formatDateTime(date) {
  return new Intl.DateTimeFormat("tr-TR", {
    year: "numeric",
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}

function formatInterval(minutes) {
  if (minutes === null || !Number.isFinite(minutes)) return "Ölçülemedi";
  if (minutes >= 60 && minutes % 60 === 0) return `${minutes / 60} saat`;
  return `${formatNumber(minutes, minutes % 1 === 0 ? 0 : 1)} dakika`;
}

function svgElement(name, attributes = {}, text = "") {
  const node = document.createElementNS("http://www.w3.org/2000/svg", name);
  Object.entries(attributes).forEach(([key, value]) => node.setAttribute(key, value));
  if (text) node.textContent = text;
  return node;
}

function renderChart() {
  const rows = state.rows;
  const forecast = state.forecast;
  elements.chart.replaceChildren(
    svgElement("title", { id: "chartTitle" }, "Saatlik yük ve senaryo tahmini"),
    svgElement("desc", { id: "chartDescription" }, "Yüklenen veriden hesaplanan gözlem çizgisi, temel tahmin, müdahale senaryosu ve model artığı yüksek noktalar."),
  );
  elements.chartEmpty.hidden = rows.length > 0;
  if (!rows.length) return;

  const width = 1000;
  const height = 420;
  const padding = { left: 58, right: 24, top: 22, bottom: 42 };
  const allLoads = [
    ...rows.map((row) => row.load),
    ...state.baselineForecast.map((row) => row.load),
    ...forecast.map((row) => row.load),
  ];
  const minLoad = Math.min(...allLoads) * 0.9;
  const maxLoad = Math.max(...allLoads) * 1.08;
  const count = rows.length + forecast.length;
  const x = (index) => padding.left + (index / Math.max(1, count - 1)) * (width - padding.left - padding.right);
  const y = (value) => padding.top + ((maxLoad - value) / Math.max(1, maxLoad - minLoad)) * (height - padding.top - padding.bottom);

  for (let line = 0; line <= 4; line += 1) {
    const value = minLoad + ((maxLoad - minLoad) * (4 - line)) / 4;
    const lineY = padding.top + (line / 4) * (height - padding.top - padding.bottom);
    elements.chart.append(
      svgElement("line", { class: "chart-grid", x1: padding.left, x2: width - padding.right, y1: lineY, y2: lineY }),
      svgElement("text", { class: "chart-label", x: 0, y: lineY + 4 }, `${Math.round(value)} MW`),
    );
  }

  const observedPath = rows.map((row, index) => `${index === 0 ? "M" : "L"}${x(index).toFixed(2)},${y(row.load).toFixed(2)}`).join(" ");
  elements.chart.append(svgElement("path", { class: "observed-line", d: observedPath }));

  const forecastPoints = [{ load: rows.at(-1).load }, ...forecast];
  const baselinePoints = [{ load: rows.at(-1).load }, ...state.baselineForecast];
  const baselinePath = baselinePoints.map((row, index) => `${index === 0 ? "M" : "L"}${x(rows.length - 1 + index).toFixed(2)},${y(row.load).toFixed(2)}`).join(" ");
  const forecastPath = forecastPoints.map((row, index) => `${index === 0 ? "M" : "L"}${x(rows.length - 1 + index).toFixed(2)},${y(row.load).toFixed(2)}`).join(" ");
  elements.chart.append(svgElement("path", { class: "baseline-line", d: baselinePath }));
  elements.chart.append(svgElement("path", { class: "forecast-line", d: forecastPath }));

  state.anomalies.forEach((item) => {
    elements.chart.append(svgElement("circle", { class: "anomaly-dot", cx: x(item.index), cy: y(item.row.load), r: 7 }));
  });

  const labels = [0, Math.floor((rows.length - 1) / 2), rows.length - 1, count - 1];
  labels.forEach((index) => {
    const date = index < rows.length ? rows[index].timestamp : forecast[index - rows.length].timestamp;
    elements.chart.append(svgElement("text", { class: "chart-label", x: x(index), y: height - 8, "text-anchor": index === 0 ? "start" : index === count - 1 ? "end" : "middle" }, formatTime(date)));
  });
}

function renderFindings() {
  elements.findingList.replaceChildren();
  if (!state.anomalies.length) {
    const empty = document.createElement("p");
    empty.className = "finding-empty";
    empty.textContent = "Seçili veri setinde eşik üstü model artığı bulunmadı.";
    elements.findingList.append(empty);
    return;
  }

  state.anomalies.slice(0, 6).forEach((item) => {
    const article = document.createElement("article");
    article.className = "finding";
    const direction = item.residual > 0 ? "modelin üstünde" : "modelin altında";
    const outage = item.row.outageMinutes > 0 ? `, ${item.row.outageMinutes} dakika kesinti kaydı` : "";
    article.innerHTML = `
      <header><span>${formatTime(item.row.timestamp)}</span><span>${formatNumber(item.score, 2)}σ</span></header>
      <h3>${formatNumber(item.row.load)} MW yük</h3>
      <p>Ölçüm beklenen değerin ${formatNumber(Math.abs(item.residual))} MW ${direction}${outage}. Operatör incelemesi önerilir.</p>
    `;
    elements.findingList.append(article);
  });
}

function renderScenario() {
  const scenario = scenarioValues();
  elements.demandOutput.textContent = `${scenario.demandShift >= 0 ? "+" : ""}${scenario.demandShift}%`;
  elements.temperatureOutput.textContent = `${scenario.temperatureShift >= 0 ? "+" : ""}${scenario.temperatureShift}°C`;
  elements.generationOutput.textContent = `${scenario.generationShare}%`;

  state.forecast = buildForecast(state.rows, state.model, scenario);
  const peak = Math.max(...state.forecast.map((row) => row.load));
  const observedPeak = Math.max(...state.rows.map((row) => row.load));
  const ratio = peak / Math.max(1, observedPeak);
  const riskPercent = Math.min(100, Math.round(ratio * 78));
  const risk = ratio >= 1.12 ? "Yüksek yük riski" : ratio >= 0.94 ? "Yakın izleme" : "Planlanan aralıkta";

  elements.scenarioPeak.textContent = formatNumber(peak);
  elements.riskLabel.textContent = risk;
  elements.riskFill.style.width = `${riskPercent}%`;
  elements.scenarioReason.textContent = `Tahmin, talebi ${scenario.demandShift}% değiştiriyor; sıcaklığı ${scenario.temperatureShift}°C kaydırıyor ve dağıtık üretimin yükü ${scenario.generationShare}% azaltacağını varsayıyor. Gözlenen tepe ${formatNumber(observedPeak)} MW.`;
  renderChart();
}

function renderDataContract() {
  const contract = state.contract;
  if (!contract) return;
  const mapping = contract.mapping;
  const usedColumns = [
    `zaman: ${mapping.timestamp}`,
    `yük: ${mapping.load}`,
    `gerilim: ${mapping.voltage}`,
    `sıcaklık: ${mapping.temperature}`,
    ...(mapping.outage ? [`kesinti: ${mapping.outage}`] : []),
  ];
  const irregularCount = contract.irregularIntervals.length;

  elements.modelVersion.textContent = contract.modelVersion;
  elements.dataRange.textContent = `${formatDateTime(new Date(contract.dateRange.start))} — ${formatDateTime(new Date(contract.dateRange.end))}`;
  elements.usedColumns.textContent = usedColumns.join("; ");
  elements.rowQuality.textContent = `${contract.acceptedRows}/${contract.inputRows} satır kullanıldı; ${contract.skippedRows.length} satır atlandı`;
  elements.intervalQuality.textContent = `${formatInterval(contract.expectedIntervalMinutes)} beklenen aralık; ${irregularCount} düzensiz geçiş`;
  elements.unitConversions.textContent = contract.conversions.join("; ");

  elements.skippedRowsDetails.hidden = contract.skippedRows.length === 0;
  elements.skippedRowsSummary.textContent = `Atlanan satırlar (${contract.skippedRows.length})`;
  elements.skippedRowsList.replaceChildren();
  contract.skippedRows.forEach((item) => {
    const entry = document.createElement("li");
    entry.textContent = `${item.line}. satır: ${item.reason}`;
    elements.skippedRowsList.append(entry);
  });
}

function render() {
  const loads = state.rows.map((row) => row.load);
  elements.currentLoad.textContent = formatNumber(loads.at(-1));
  elements.peakLoad.textContent = formatNumber(Math.max(...loads));
  elements.averageVoltage.textContent = formatNumber(mean(state.rows.map((row) => row.voltage)), 2);
  elements.anomalyCount.textContent = String(state.anomalies.length);
  elements.equation.textContent = `yük = ${formatNumber(state.model.intercept, 2)} + ${formatNumber(state.model.temperatureCoefficient, 2)} × sıcaklık`;
  elements.modelQuality.textContent = `R² ${formatNumber(state.model.rSquared, 2)}. Eşik: mutlak artık ≥ 1,75σ.`;
  renderDataContract();
  renderFindings();
  renderScenario();
}

function loadParsedDataset(table, sourceName, mapping, units) {
  const parsed = parseMappedDataset(table, mapping, units);
  const model = fitLinearModel(parsed.rows);
  const anomalies = deriveAnomalies(parsed.rows, model);
  const baselineForecast = buildForecast(parsed.rows, model, {
    demandShift: 0,
    temperatureShift: 0,
    generationShare: 0,
  }, parsed.contract);

  state.rows = parsed.rows;
  state.sourceName = sourceName;
  state.contract = parsed.contract;
  state.model = model;
  state.anomalies = anomalies;
  state.baselineForecast = baselineForecast;
  state.rawDataset = null;
  state.scenarioChanged = false;
  state.reportExported = false;
  elements.demandShift.value = "0";
  elements.temperatureShift.value = "0";
  elements.generationShare.value = "0";
  elements.sourceNote.textContent = `${sourceName} yüklendi. ${state.rows.length} satır analiz edildi; ${parsed.contract.skippedRows.length} satır atlandı.`;
  elements.mappingPanel.hidden = true;
  render();
  setWorkflowStatus(
    "analyzed",
    `${sourceName} analiz edildi. Veri sözleşmesini incele veya senaryoyu değiştir.`,
  );
}

elements.csvInput.addEventListener("change", async (event) => {
  const file = event.target.files?.[0];
  if (!file) return;
  state.rawDataset = null;
  elements.mappingPanel.hidden = true;
  elements.sourceNote.textContent = `${file.name} okunuyor. ${retainedResultText()}`;
  setWorkflowStatus("reading", `${file.name} bu sekmede okunuyor. ${retainedResultText()}`);
  try {
    let csv;
    try {
      csv = await file.text();
    } catch {
      throw new DOMException("Dosya okuma erişimi başarısız oldu.", "FileReadError");
    }

    let table;
    try {
      table = parseCsvTable(csv);
    } catch (error) {
      const reason = error instanceof Error ? error.message : "CSV içeriği geçersiz.";
      elements.sourceNote.textContent = `CSV doğrulanamadı: ${reason} ${retainedResultText()}`;
      setWorkflowStatus(
        "invalid-file",
        `${reason} Dosyayı UTF-8 CSV olarak kaydedip yeniden seç. ${retainedResultText()}`,
      );
      return;
    }
    showColumnMapping(table, file.name);
  } catch (error) {
    elements.sourceNote.textContent = `Dosya okunamadı. ${retainedResultText()}`;
    setWorkflowStatus(
      "file-read-error",
      `Tarayıcı ${file.name} dosyasını okuyamadı. Dosyayı yerel diske kopyalayıp yeniden seç. ${retainedResultText()}`,
    );
  } finally {
    event.target.value = "";
  }
});

elements.mappingForm.addEventListener("submit", (event) => {
  event.preventDefault();
  if (!state.rawDataset) return;
  elements.mappingError.textContent = "";
  const mapping = selectedMapping();
  const mappingError = validateMapping(mapping);
  if (mappingError) {
    elements.mappingError.textContent = mappingError;
    setWorkflowStatus(
      "mapping-required",
      `${mappingError} ${retainedResultText()}`,
    );
    return;
  }
  try {
    loadParsedDataset(
      state.rawDataset,
      state.rawDataset.sourceName,
      mapping,
      selectedUnits(),
    );
  } catch (error) {
    const reason = error instanceof Error ? error.message : "Sütun eşlemesi uygulanamadı.";
    elements.mappingError.textContent = reason;
    setWorkflowStatus(
      "analysis-error",
      `${reason} Sütunları, birimleri ve en az iki geçerli satırı kontrol et. ${retainedResultText()}`,
    );
  }
});

[elements.demandShift, elements.temperatureShift, elements.generationShare].forEach((input) => {
  input.addEventListener("input", () => {
    state.scenarioChanged = true;
    state.reportExported = false;
    renderScenario();
    renderWorkflowStatus();
  });
});

elements.scenarioForm.addEventListener("reset", () => {
  state.scenarioChanged = false;
  state.reportExported = false;
  requestAnimationFrame(() => {
    renderScenario();
    renderWorkflowStatus();
  });
});

function exportReport() {
  if (!hasAnalysis()) return;
  const report = {
    product: "GridSynth",
    reportSchema: "gridsynth-scenario-report-v1",
    source: {
      name: state.sourceName,
      classification: "uploaded-csv",
    },
    generatedAt: new Date().toISOString(),
    observations: state.rows.length,
    dataset: state.contract,
    model: {
      version: MODEL_VERSION,
      expression: elements.equation.textContent,
      rSquared: state.model.rSquared,
      residualThresholdSigma: 1.75,
    },
    scenario: scenarioValues(),
    baselineForecast: state.baselineForecast.map((row) => ({ timestamp: row.timestamp.toISOString(), loadMw: row.load, temperatureC: row.temperature })),
    forecast: state.forecast.map((row) => ({ timestamp: row.timestamp.toISOString(), loadMw: row.load, temperatureC: row.temperature })),
    findings: state.anomalies.map((item) => ({ timestamp: item.row.timestamp.toISOString(), loadMw: item.row.load, residualMw: item.residual, sigma: item.score, outageMinutes: item.row.outageMinutes })),
  };
  const blob = new Blob([JSON.stringify(report, null, 2)], { type: "application/json" });
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = "gridsynth-uploaded-scenario-report.json";
  link.click();
  URL.revokeObjectURL(link.href);
  state.reportExported = true;
  renderWorkflowStatus();
}

elements.exportButton.addEventListener("click", exportReport);
elements.routeExportButton.addEventListener("click", exportReport);

renderWorkflowStatus();
