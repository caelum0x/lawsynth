import { DatasetPicker, parseNumericCsv } from "../src/dataset_picker.js";
import { deepEqual, equal, test, throws } from "./testkit.js";

await test("numeric CSV ingestion preserves quoted fields, missing values, and deterministic ids", () => {
  const first = parseNumericCsv("\uFEFFtime,value\r\n0,1\r\n1,\r\n2,\"3\"\r\n", "Run A");
  const second = parseNumericCsv("\uFEFFtime,value\r\n0,1\r\n1,\r\n2,\"3\"\r\n", "Renamed");
  deepEqual(first.columns, ["time", "value"]);
  deepEqual(first.rows, [[0, 1], [1, null], [2, 3]]);
  equal(first.id, second.id);
  throws(() => parseNumericCsv("x,x\n1,2\n"), /unique/);
  throws(() => parseNumericCsv("x\nnot-a-number\n"), /not numeric/);
});

await test("dataset selection is constrained to registered datasets", () => {
  const picker = new DatasetPicker();
  const alpha = parseNumericCsv("t,x\n0,2\n", "Alpha");
  const beta = parseNumericCsv("t,x\n0,3\n", "Beta");
  picker.add(beta);
  picker.add(alpha);
  deepEqual(picker.items.map((dataset) => dataset.name), ["Alpha", "Beta"]);
  picker.select(alpha.id);
  equal(picker.selected?.id, alpha.id);
  equal(picker.remove(alpha.id), true);
  equal(picker.selected, undefined);
  throws(() => picker.select(alpha.id), /unknown dataset/);
});
