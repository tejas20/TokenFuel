import sharp from "sharp";

// Browser captures are 1280x720 at density 1. Keep README images at native scale.
for (const [name, left, width, height] of [
  ["strip", 394, 492, 77],
  ["details", 394, 492, 365],
  ["focus", 484, 312, 77],
]) {
  await sharp(`docs/design/${name}-browser-full.jpg`)
    .extract({ left, top: 0, width, height })
    .jpeg({ quality: 95 })
    .toFile(`docs/images/widget-${name}.jpg`);
}

const source = "docs/design/combined-bar-reference.png";
// Compare three component states at equal widths. Preserve source aspect ratios;
// the image board isn't a literal 48px spec and its detail panel omits metadata.
const sourceRegions = [
  [{ left: 65, top: 205, width: 1646, height: 128 }, 480],
  [{ left: 65, top: 590, width: 916, height: 104 }, 300],
  [{ left: 1060, top: 345, width: 635, height: 422 }, 340],
];
const actualRegions = [
  ["strip", { left: 6, top: 6, width: 480, height: 48 }],
  ["focus", { left: 6, top: 6, width: 300, height: 48 }],
  ["details", { left: 146, top: 79, width: 340, height: 280 }],
];
const layers = [];
const rows = [12, 92, 172];
for (let i = 0; i < rows.length; i++) {
  layers.push({
    input: await sharp(source)
      .extract(sourceRegions[i][0])
      .resize(sourceRegions[i][1])
      .png()
      .toBuffer(),
    left: 12,
    top: rows[i],
  });
  layers.push({
    input: await sharp(`docs/images/widget-${actualRegions[i][0]}.jpg`)
      .extract(actualRegions[i][1])
      .png()
      .toBuffer(),
    left: 516,
    top: rows[i],
  });
}
await sharp({
  create: { width: 1010, height: 510, channels: 3, background: "#111111" },
})
  .composite(layers)
  .png()
  .toFile("docs/design/combined-comparison.png");
