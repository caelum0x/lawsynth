import { PrivacyAnalytics, type AnalyticsSink } from "../src/analytics.js";

/**
 * Build the privacy-preserving tracker around the application's actual sink.
 * The docs package never chooses or emulates a network transport.
 */
export function createDocumentationAnalytics(sink: AnalyticsSink, enabled = true): PrivacyAnalytics {
  return new PrivacyAnalytics(sink, enabled);
}

/** Record one path-only view and wait until the supplied production sink accepts it. */
export async function recordDocumentationPageView(sink: AnalyticsSink, path: string): Promise<void> {
  const analytics = createDocumentationAnalytics(sink);
  analytics.track({ name: "page_view", path, properties: { surface: "documentation" } });
  await analytics.flush();
}
