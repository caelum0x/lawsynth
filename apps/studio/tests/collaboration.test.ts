import { InMemoryCollaborationTransport, LawSynthClient } from "@lawsynth/api-client";
import { SCREEN_IDS } from "../src/screens/index.js";
import type { ActionButton, ScreenModel, TableRow } from "../src/screens/index.js";
import { ScreensController } from "../src/screens/controller.js";
import { equal, store, world } from "./support.js";

function tableRows(model: ScreenModel, id: string): readonly TableRow[] {
  const section = model.sections.find((entry) => entry.kind === "table" && entry.id === id);
  if (section === undefined || section.kind !== "table") throw new Error(`no table section ${id}`);
  return section.rows;
}

function button(model: ScreenModel, id: string): ActionButton {
  for (const section of model.sections) {
    if (section.kind === "actions") {
      const found = section.buttons.find((candidate) => candidate.id === id);
      if (found !== undefined) return found;
    }
  }
  throw new Error(`no action button ${id}`);
}

function metricValue(model: ScreenModel, label: string): string {
  for (const section of model.sections) {
    if (section.kind === "metrics") {
      const metric = section.metrics.find((candidate) => candidate.label === label);
      if (metric !== undefined) return metric.value;
    }
  }
  throw new Error(`no metric ${label}`);
}

/**
 * Proves the LIVE-wired Studio collaboration screen end to end offline: the
 * screen loads membership + revision lineage + annotations through the real
 * api-client (over a fake transport), gates review/annotate/membership actions
 * by the acting role (read from the shared store), and drives the review state
 * machine and annotation/member mutations back through the client.
 */
export async function collaborationTests(): Promise<void> {
  // The 12th screen is registered.
  equal(SCREEN_IDS.length, 12);
  equal(SCREEN_IDS.includes("collaboration"), true);

  const transport = new InMemoryCollaborationTransport({
    projectId: "project-collab",
    worldId: world.id,
    owner: "token:owner",
    members: { "token:editor": "editor" },
  });
  const api = new LawSynthClient(transport);
  const shared = store();
  const controller = new ScreensController({ store: shared, api, randomId: () => "idem", world });
  shared.dispatch({ kind: "workspace.update", patch: { projectId: "project-collab" } });

  controller.setScreen("collaboration");
  await controller.onAction("collab:refresh"); // deterministic load

  // Members + roles rendered from the service.
  let model = controller.model();
  const members = tableRows(model, "collab-members");
  equal(members.length, 2);
  equal(members.some((row) => row.id === "token:owner"), true);
  equal(members.some((row) => row.id === "token:editor"), true);

  // Revision lineage: one seeded draft revision.
  equal(tableRows(model, "collab-revisions").length, 1);
  equal(metricValue(model, "Revisions"), "1");

  // Select revision 1 (through the shared store).
  controller.onSelect("collab-revisions", "1");

  // Role gate — a viewer can mutate nothing.
  controller.onControl("collab:role", "viewer");
  model = controller.model();
  equal(metricValue(model, "Acting role"), "viewer");
  equal(button(model, "collab:request-review").disabled, true);
  equal(button(model, "collab:approve").disabled, true);
  equal(button(model, "collab:add-annotation").disabled, true);

  // An editor may request review, but may NOT approve (owner-only).
  controller.onControl("collab:role", "editor");
  model = controller.model();
  equal(button(model, "collab:request-review").disabled, false);
  await controller.onAction("collab:request-review");
  model = controller.model();
  equal(metricValue(model, "Selected revision"), "#1 · in_review");
  equal(button(model, "collab:approve").disabled, true); // editor cannot approve

  // Only an owner may approve; the world then becomes trusted.
  controller.onControl("collab:role", "owner");
  equal(button(controller.model(), "collab:approve").disabled, false);
  await controller.onAction("collab:approve");
  model = controller.model();
  equal(metricValue(model, "Selected revision"), "#1 · approved");
  equal(metricValue(model, "World trusted"), "yes");

  // Editor adds an annotation; it flows back through the client and reloads.
  controller.onControl("collab:role", "editor");
  controller.onControl("collab:annotation-text", "second-order term");
  await controller.onAction("collab:add-annotation");
  model = controller.model();
  equal(tableRows(model, "collab-annotations").length, 1);
  equal(tableRows(model, "collab-annotations")[0]?.cells.includes("second-order term"), true);

  // Owner adds a member; membership reloads with the new viewer.
  controller.onControl("collab:role", "owner");
  controller.onControl("collab:member-principal", "token:viewer");
  controller.onControl("collab:member-role", "viewer");
  await controller.onAction("collab:add-member");
  model = controller.model();
  equal(tableRows(model, "collab-members").some((row) => row.id === "token:viewer"), true);

  controller.dispose();
}
