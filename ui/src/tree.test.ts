import { describe, expect, it } from "vitest";
import { due, tree } from "./Home";

describe("the objectives tree", () => {
  it("lists roots then their children, depth first", () => {
    const t = tree([
      { id: "b", parent: "a" },
      { id: "a" },
      { id: "c", parent: "b" },
      { id: "d", parent: "gone" },
    ]);
    expect(t.map((x) => [x.row.id, x.depth])).toEqual([
      ["a", 0],
      ["b", 1],
      ["c", 2],
      ["d", 0],
    ]);
  });
});

describe("a check-in due", () => {
  const today = "2026-11-06";
  it("comes once its days have passed since the last", () => {
    expect(due({ id: "a", last_check_in: "2026-10-30", check_in_every: 7 }, today)).toBe(true);
    expect(due({ id: "a", last_check_in: "2026-10-31", check_in_every: 7 }, today)).toBe(false);
    expect(due({ id: "a", last_check_in: "2026-11-04" }, today)).toBe(false);
    expect(due({ id: "a", last_check_in: "2026-10-29" }, today)).toBe(true);
  });
  it("never for one measured from the records, never asked, or never checked in", () => {
    expect(due({ id: "a", last_check_in: "2026-01-01", measure: { entity: "Project" } }, today)).toBe(false);
    expect(due({ id: "a", last_check_in: "2026-01-01", metric: "m-1" }, today)).toBe(false);
    expect(due({ id: "a", last_check_in: "2026-01-01", check_in_every: 0 }, today)).toBe(false);
    expect(due({ id: "a" }, today)).toBe(false);
  });
});
