import { spawn } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";

const chromePath = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const appUrl = "http://127.0.0.1:1420/";
const maximumVerticalOffset = 1;

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
        await new Promise((resolve) => setTimeout(resolve, 500));
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
    socket.addEventListener(
      "message",
      (event) => {
        const message = JSON.parse(event.data);
        if (message.id !== 1) {
          return;
        }

        if (message.error) {
          reject(new Error(message.error.message));
        } else if (message.result.exceptionDetails) {
          reject(
            new Error(
              message.result.exceptionDetails.exception?.description ??
                message.result.exceptionDetails.text,
            ),
          );
        } else {
          resolve(message.result.result.value);
        }
      },
      { once: false },
    );

    socket.send(
      JSON.stringify({
        id: 1,
        method: "Runtime.evaluate",
        params: {
          awaitPromise: true,
          expression,
          returnByValue: true,
        },
      }),
    );
  });

  socket.close();
  return result;
}

const measurementExpression = String.raw`(async () => {
  const injectedCss = ${JSON.stringify(process.env.ALIGNMENT_CSS ?? "")};
  if (injectedCss) {
    const style = document.createElement("style");
    style.textContent = injectedCss;
    document.head.append(style);
  }

  await document.fonts.ready;

  for (let attempt = 0; attempt < 100; attempt += 1) {
    const controls = document.querySelectorAll(".mermaid-controls-95__button");
    if (controls.length >= 2) {
      break;
    }
    await new Promise((resolve) => setTimeout(resolve, 25));
  }

  return [
    { accessibleName: "Zoom out", label: "−" },
    { accessibleName: "Zoom in", label: "+" },
  ].map(({ accessibleName, label }) => {
    const button = document.querySelector(
      '.mermaid-controls-95__button[aria-label="' + accessibleName + '"]'
    );
    if (!button) {
      throw new Error(
        "Missing Mermaid control button: " + label +
        "; labels=" + [...document.querySelectorAll(".mermaid-controls-95__button")]
          .map((candidate) => JSON.stringify(candidate.getAttribute("aria-label")))
          .join(",") +
        "; body=" + document.body.innerText.slice(0, 200)
      );
    }

    const buttonBox = button.getBoundingClientRect();
    const glyphLines = [...button.querySelectorAll(".mermaid-controls-95__glyph-line")];
    if (glyphLines.length > 0) {
      const lineBoxes = glyphLines.map((line) => line.getBoundingClientRect());
      const inkTop = Math.min(...lineBoxes.map((box) => box.top));
      const inkBottom = Math.max(...lineBoxes.map((box) => box.bottom));
      const inkCenter = (inkTop + inkBottom) / 2;
      const buttonCenter = buttonBox.top + buttonBox.height / 2;

      return {
        label,
        buttonCenter,
        inkCenter,
        display: getComputedStyle(button.firstElementChild).display,
        lineHeight: getComputedStyle(button.firstElementChild).lineHeight,
        paddingBottom: getComputedStyle(button).paddingBottom,
        paddingTop: getComputedStyle(button).paddingTop,
        verticalOffset: inkCenter - buttonCenter,
      };
    }

    const textNode = button.querySelector("span") ?? button;
    const style = getComputedStyle(textNode);
    const range = document.createRange();
    range.selectNodeContents(textNode);
    const lineBox = range.getBoundingClientRect();

    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    context.font = [style.fontStyle, style.fontWeight, style.fontSize, style.fontFamily].join(" ");
    const metrics = context.measureText(label);
    const fontAscent = metrics.fontBoundingBoxAscent;
    const fontDescent = metrics.fontBoundingBoxDescent;
    const baseline =
      lineBox.top + (lineBox.height - fontAscent - fontDescent) / 2 + fontAscent;
    const inkCenter =
      baseline + (metrics.actualBoundingBoxDescent - metrics.actualBoundingBoxAscent) / 2;
    const buttonCenter = buttonBox.top + buttonBox.height / 2;

    return {
      label,
      buttonCenter,
      inkCenter,
      display: style.display,
      lineHeight: style.lineHeight,
      paddingBottom: getComputedStyle(button).paddingBottom,
      paddingTop: getComputedStyle(button).paddingTop,
      verticalOffset: inkCenter - buttonCenter,
    };
  });
})()`;

const profileDirectory = await mkdtemp(join(tmpdir(), "mermaid-live-alignment-"));
const port = await reservePort();
const chrome = spawn(
  chromePath,
  [
    "--headless=new",
    "--disable-gpu",
    "--no-first-run",
    "--no-default-browser-check",
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
    const offset = Number(measurement.verticalOffset.toFixed(2));
    console.log(
      `${measurement.label}: vertical offset ${offset}px ` +
        `(display=${measurement.display}, line-height=${measurement.lineHeight}, ` +
        `padding=${measurement.paddingTop}/${measurement.paddingBottom})`,
    );
    if (Math.abs(offset) > maximumVerticalOffset) {
      failed = true;
    }
  }

  if (failed) {
    console.error(`Control glyphs must be within ${maximumVerticalOffset}px of vertical center.`);
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
