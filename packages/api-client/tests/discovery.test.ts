import test from "node:test";
import assert from "node:assert/strict";
import { InMemoryDiscoveryTransport, LawSynthClient } from "../dist/index.js";

test("discovery client drives submit -> poll -> world through a fake transport", async () => {
  const transport = new InMemoryDiscoveryTransport({ pollsUntilSucceeded: 1 });
  const client = new LawSynthClient(transport);

  const submitted = await client.runs.submitDiscovery(
    { states: ["x", "v"], dataset_id: "dataset-1", discovery: { polynomial_degree: 3, threshold: 0.1 } },
    "idem-1",
  );
  assert.equal(submitted.status, "queued");

  // First poll reports running, the second settles to succeeded.
  const first = await client.runs.get(submitted.id);
  assert.equal(first.status, "running");
  const second = await client.runs.get(submitted.id);
  assert.equal(second.status, "succeeded");
  assert.equal(second.world_id, "world-oscillator");

  const runWorld = await client.runs.getWorld(submitted.id);
  assert.equal(runWorld.world_id, "world-oscillator");
  assert.deepEqual(runWorld.world.states, ["x", "v"]);
  assert.equal(runWorld.world.equations["v"], "-4*x - 0.5*v");
});

test("world product actions are reachable through the client", async () => {
  const client = new LawSynthClient(new InMemoryDiscoveryTransport({ pollsUntilSucceeded: 0 }));
  const run = await client.runs.submitDiscovery({ states: ["x", "v"], dataset_id: "dataset-1" }, "idem-2");
  await client.runs.get(run.id); // settle to succeeded
  const world = await client.runs.getWorld(run.id);

  const explanation = await client.worlds.explain(world.world_id);
  assert.equal(explanation.laws.length, 2);
  const forecast = await client.worlds.forecast(world.world_id, { horizon: 1, step: 0.5, initial: { x: 1, v: 0 } }, "idem-3");
  assert.ok(forecast.trajectory.time.length > 0);
  const report = await client.worlds.report(world.world_id);
  assert.match(report, /LawSynth World/);
});

test("getWorld before completion surfaces a conflict", async () => {
  const client = new LawSynthClient(new InMemoryDiscoveryTransport({ pollsUntilSucceeded: 3 }));
  const run = await client.runs.submitDiscovery({ states: ["x"], dataset_id: "d" }, "idem-4");
  await client.runs.get(run.id); // still running
  await assert.rejects(() => client.runs.getWorld(run.id), /world/);
});
