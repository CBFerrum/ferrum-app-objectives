import { describe, expect, it } from "vitest";
import { tree } from "./Home";

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
