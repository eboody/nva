import { chromium } from "playwright";
import { cpSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawn } from "node:child_process";

const appRoot = resolve(import.meta.dirname, "..");
const candidates = [
  join(appRoot, ".next", "standalone", "apps", "staff-web", "server.js"),
  join(appRoot, ".next", "standalone", "server.js")
];
const serverPath = candidates.find(existsSync);
if (!serverPath) {
  throw new Error(`standalone server not found; checked ${candidates.join(", ")}`);
}

const serverRoot = dirname(serverPath);
const staticSource = join(appRoot, ".next", "static");
const staticTarget = join(serverRoot, ".next", "static");
if (existsSync(staticSource)) cpSync(staticSource, staticTarget, { recursive: true });
const publicSource = join(appRoot, "public");
const publicTarget = join(serverRoot, "public");
if (existsSync(publicSource)) cpSync(publicSource, publicTarget, { recursive: true });

const port = "3101";
const origin = `http://127.0.0.1:${port}`;
const server = spawn(process.execPath, [serverPath], {
  cwd: serverRoot,
  env: { ...process.env, HOSTNAME: "127.0.0.1", PORT: port },
  stdio: ["ignore", "pipe", "pipe"]
});
let output = "";
server.stdout.on("data", (chunk) => { output += chunk; });
server.stderr.on("data", (chunk) => { output += chunk; });

const waitForServer = async () => {
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(origin);
      if (response.ok) return;
    } catch {}
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 200));
  }
  throw new Error(`standalone server did not become ready: ${output}`);
};

let browser;
try {
  await waitForServer();
  browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  await page.goto(origin);
  const heading = page.getByRole("heading", { level: 1, name: "Owned Operations Platform" });
  if (!(await heading.isVisible())) throw new Error("standalone page heading was not visible");
  const tool = page.getByRole("button", { name: "Inspect Intake / Booking Triage lineage" });
  await tool.click();
  if ((await tool.getAttribute("aria-pressed")) !== "true") {
    throw new Error("standalone client hydration did not activate the selected tool");
  }
  console.log("standalone browser startup verified");
} finally {
  if (browser) await browser.close();
  server.kill("SIGTERM");
}
