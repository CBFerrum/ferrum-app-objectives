// The active objectives as a tree, each with its progress, and its key results with where they
// stand. An objective or key result opens its record.

import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router";
import { api, docPath } from "@ferrum/client";
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

const TONE: Record<string, "green" | "amber" | "red"> = { "On Track": "green", "At Risk": "amber", "Off Track": "red" };

export function Home() {
  useShellTitle("Objectives");
  const q = useQuery({
    queryKey: ["objectives", "tree"],
    queryFn: async () => {
      const [objectives, results] = await Promise.all([rows("Objective", [["status", "eq", "Active"]], { sort: "title asc" }), rows("KeyResult", [])]);
      return { objectives, results };
    },
  });
  if (q.isPending) return <PageLoading />;
  if (q.isError) return <ErrorState error={q.error} onRetry={() => void q.refetch()} />;
  return (
    <div className="np-page">
      <header className="row" style={{ justifyContent: "space-between" }}>
        <h2>Objectives</h2>
        <Link to="/e/Objective/new">New objective</Link>
      </header>
      <section className="card card-pad stack" aria-label="The tree">
        {q.data.objectives.length === 0 && <p className="faint">No objectives yet: start with the one the others serve.</p>}
        <ul className="stack">
          {tree(q.data.objectives).map(({ row: o, depth }) => (
            <li key={o.id} style={{ marginLeft: depth * 24 }} className="stack">
              <span>
                <Link to={`/e/Objective/${o.id}`}>{String(o.title)}</Link> · {String(o.progress ?? 0)}%
              </span>
              <ul className="stack">
                {q.data.results
                  .filter((r) => r.objective === o.id)
                  .map((r) => (
                    <li key={r.id} className="faint">
                      <Link to={`/e/KeyResult/${r.id}`}>{String(r.title)}</Link> · {String(r.current_value ?? "")} of {String(r.target_value ?? "")} · {String(r.progress ?? 0)}%{" "}
                      {r.confidence ? <Badge tone={TONE[String(r.confidence)] ?? "amber"}>{String(r.confidence)}</Badge> : null}
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
