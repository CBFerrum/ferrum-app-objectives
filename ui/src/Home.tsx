// The active objectives as a tree, each with its progress, and its key results with where they
// stand: measured from the records, or due a check-in. An objective or key result opens its record.

import { useQuery } from "@tanstack/react-query";
import { Target } from "lucide-react";
import { api, docPath, todayIso, t, isolate } from "@ferrum/client";
import { Badge, ButtonLink, HomeSkeleton, LoadFailed, PageHead, Queue, QueueGrid, QueueItem, StatePanel, useShellTitle } from "@ferrum/kit";
import { Progress } from "@ferrum/kit/primitives";

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

/** A progress figure as a whole number, end-aligned and tabular. */
export function Percent({ value }: { value: unknown }) {
  return <span className="min-w-10 text-end text-sm font-medium tabular-nums">{isolate(String(value ?? 0))}%</span>;
}

/** A slim read-only bar for a progress figure (0 to 100). */
export function Bar({ value, label }: { value: unknown; label: string }) {
  const n = Math.max(0, Math.min(100, Number(value ?? 0) || 0));
  return <Progress value={n} aria-valuenow={Math.round(n)} aria-label={label} className="h-1.5 max-w-64" />;
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
  const head = (
    <PageHead
      title={t("Objectives")}
      actions={
        <>
          <ButtonLink to="/review">{t("Review")}</ButtonLink>
          <ButtonLink to="/e/Objective/new" variant="primary" icon="plus">
            {t("New objective")}
          </ButtonLink>
        </>
      }
    />
  );
  if (q.isPending) return <>{head}<HomeSkeleton /></>;
  if (q.isError) return <>{head}<LoadFailed error={q.error} onRetry={() => void q.refetch()} /></>;
  // The workspace's day, as its people read dates.
  const today = todayIso();
  if (q.data.objectives.length === 0)
    return (
      <>
        {head}
        <StatePanel kind="empty" icon={Target} title={t("No objectives yet")} action={<ButtonLink to="/e/Objective/new" variant="primary" icon="plus">{t("New objective")}</ButtonLink>}>
          {t("Start with the one the others serve.")}
        </StatePanel>
      </>
    );
  return (
    <>
      {head}
      <QueueGrid className="ob-tree">
        {tree(q.data.objectives).map(({ row: o, depth }) => {
          const results = q.data.results.filter((r) => r.objective === o.id);
          return (
            <div key={o.id} style={{ marginInlineStart: depth * 24 }}>
              <Queue title={isolate(String(o.title))} count={results.length}>
                <QueueItem to={`/e/Objective/${o.id}`} title={t("Progress")} meta={<Bar value={o.progress} label={t("Progress")} />} trailing={<Percent value={o.progress} />} />
                {results.map((r) => (
                  <QueueItem
                    key={r.id}
                    to={`/e/KeyResult/${r.id}`}
                    title={isolate(String(r.title))}
                    meta={
                      <span className="flex flex-wrap items-center gap-x-2 gap-y-1 whitespace-normal">
                        <span>{t("{value} of {value2}", { value: String(r.current_value ?? ""), value2: String(r.target_value ?? "") })}</span>
                        {r.confidence ? <Badge tone={TONE[String(r.confidence)] ?? "amber"}>{isolate(String(r.confidence))}</Badge> : null}
                        {r.metric ? <Badge tone="blue">{t("Measured by a certified metric")}</Badge> : r.measure ? <Badge tone="blue">{t("Measured from the records")}</Badge> : null}
                        {due(r, today) ? <Badge tone="amber">{t("Check-in due")}</Badge> : null}
                      </span>
                    }
                    trailing={<Percent value={r.progress} />}
                  />
                ))}
              </Queue>
            </div>
          );
        })}
      </QueueGrid>
    </>
  );
}
