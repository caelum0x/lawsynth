import type { ArtifactDescriptor, SimulationId, SimulationSummary } from "./generated.js";
import { pathSegment, type Transport } from "./transport.js";
export class SimulationsApi {
  constructor(private readonly transport: Transport) {}
  get(id: SimulationId, signal?: AbortSignal): Promise<SimulationSummary> { return this.transport.request({ path: `/v1/simulations/${pathSegment(id)}`, signal }); }
  cancel(id: SimulationId, signal?: AbortSignal): Promise<SimulationSummary> { return this.transport.request({ method: "POST", path: `/v1/simulations/${pathSegment(id)}/cancel`, idempotencyKey: `cancel-${id}`, signal }); }
  artifact(id: SimulationId, signal?: AbortSignal): Promise<ArtifactDescriptor> { return this.transport.request({ path: `/v1/simulations/${pathSegment(id)}/artifact`, signal }); }
}
