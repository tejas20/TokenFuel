import sharp from "sharp";
// Normalize framed source crops and browser screenshot density before comparing.
const source = "docs/design/selected-dock.png";
const sourceBars = await sharp(source)
  .extract({ left: 126, top: 172, width: 1198, height: 303 })
  .resize(832)
  .png()
  .toBuffer();
const sourceRings = await sharp(source)
  .extract({ left: 126, top: 537, width: 1198, height: 387 })
  .resize(832)
  .png()
  .toBuffer();
// CSS viewport 860x760, density 1. Browser API captures the actual widget region.
const bars = await sharp("docs/design/tokenfuel-bars.png").png().toBuffer();
const rings = await sharp("docs/design/tokenfuel-rings.png").png().toBuffer();
const items = [sourceBars, bars, sourceRings, rings];
let y = 0;
const composites = [];
for (const item of items) {
  composites.push({ input: item, left: 0, top: y });
  y += (await sharp(item).metadata()).height + 16;
}
await sharp({
  create: { width: 832, height: y, channels: 3, background: "#111827" },
})
  .composite(composites)
  .png()
  .toFile("docs/design/comparison.png");
