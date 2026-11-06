// What a review meeting goes through: each active objective with how far its key results have come
// since a day (the last review), where each stands now and how confident its owner is, those at
// risk first. Progress is worked out as the server works it out (src/okr.rs `progress`).

export interface Row {
  id: string;
  [k: string]: unknown;
}

/** How far a key result has come from its start towards its target, from 0 to 100, to a tenth. */
export function progressOf(kind: string, start: number, target: number, current: number): number {
  let p: number;
  if (kind === "Milestone") p = current >= 1 ? 100 : 0;
  else if (target === start) p = 0;
  else p = ((current - start) / (target - start)) * 100;
  return Math.round(Math.min(100, Math.max(0, p)) * 10) / 10;
}

/** A key result's value on `day` (YYYY-MM-DD): its latest check-in's on or before it, by day and
 *  then as entered; its start before any. */
export function valueOn(kr: Row, checkIns: Row[], day: string): number {
  const mine = checkIns.filter((c) => c.key_result === kr.id && typeof c.date === "string" && c.date.slice(0, 10) <= day && c.value !== null && c.value !== undefined && c.value !== "");
  mine.sort((a, b) => String(a.date).localeCompare(String(b.date)) || String(a.created_at ?? "").localeCompare(String(b.created_at ?? "")));
  const last = mine[mine.length - 1];
  return last ? Number(last.value) : kr.kind === "Milestone" ? 0 : Number(kr.start_value ?? 0);
}

export interface ReviewedResult {
  kr: Row;
  then: number;
  now: number;
  confidence: string;
}

export interface Reviewed {
  objective: Row;
  /** Its key results' progress on average, on the day reviewed from and now; none without any. */
  then: number | null;
  now: number | null;
  results: ReviewedResult[];
  /** 2 when a key result is Off Track, 1 At Risk, else 0. */
  risk: number;
}

const RISK: Record<string, number> = { "Off Track": 2, "At Risk": 1 };

const average = (xs: number[]): number | null => (xs.length ? Math.round((xs.reduce((a, b) => a + b, 0) / xs.length) * 10) / 10 : null);

/** The review of `objectives` since `since`: those at risk first, then the furthest behind. */
export function review(objectives: Row[], results: Row[], checkIns: Row[], since: string): Reviewed[] {
  return objectives
    .map((objective) => {
      const rs: ReviewedResult[] = results
        .filter((r) => r.objective === objective.id)
        .map((kr) => ({
          kr,
          then: progressOf(String(kr.kind ?? "Increase"), Number(kr.start_value ?? 0), Number(kr.target_value ?? 0), valueOn(kr, checkIns, since)),
          now: Number(kr.progress ?? 0),
          confidence: String(kr.confidence ?? ""),
        }));
      return {
        objective,
        then: average(rs.map((r) => r.then)),
        now: average(rs.map((r) => r.now)),
        results: rs,
        risk: Math.max(0, ...rs.map((r) => RISK[r.confidence] ?? 0)),
      };
    })
    .sort((a, b) => b.risk - a.risk || (a.now ?? Number(a.objective.progress ?? 0)) - (b.now ?? Number(b.objective.progress ?? 0)));
}

/** A change in points, signed: "+15", "−3.5", "0". */
export function change(then: number | null, now: number | null): string {
  if (then === null || now === null) return "";
  const d = Math.round((now - then) * 10) / 10;
  return d > 0 ? `+${d}` : d < 0 ? `−${-d}` : "0";
}

/** The day `days` before `day` (YYYY-MM-DD). */
export function daysBefore(day: string, days: number): string {
  return new Date(Date.parse(`${day}T00:00:00Z`) - days * 86_400_000).toISOString().slice(0, 10);
}
