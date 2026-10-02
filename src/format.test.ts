import { expect, test } from "vitest";
import { countdown, percent } from "./format";
test("unknown counters and elapsed resets stay explicit", () => {
  expect(percent(null)).toBe("—");
  expect(countdown(null)).toBe("Reset not reported");
  expect(
    countdown("2025-01-01T00:00:00Z", Date.parse("2025-01-02T00:00:00Z")),
  ).toBe("Reset due · refresh");
  expect(
    countdown("2025-01-17T12:00:00Z", Date.parse("2025-01-15T00:00:00Z")),
  ).toBe("Resets in 2d 12h");
});
