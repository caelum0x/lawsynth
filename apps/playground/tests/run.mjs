import { lstat, mkdir, readdir, rmdir, symlink, unlink } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawn } from "node:child_process";

const appDirectory = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repositoryRoot = resolve(appDirectory, "../..");
const scopeDirectory = resolve(repositoryRoot, "node_modules/@lawsynth");
const packages = ["chart-core", "layout-engine", "world-schema", "world-viewer"];
const createdLinks = [];
let createdScopeDirectory = false;
let createdNodeModulesDirectory = false;

async function pathExists(path) {
  try {
    await lstat(path);
    return true;
  } catch (error) {
    if (error && typeof error === "object" && error.code === "ENOENT") return false;
    throw error;
  }
}

async function ensureWorkspaceLinks() {
  const nodeModulesDirectory = resolve(repositoryRoot, "node_modules");
  if (!await pathExists(nodeModulesDirectory)) {
    await mkdir(nodeModulesDirectory);
    createdNodeModulesDirectory = true;
  }
  if (!await pathExists(scopeDirectory)) {
    await mkdir(scopeDirectory);
    createdScopeDirectory = true;
  }
  for (const name of packages) {
    const link = resolve(scopeDirectory, name);
    if (await pathExists(link)) continue;
    await symlink(resolve(repositoryRoot, "packages", name), link, "dir");
    createdLinks.push(link);
  }
}

async function removeOwnedLinks() {
  for (const link of createdLinks.reverse()) await unlink(link);
  if (createdScopeDirectory) await rmdir(scopeDirectory);
  if (createdNodeModulesDirectory) await rmdir(resolve(repositoryRoot, "node_modules"));
}

function runNodeTests(files) {
  return new Promise((resolvePromise, reject) => {
    const child = spawn(process.execPath, ["--test", ...files], { cwd: appDirectory, stdio: "inherit" });
    child.once("error", reject);
    child.once("exit", (code, signal) => {
      if (code === 0) resolvePromise();
      else reject(new Error(`playground test process exited with ${signal ?? code ?? "an unknown status"}`));
    });
  });
}

try {
  await ensureWorkspaceLinks();
  const tests = (await readdir(resolve(appDirectory, "dist/tests")))
    .filter((file) => file.endsWith(".test.js"))
    .sort()
    .map((file) => `dist/tests/${file}`);
  if (tests.length === 0) throw new Error("playground build produced no test files");
  await runNodeTests(tests);
} finally {
  await removeOwnedLinks();
}
