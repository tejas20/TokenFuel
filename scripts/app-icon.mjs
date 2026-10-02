import fs from "node:fs/promises";
import sharp from "sharp";
const svg = (await fs.readFile("docs/design/gas-pump.svg", "utf8")).replaceAll(
  "currentColor",
  "#42998b",
);
await sharp(Buffer.from(svg))
  .resize(1024, 1024)
  .png()
  .toFile("docs/design/app-icon.png");
