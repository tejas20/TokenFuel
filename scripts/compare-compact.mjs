import sharp from "sharp";
const root = "docs/design/";
const captures = [
  ["bars", 667, 6, 381, 101],
  ["rings", 737, 6, 241, 133],
  ["light", 737, 6, 241, 133],
  ["narrow", 12, 6, 336, 133],
];
for (const [name, left, top, width, height] of captures) {
  await sharp(`${root}compact-live-${name}-full.png`)
    .extract({ left, top, width, height }).png().toFile(`${root}compact-${name}.png`);
}
const source = `${root}compact-selected.png`;
const items = [
  await sharp(source).extract({ left: 122, top: 189, width: 1278, height: 252 }).resize(760).png().toBuffer(),
  await sharp(`${root}compact-bars.png`).resize(760).png().toBuffer(),
  await sharp(source).extract({ left: 430, top: 638, width: 674, height: 313 }).resize(480).png().toBuffer(),
  await sharp(`${root}compact-rings.png`).resize(480).png().toBuffer(),
];
let top = 12;
const layers = [];
for (const input of items) {
  layers.push({ input, left: 12, top });
  top += (await sharp(input).metadata()).height + 16;
}
await sharp({ create: { width: 784, height: top, channels: 3, background: "#0f1720" } })
  .composite(layers).png().toFile(`${root}compact-comparison.png`);
await sharp({ create: { width: 800, height: 540, channels: 3, background: "#111827" } })
  .composite([
    { input: await sharp(`${root}compact-bars.png`).resize(760).png().toBuffer(), left: 20, top: 20 },
    { input: await sharp(`${root}compact-rings.png`).resize(480).png().toBuffer(), left: 160, top: 250 },
  ]).png().toFile(`${root}compact-delivery.png`);
