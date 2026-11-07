// The review meeting's page: each active objective, those at risk first, with how far its key
// results have come since the day reviewed from (two weeks back unless chosen), where each stands
// now, its confidence and its last check-in. Each opens its record.

import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router";
import { api, docPath, formatMoment, todayIso, t } from "@ferrum/client";
import { Badge, ErrorState, PageLoading, useShellTitle } from "@ferrum/kit";
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
  useShellTitle("Review");
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
  if (q.isPending) return <PageLoading />;
  if (q.isError) return <ErrorState error={q.error} onRetry={() => void q.refetch()} />;
  const reviewed = review(q.data.objectives, q.data.results, q.data.checkIns, since);
  return (
    <div className="np-page">
      <header className="row" style={{ justifyContent: "space-between", flexWrap: "wrap", gap: 8 }}>
        <h2>{t("Review")}</h2>
        <label className="row" style={{ gap: 8 }}>
          <span>{t("Since")}</span>
          <input type="date" className="input" value={since} max={today} onChange={(e) => e.target.value && setSince(e.target.value)} />
        </label>
      </header>
      {reviewed.length === 0 && <p className="faint">{t("No active objectives to review.")}</p>}
      {reviewed.map(({ objective: o, then, now, results, risk }) => (
        <section key={o.id} className="card card-pad stack" aria-label={String(o.title)}>
          <header className="row" style={{ justifyContent: "space-between", flexWrap: "wrap", gap: 8 }}>
            <h3>
              <Link to={`/e/Objective/${o.id}`}>{String(o.title)}</Link> <span className="faint">· {String(o.owner ?? "")}</span>
            </h3>
            <span className="row" style={{ gap: 8 }}>
              {risk === 2 ? <Badge tone="red">{t("Off track")}</Badge> : risk === 1 ? <Badge tone="amber">{t("At risk")}</Badge> : null}
              <strong>{String(o.progress ?? 0)}%</strong>
              {then !== null && now !== null ? (
                <span className="faint">
                  {t("key results {then}% → {now}% (", { then, now })}{change(then, now)})
                </span>
              ) : null}
            </span>
          </header>
          {results.length === 0 ? (
            <p className="faint">{t("No key results of its own: its progress is the objectives aligned to it.")}</p>
          ) : (
            <div className="table-wrap">
              <table className="table">
                <caption className="sr-only">{t("The key results of")} {String(o.title)}</caption>
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
                        <Link to={`/e/KeyResult/${r.kr.id}`}>{String(r.kr.title)}</Link>
                        <span className="faint">
                          {" "}
                          · {String(r.kr.current_value ?? "")} of {String(r.kr.target_value ?? "")}
                        </span>
                      </td>
                      <td className="num">{r.then}%</td>
                      <td className="num">{r.now}%</td>
                      <td className="num">{change(r.then, r.now)}</td>
                      <td>{r.confidence ? <Badge tone={TONE[r.confidence] ?? "amber"}>{r.confidence}</Badge> : null}</td>
                      <td>{day(r.kr.last_check_in)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </section>
      ))}
    </div>
  );
}
