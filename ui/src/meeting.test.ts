import { describe, expect, it } from "vitest";
import { change, daysBefore, progressOf, review, valueOn, weighted, type Row } from "./meeting";

describe("the review meeting", () => {
  it("works progress out as the server does", () => {
    expect([progressOf("Increase", 100, 200, 150), progressOf("Decrease", 40, 20, 30), progressOf("Increase", 0, 3, 1)]).toEqual([50, 50, 33.3]);
    expect([progressOf("Milestone", 0, 1, 1), progressOf("Milestone", 0, 1, 0), progressOf("Increase", 5, 5, 9), progressOf("Increase", 0, 10, 20)]).toEqual([100, 0, 0, 100]);
  });

  it("reads a key result's value on a day from its check-ins", () => {
    const kr: Row = { id: "k", kind: "Increase", start_value: "10" };
    const checkIns: Row[] = [
      { id: "1", key_result: "k", date: "2026-10-01", value: "20", created_at: "2026-10-01T09:00:00Z" },
      { id: "2", key_result: "k", date: "2026-10-20", value: "35", created_at: "2026-10-20T09:00:00Z" },
      { id: "3", key_result: "k", date: "2026-10-20", value: "40", created_at: "2026-10-20T10:00:00Z" },
      { id: "4", key_result: "other", date: "2026-10-02", value: "99" },
    ];
    expect([valueOn(kr, checkIns, "2026-09-30"), valueOn(kr, checkIns, "2026-10-05"), valueOn(kr, checkIns, "2026-10-20")]).toEqual([10, 20, 40]);
  });

  it("puts those at risk first, then the furthest behind, with what changed", () => {
    const objectives: Row[] = [
      { id: "a", title: "Grow", progress: "80" },
      { id: "b", title: "Ship", progress: "20" },
      { id: "c", title: "Hire", progress: "60" },
    ];
    const results: Row[] = [
      { id: "ka", objective: "a", kind: "Increase", start_value: "0", target_value: "10", progress: "80", confidence: "On Track" },
      { id: "kb", objective: "b", kind: "Increase", start_value: "0", target_value: "10", progress: "20", confidence: "On Track" },
      { id: "kc", objective: "c", kind: "Increase", start_value: "0", target_value: "10", progress: "60", confidence: "Off Track" },
    ];
    const checkIns: Row[] = [{ id: "1", key_result: "ka", date: "2026-10-01", value: "5" }];
    const r = review(objectives, results, checkIns, "2026-10-15");
    expect(r.map((x) => x.objective.id)).toEqual(["c", "b", "a"]);
    expect([r[2].then, r[2].now, change(r[2].then, r[2].now), r[0].risk]).toEqual([50, 80, "+30", 2]);
    expect([change(50, 46.5), change(10, 10), change(null, 3)]).toEqual(["−3.5", "0", ""]);
    expect(daysBefore("2026-11-06", 14)).toBe("2026-10-23");
  });
});

describe("weights", () => {
  it("count each key result by its weight, as the server does", () => {
    expect(weighted([[100, 1], [0, 1]])).toBe(50);
    expect(weighted([[100, 3], [0, 1]])).toBe(75);
    expect(weighted([[100, 0], [20, 1]])).toBe(20);
    expect(weighted([[80, 0]])).toBe(0);
    expect(weighted([])).toBeNull();
  });

  it("reads every page of check-ins, not the first 500", async () => {
    const { every } = await import("./Review");
    const pages: Record<string, { rows: Row[]; next_cursor?: string | null }> = {
      "": { rows: [{ id: "a" }, { id: "b" }], next_cursor: "c2" },
      c2: { rows: [{ id: "c" }] },
    };
    const asked: Record<string, unknown>[] = [];
    const got = await every("CheckIn", [], {}, async (_e, q) => {
      asked.push(q);
      return pages[String(q.cursor ?? "")];
    });
    expect(got.map((r) => r.id)).toEqual(["a", "b", "c"]);
    expect(asked.map((q) => [q.cursor, q.sort, q.limit])).toEqual([
      [undefined, "created_at desc", 500],
      ["c2", "created_at desc", 500],
    ]);
  });
});
