// The active objectives as a tree, each with its progress, and its key results with where they
// stand: measured from the records, or due a check-in. An objective or key result opens its record.

import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router";
import { api, docPath, todayIso, t, isolate } from "@ferrum/client";
import { Badge, ErrorState, PageLoading, useShellTitle } from "@ferrum/kit";

export interface Row {
  id: string;
  [k: string]: unknown;
}

const rows = (entity: string, filters: unknown[], extra: Record<string, string | number> = {}) =>
  api.get<{ rows: Row[] }>(docPath(entity), { filters: JSON.stringify(filters), limit: 500, ...extra }).then((r) => r.rows);

/** Objectives as a tree: the roots (no parent among them), each with its children, depth first. */
export function tree(list: Row[]): { row: Row; depth: number }[] {
  const ids = new Set(list.map((o) => o.id));
  const kids = new Map<string, Row[]>();
  for (const o of list) {
    const p = typeof o.parent === "string" && ids.has(o.parent) ? o.parent : "";
    kids.set(p, [...(kids.get(p) ?? []), o]);
  }
  const out: { row: Row; depth: number }[] = [];
  const walk = (parent: string, depth: number) => {
    for (const o of kids.get(parent) ?? []) {
      out.push({ row: o, depth });
      walk(o.id, depth + 1);
    }
  };
  walk("", 0);
  return out;
}

/** Whether a key result checked in by hand is due a check-in on `today` (YYYY-MM-DD), as its owner is reminded. */
export function due(r: Row, today: string): boolean {
  const every = Number(r.check_in_every ?? 7);
  const last = typeof r.last_check_in === "string" ? r.last_check_in.slice(0, 10) : "";
  if (r.measure || r.metric || !(every > 0) || !last) return false;
  return (Date.parse(today) - Date.parse(last)) / 86_400_000 >= every;
}

const TONE: Record<string, "green" | "amber" | "red"> = { "On Track": "green", "At Risk": "amber", "Off Track": "red" };

export function Home() {
  useShellTitle(t("Objectives"));
  const q = useQuery({
    queryKey: ["objectives", "tree"],
    queryFn: async () => {
      const [objectives, results] = await Promise.all([rows("Objective", [["status", "eq", "Active"]], { sort: "title asc" }), rows("KeyResult", [])]);
      return { objectives, results };
    },
  });
  if (q.isPending) return <PageLoading />;
  if (q.isError) return <ErrorState error={q.error} onRetry={() => void q.refetch()} />;
  // The workspace's day, as its people read dates.
  const today = todayIso();
  return (
    <div className="np-page">
      <header className="row" style={{ justifyContent: "space-between" }}>
        <h2>{t("Objectives")}</h2>
        <span className="row" style={{ gap: 12 }}>
          <Link to="/review">{t("Review")}</Link>
          <Link to="/e/Objective/new">{t("New objective")}</Link>
        </span>
      </header>
      <section className="card card-pad stack" aria-label={t("The tree")}>
        {q.data.objectives.length === 0 && <p className="faint">{t("No objectives yet: start with the one the others serve.")}</p>}
        <ul className="stack">
          {tree(q.data.objectives).map(({ row: o, depth }) => (
            <li key={o.id} style={{ marginLeft: depth * 24 }} className="stack">
              <span>
                <Link to={`/e/Objective/${o.id}`}>{isolate(String(o.title))}</Link> · {isolate(String(o.progress ?? 0))}%
              </span>
              <ul className="stack">
                {q.data.results
                  .filter((r) => r.objective === o.id)
                  .map((r) => (
                    <li key={r.id} className="faint">
                      <Link to={`/e/KeyResult/${r.id}`}>{isolate(String(r.title))}</Link> {t("· {value} of {value2} · {value3}%", { value: String(r.current_value ?? ""), value2: String(r.target_value ?? ""), value3: String(r.progress ?? 0) })}{isolate(" ")}
                      {r.confidence ? <Badge tone={TONE[String(r.confidence)] ?? "amber"}>{isolate(String(r.confidence))}</Badge> : null}{isolate(" ")}
                      {r.metric ? <Badge tone="blue">{t("Measured by a certified metric")}</Badge> : r.measure ? <Badge tone="blue">{t("Measured from the records")}</Badge> : null}
                      {due(r, today) ? <Badge tone="amber">{t("Check-in due")}</Badge> : null}
                    </li>
                  ))}
              </ul>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
