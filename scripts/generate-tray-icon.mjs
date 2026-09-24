// Generates the menu bar icon at apps/desktop/src-tauri/icons/tray.png.
//
// macOS renders menu bar icons as template images, using only the alpha channel
// so the glyph inverts with the menu bar. That rules out the app artwork, whose
// detail also disappears at 22pt, so this draws a flat three-node chart mark.
//
// Run with: pnpm generate:tray-icon

import { writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { deflateSync } from "node:zlib";

const SIZE = 44;
const STROKE = 2;

const OUTPUT_PATH = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "apps",
  "desktop",
  "src-tauri",
  "icons",
  "tray.png",
);

const alpha = new Uint8Array(SIZE * SIZE);

function fillRect(left, top, right, bottom) {
  for (let y = Math.max(0, top); y < Math.min(SIZE, bottom); y += 1) {
    for (let x = Math.max(0, left); x < Math.min(SIZE, right); x += 1) {
      alpha[y * SIZE + x] = 255;
    }
  }
}

function strokeRect(left, top, width, height) {
  fillRect(left, top, left + width, top + STROKE);
  fillRect(left, top + height - STROKE, left + width, top + height);
  fillRect(left, top, left + STROKE, top + height);
  fillRect(left + width - STROKE, top, left + width, top + height);
}

strokeRect(15, 4, 14, 10); // parent node
fillRect(21, 14, 23, 20); // stem below the parent
fillRect(9, 20, 35, 22); // horizontal connector
fillRect(9, 22, 11, 29); // drop to the left child
fillRect(33, 22, 35, 29); // drop to the right child
strokeRect(4, 29, 14, 10); // left child node
strokeRect(26, 29, 14, 10); // right child node

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);

  for (let index = 0; index < 256; index += 1) {
    let value = index;
    for (let bit = 0; bit < 8; bit += 1) {
      value = value & 1 ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
    }
    table[index] = value >>> 0;
  }

  return table;
})();

function crc32(buffer) {
  let crc = 0xffffffff;

  for (const byte of buffer) {
    crc = CRC_TABLE[(crc ^ byte) & 0xff] ^ (crc >>> 8);
  }

  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length);

  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const checksum = Buffer.alloc(4);
  checksum.writeUInt32BE(crc32(body));

  return Buffer.concat([length, body, checksum]);
}

function header() {
  const data = Buffer.alloc(13);
  data.writeUInt32BE(SIZE, 0);
  data.writeUInt32BE(SIZE, 4);
  data[8] = 8; // bits per channel
  data[9] = 6; // RGBA
  return data;
}

function pixels() {
  const data = Buffer.alloc(SIZE * (1 + SIZE * 4));
  let offset = 0;

  for (let y = 0; y < SIZE; y += 1) {
    data[offset] = 0; // no scanline filter
    offset += 1;

    for (let x = 0; x < SIZE; x += 1) {
      // Template images ignore colour, so only the alpha channel carries shape.
      data[offset + 3] = alpha[y * SIZE + x];
      offset += 4;
    }
  }

  return data;
}

writeFileSync(
  OUTPUT_PATH,
  Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", header()),
    chunk("IDAT", deflateSync(pixels(), { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]),
);

console.log(`wrote ${OUTPUT_PATH}`);
