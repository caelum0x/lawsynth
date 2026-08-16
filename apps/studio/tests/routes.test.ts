import { parseRoute, routePath, StudioRouter } from "../src/routes.js";
import { deepEqual, equal, rejects } from "./support.js";

export async function routesTests(): Promise<void> {
  const route = parseRoute("/projects/project_1/worlds/world_1?panel=provenance");
  deepEqual(route, { name: "world", projectId: "project_1", worldId: "world_1", panel: "provenance" });
  equal(routePath(route), "/projects/project_1/worlds/world_1?panel=provenance");
  const router = new StudioRouter(); let destination = "";
  router.addEventListener("navigate", (event) => { destination = (event as CustomEvent<{ path: string }>).detail.path; });
  router.navigate({ name: "settings" }, true); equal(destination, "/settings");
  await rejects(() => Promise.resolve(parseRoute("/projects/no spaces")), /invalid/);
}
