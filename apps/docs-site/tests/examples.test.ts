import { createDocumentationAnalytics, recordDocumentationPageView } from "../examples/analytics.example.js";
import { resolveLegacyDocumentationPath } from "../examples/redirects.example.js";
import { renderDocumentationSeo } from "../examples/seo.example.js";
import { documentationThemeBootstrap } from "../examples/theme.example.js";
import { documentationVersions } from "../examples/versions.example.js";
import { ExampleRegistry } from "../src/examples.js";
import { contains, deepEqual, equal, throws } from "./assertions.js";

const received: string[] = [];
const sink = { send: async (event: { readonly name: string; readonly path: string }) => { received.push(event.name + ":" + event.path); } };
await recordDocumentationPageView(sink, "/guide/install");

deepEqual(received, ["page_view:/guide/install"]);
createDocumentationAnalytics(sink).setEnabled(false);
equal(resolveLegacyDocumentationPath("/docs/quickstart")!.to, "/guide/quickstart");
contains(renderDocumentationSeo({ title: "Install LawSynth", description: "Install the native LawSynth discovery and simulation toolkit.", canonicalUrl: "https://docs.lawsynth.dev/guide/install" }), "og:title");
contains(documentationThemeBootstrap(), "lawsynth:docs:theme");
equal(documentationVersions.current.version, "0.1.0");

const registry = new ExampleRegistry();
registry.add({ id: "discover-python", title: "Discover with Python", description: "Run native sparse discovery from Python.", language: "python", source: "lawsynth.discover(data)", runnable: true, capabilities: ["discovery", "python", "discovery"] });
equal(registry.list("discovery")[0]!.capabilities.join(","), "discovery,python");
throws(() => registry.add({ id: "discover-python", title: "Duplicate", description: "This duplicate id must be rejected by the registry.", language: "python", source: "pass", runnable: false, capabilities: [] }), /duplicate/);
