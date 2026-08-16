import { datasetViewModel, profilePreview, validatePreview } from "../src/dataset.js";
import { deepEqual, equal, rejects } from "./support.js";

export async function datasetTests(): Promise<void> {
  const preview = validatePreview({ columns: [" time ", "prey", "predator"], rows: [["2026-01-01T00:00:00Z", 4, 1], ["2026-01-02T00:00:00Z", 6, null]], totalRows: 2, truncated: false });
  deepEqual(preview.columns, ["time", "prey", "predator"]);
  const profiles = profilePreview(preview);
  equal(profiles[1]?.minimum, 4); equal(profiles[1]?.maximum, 6); equal(profiles[2]?.missing, 1);
  const model = datasetViewModel({ id: "dataset_1", organization_id: "org", name: "observations", schema: ["time", "prey", "predator"], created_at: "2026-01-01T00:00:00Z", deleted_at: null }, preview);
  equal(model.usableForDiscovery, true);
  await rejects(() => validatePreview({ columns: ["x", "x"], rows: [], truncated: false }), /unique/);
}
