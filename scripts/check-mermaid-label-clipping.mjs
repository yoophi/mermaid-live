import { spawn } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";

const chromePath = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const appUrl = "http://127.0.0.1:1420/";
const maximumOverflow = 0.1;

function reservePort() {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        reject(new Error("Could not reserve a Chrome debugging port."));
        return;
      }

      const { port } = address;
      server.close((error) => (error ? reject(error) : resolve(port)));
    });
  });
}

async function waitForPage(port) {
  const endpoint = `http://127.0.0.1:${port}/json/list`;
  for (let attempt = 0; attempt < 200; attempt += 1) {
    try {
      const pages = await fetch(endpoint).then((response) => response.json());
      const page = pages.find(
        (candidate) => candidate.type === "page" && candidate.url.startsWith(appUrl),
      );
      if (page) {
        return page;
      }
    } catch {
      // Chrome has not opened its debugging endpoint yet.
    }

    await new Promise((resolve) => setTimeout(resolve, 100));
  }

  throw new Error("Chrome did not expose the Mermaid Live page in time.");
}

async function evaluate(webSocketUrl, expression) {
  const socket = new WebSocket(webSocketUrl);
  await new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });

  const result = await new Promise((resolve, reject) => {
    socket.addEventListener("message", (event) => {
      const message = JSON.parse(event.data);
      if (message.id !== 1) return;
      if (message.error) reject(new Error(message.error.message));
      else if (message.result.exceptionDetails) {
        reject(
          new Error(
            message.result.exceptionDetails.exception?.description ??
              message.result.exceptionDetails.text,
          ),
        );
      } else resolve(message.result.result.value);
    });

    socket.send(
      JSON.stringify({
        id: 1,
        method: "Runtime.evaluate",
        params: { awaitPromise: true, expression, returnByValue: true },
      }),
    );
  });

  socket.close();
  return result;
}

const measurementExpression = String.raw`(async () => {
  await document.fonts.ready;

  let svg;
  for (let attempt = 0; attempt < 200; attempt += 1) {
    svg = document.querySelector(".mermaid-preview svg");
    if (svg?.querySelector("foreignObject")) break;
    await new Promise((resolve) => setTimeout(resolve, 25));
  }
  if (!svg) throw new Error("Mermaid preview SVG was not rendered.");

  await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));

  return [...svg.querySelectorAll("foreignObject")]
    .map((foreignObject) => {
      const label = foreignObject.textContent.replace(/\s+/g, " ").trim();
      if (!label) return null;

      const container = foreignObject.getBoundingClientRect();
      const walker = document.createTreeWalker(foreignObject, NodeFilter.SHOW_TEXT);
      const textRects = [];
      while (walker.nextNode()) {
        if (!walker.currentNode.textContent.trim()) continue;
        const range = document.createRange();
        range.selectNodeContents(walker.currentNode);
        const rect = range.getBoundingClientRect();
        if (rect.width > 0 && rect.height > 0) textRects.push(rect);
      }
      if (textRects.length === 0) return null;

      const ink = {
        left: Math.min(...textRects.map((rect) => rect.left)),
        right: Math.max(...textRects.map((rect) => rect.right)),
        top: Math.min(...textRects.map((rect) => rect.top)),
        bottom: Math.max(...textRects.map((rect) => rect.bottom)),
      };

      return {
        label,
        fontFamily: getComputedStyle(foreignObject.querySelector("div, span, p") ?? foreignObject)
          .fontFamily,
        containerWidth: container.width,
        inkWidth: ink.right - ink.left,
        overflow: {
          left: Math.max(0, container.left - ink.left),
          right: Math.max(0, ink.right - container.right),
          top: Math.max(0, container.top - ink.top),
          bottom: Math.max(0, ink.bottom - container.bottom),
        },
      };
    })
    .filter(Boolean);
})()`;

const profileDirectory = await mkdtemp(join(tmpdir(), "mermaid-live-labels-"));
const port = await reservePort();
const chrome = spawn(
  chromePath,
  [
    "--headless=new",
    "--disable-gpu",
    "--no-first-run",
    "--no-default-browser-check",
    "--window-size=1280,720",
    `--remote-debugging-port=${port}`,
    `--user-data-dir=${profileDirectory}`,
    appUrl,
  ],
  { stdio: "ignore" },
);

try {
  const page = await waitForPage(port);
  const measurements = await evaluate(page.webSocketDebuggerUrl, measurementExpression);
  let failed = false;

  for (const measurement of measurements) {
    const worstOverflow = Math.max(...Object.values(measurement.overflow));
    console.log(
      `${JSON.stringify(measurement.label)}: overflow=${worstOverflow.toFixed(2)}px; ` +
        `container=${measurement.containerWidth.toFixed(2)}px; ` +
        `ink=${measurement.inkWidth.toFixed(2)}px; font=${measurement.fontFamily}`,
    );
    if (worstOverflow > maximumOverflow) failed = true;
  }

  if (failed) {
    console.error(
      `Mermaid label ink must remain within its container by ${maximumOverflow}px.`,
    );
    process.exitCode = 1;
  }
} finally {
  await new Promise((resolve) => {
    const timeout = setTimeout(resolve, 2_000);
    chrome.once("exit", () => {
      clearTimeout(timeout);
      resolve();
    });
    chrome.kill("SIGTERM");
  });
  await rm(profileDirectory, {
    force: true,
    maxRetries: 5,
    recursive: true,
    retryDelay: 100,
  });
}
