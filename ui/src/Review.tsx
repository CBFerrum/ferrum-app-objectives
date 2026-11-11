// The review meeting's page: each active objective, those at risk first, with how far its key
// results have come since the day reviewed from (two weeks back unless chosen), where each stands
// now, its confidence and its last check-in. Each opens its record.

import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router";
import { api, docPath, formatMoment, todayIso, t, isolate } from "@ferrum/client";
import { Target } from "lucide-react";
import { Badge, HomeSkeleton, LoadFailed, PageHead, StatePanel, useShellTitle } from "@ferrum/kit";
import { Bar, Percent } from "./Home";
import { change, daysBefore, review, type Row } from "./meeting";

type Get = (entity: string, query: Record<string, string | number | undefined>) => Promise<{ rows: Row[]; next_cursor?: string | null }>;
const get: Get = (entity, query) => api.get(docPath(entity), query);

/** Every matching row, page after page by cursor: the server pages at 500, and cursors need the
 * creation order. */
export async function every(entity: string, filters: unknown[], extra: Record<string, string> = {}, fetchPage: Get = get): Promise<Row[]> {
  const out: Row[] = [];
  let cursor: string | undefined;
  do {
    const p = await fetchPage(entity, { filters: JSON.stringify(filters), limit: 500, sort: "created_at desc", cursor, ...extra });
    out.push(...p.rows);
    cursor = p.next_cursor ?? undefined;
  } while (cursor);
  return out;
}

const TONE: Record<string, "green" | "amber" | "red"> = { "On Track": "green", "At Risk": "amber", "Off Track": "red" };

const day = (at: unknown) => (typeof at === "string" && at ? formatMoment(at, { day: "numeric", month: "short", year: "numeric" }) : "never");

export function Review() {
  useShellTitle(t("Review"));
  const today = todayIso();
  const [since, setSince] = useState(() => daysBefore(today, 14));
  const q = useQuery({
    queryKey: ["objectives", "review"],
    queryFn: async () => {
      const [objectives, results, checkIns] = await Promise.all([
        every("Objective", [["status", "eq", "Active"]]).then((list) => list.sort((a, b) => String(a.title).localeCompare(String(b.title)))),
        every("KeyResult", []),
        every("CheckIn", [["date", "<=", today]], { fields: "id,key_result,date,value,created_at" }),
      ]);
      return { objectives, results, checkIns };
    },
  });
  const head = (
    <PageHead
      title={t("Review")}
      back={{ to: "/", label: t("Objectives") }}
      actions={
        <label className="flex items-center gap-2 text-sm text-muted-foreground">
          <span>{t("Since")}</span>
          <input type="date" className="input" value={since} max={today} onChange={(e) => e.target.value && setSince(e.target.value)} />
        </label>
      }
    />
  );
  if (q.isPending) return <>{head}<HomeSkeleton queues={2} /></>;
  if (q.isError) return <>{head}<LoadFailed error={q.error} onRetry={() => void q.refetch()} /></>;
  const reviewed = review(q.data.objectives, q.data.results, q.data.checkIns, since);
  return (
    <>
      {head}
      {reviewed.length === 0 && <StatePanel kind="empty" icon={Target} title={t("No active objectives to review.")} />}
      <div className="grid grid-cols-[minmax(0,1fr)] gap-4">
      {reviewed.map(({ objective: o, then, now, results, risk }) => (
        <section key={o.id} className="card card-pad stack" aria-label={String(o.title)}>
          <header className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
            <h2 className="text-base font-semibold">
              <Link to={`/e/Objective/${o.id}`}>{isolate(String(o.title))}</Link>
              {o.owner ? <span className="ms-2 text-sm font-normal text-muted-foreground">{isolate(String(o.owner))}</span> : null}
            </h2>
            <span className="flex items-center gap-3">
              {risk === 2 ? <Badge tone="red">{t("Off track")}</Badge> : risk === 1 ? <Badge tone="amber">{t("At risk")}</Badge> : null}
              {then !== null && now !== null ? (
                <span className="text-xs text-muted-foreground tabular-nums">
                  {t("key results {then}% → {now}% (", { then, now })}{isolate(change(then, now))})
                </span>
              ) : null}
              <span className="flex w-40 items-center gap-2">
                <Bar value={o.progress} label={t("Progress")} />
                <Percent value={o.progress} />
              </span>
            </span>
          </header>
          {results.length === 0 ? (
            <p className="text-sm text-muted-foreground">{t("No key results of its own: its progress is the objectives aligned to it.")}</p>
          ) : (
            <div className="table-wrap">
              <table className="table">
                <caption className="sr-only">{t("The key results of")} {isolate(String(o.title))}</caption>
                <thead>
                  <tr>
                    <th scope="col">{t("Key result")}</th>
                    <th scope="col" className="num">
                      {t("Then")}
                    </th>
                    <th scope="col" className="num">
                      {t("Now")}
                    </th>
                    <th scope="col" className="num">
                      {t("Change")}
                    </th>
                    <th scope="col">{t("Confidence")}</th>
                    <th scope="col">{t("Last check-in")}</th>
                  </tr>
                </thead>
                <tbody>
                  {results.map((r) => (
                    <tr key={r.kr.id}>
                      <td>
                        <Link to={`/e/KeyResult/${r.kr.id}`}>{isolate(String(r.kr.title))}</Link>
                        <span className="faint">
                          {isolate(" ")}
                          {t("· {value} of {value2}", { value: String(r.kr.current_value ?? ""), value2: String(r.kr.target_value ?? "") })}
                        </span>
                      </td>
                      <td className="num">{r.then}%</td>
                      <td className="num">{r.now}%</td>
                      <td className="num">{isolate(change(r.then, r.now))}</td>
                      <td>{r.confidence ? <Badge tone={TONE[r.confidence] ?? "amber"}>{isolate(r.confidence)}</Badge> : null}</td>
                      <td>{isolate(day(r.kr.last_check_in))}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </section>
      ))}
      </div>
    </>
  );
}
