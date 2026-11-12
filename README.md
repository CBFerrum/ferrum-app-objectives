# Objectives

Portfolio and objectives for [Ferrum](https://github.com/CBFerrum/Ferrum):
- objectives aligned to the objectives above them, each in a period;
- key results measured from a start to a target by check-ins, or from the records (with Metrics,
  by a certified metric they name);
- reminders to check in;
- progress rolled up the tree;
- the projects and portfolios behind them, from Projects.

| | |
|---|---|
| 🎯 App | `objectives`, native, in the server |
| 🧩 Package | `objectives` (needs Projects); with Metrics, the connector `objectives-metrics` (package `objectives_metrics`) |
| 🖥️ Interface | its own, at `/objectives/`: the objectives as a tree with their progress and key results |
| 🔐 Roles | Objectives Owner, Objectives Viewer |
| 📜 Licence | AGPL-3.0-only |

## Highlights

- **Alignment.** An objective is aligned to one above it and falls within its period. The tree
  never loops: the platform refuses a parent that would.
- **Key results.** Each goes from its start towards its target: up for an increase, down for a
  decrease, or done for a milestone. Its progress is how far along that way it is, from 0 to 100.
- **Check-ins.** A check-in records a value and a confidence on a day within the objective's
  period, never ahead of today. The latest by day is the key result's value and confidence.
- **Measured from the records.** A key result may name a Studio query of one measure, as a
  metric is defined (deals won, tickets closed). Its owner's rights read it, so only someone who
  can read what it counts can own it. It is measured when saved, by itself once a day, and
  whenever someone checks in to say how confident they are. A day it cannot be measured, the
  reason stays on the key result.
- **By a certified metric.** With Metrics installed, the connector Objectives and Metrics
  (`objectives-metrics`) lets a key result name a certified metric instead of a query of its own.
  Its value is the one Metrics gives the key result's owner, as the metric is defined now: a
  metric composed of others, or converted between currencies, reads exactly as it does in Metrics,
  and one Metrics has no value for (a missing exchange rate, say) measures nothing that day, with
  Metrics' reason kept. A metric
  changed back to draft, or deprecated, measures nothing until it is certified again: the day's
  upkeep keeps why, and a check-in is refused saying so.
- **Reminders.** The owner of a key result checked in by hand is reminded in the app once a
  check-in is due, every 7 days unless set otherwise (0 for never), and again only when the next
  one is.
- **Rolled up, by weight.** An objective's progress is the average of its key results' and of the
  objectives aligned to it, each counted by its weight (1 unless set; 0 counts for nothing), kept
  current up the tree as any of them moves. The review page weighs them the same way.
- **Closing, scored and graded.** A closed objective keeps its progress as its score, and takes no
  more key results or check-ins. Its score is told on the workspace's scale (Objectives Settings:
  0 to 100, 0.0 to 1.0, 1 to 5, or letters from A to F) and graded Missed, Progress Made (from 40
  unless set) or Achieved (from 70 unless set). Each of its key results is scored and graded with
  it, by its own progress.
- **Behind them.** Key results name the projects that move them, and objectives their portfolio.
- **The review meeting.** The Review page (`/objectives/review`) lists each active objective with
  its key results, those at risk first and then the furthest behind. Each shows how far it has
  come since the day reviewed from (two weeks back unless chosen), where it stands now, its
  confidence and its last check-in. Progress then is read from the check-ins as the server reads
  them.

## Develop

```bash
cargo test                       # needs FERRUM_OBJECTIVES_TEST_DATABASE_URL's database
cd ui && npm run build && npm run check
```

`tests/objectives_it.rs` walks a quarter of objectives:
- a company objective with two aligned under it, and loops and long periods refused;
- key results that increase, decrease and finish;
- check-ins rolling progress up the tree;
- a key result measured from projects as it is saved, at a check-in and by the day's upkeep,
  and refused to an owner who cannot read them;
- reminders to check in, once until the next is due;
- an objective moved; weights on a key result and an objective, 0 counting for nothing;
- objectives closed on the 0.0 to 1.0 scale: achieved, progress made and missed, and their key
  results graded with them;
- who may do what.

`tests/metrics_it.rs` (`FERRUM_OBJECTIVES_METRICS_TEST_DATABASE_URL`) runs the connector with
Metrics:
- a key result naming a certified metric, measured as it is saved, at a check-in and each day;
- a draft metric, or a query of its own as well, refused;
- the metric changed back to draft, measuring nothing with the reason kept, then measured again
  as it is defined now once certified anew.

## Not yet

- Scales other than these four, such as a team's own words for each band.

## Changelog

| Version | Highlights |
|---|---|
| **0.6.5** | Its names in Arabic and Hindi. |
| **0.6.4** | The assistant knows its records. |
| **0.6.3** | Rebuilt with Ferrum-Frontend 5b22e7b: its own words in Arabic and Hindi. |
| **0.6.2** | Rebuilt with Ferrum-Frontend 6537d65: once-shown answers kept; English isolated right to left; its own words translatable by a workspace. |
| **0.6.1** | Rebuilt with Ferrum-Frontend d5d98f2: a workspace's own interface words; pictures in attachment fields; a duplicate value said so. |
| **0.6.0** | Scores on 1 to 5 or as letters; key results scored and graded with their objective; a key result measured from the records built with Studio's builder. |
| **0.5.4** | Rebuilt with Ferrum-Frontend 63280d3: right to left, pages mirror; the records pages' words in Arabic and Hindi. |
| **0.7.0** | A key result measured by a certified metric reads Metrics' own value for its owner, so composed and currency-converted metrics measure it as Metrics defines them. |
| **0.7.1** | The redesigned interface: the upgraded design system, shaped loading, told-apart states. |
| **0.7.2** | Rebuilt on the tuned design system: better menu icons, a lighter first page, a visible scroll cue, cleaner print. |
| **0.5.3** | Rebuilt with Ferrum-Frontend a64416a: a box whose default is 1 shows ticked; action dialogs say every refusal and ask once. |
| **0.5.2** | The review over every key result and check-in, not one page of 500. |
| **0.5.1** | Rebuilt with Ferrum-Frontend a76295e: times on a person's own clock when they chose one. |
| **0.5.0** | Weights on key results and objectives in the roll-up and the review; scores told on 0 to 100 or 0.0 to 1.0 and graded Missed, Progress Made or Achieved. |
| **0.4.0** | The review meeting's page: each active objective's change since a day, those at risk first; dates on the workspace's day. |
| **0.3.1** | Rebuilt with Ferrum-Frontend 24c3951: times on the workspace's clock; drafts headed as drafts. |
| **0.3.0** | With Metrics, key results measured by a certified metric they name, as it is defined now (connector `objectives-metrics`). |
| **0.2.2** | Rebuilt with Ferrum-Frontend bb60115: empty numbers read as the server reads them; settings open as their record. |
| **0.2.1** | Dates are the workspace's, not UTC's. |
| **0.2.0** | Key results measured from the records with their owner's rights: when saved, each day and at each check-in. Reminders to check in. |
| **0.1.4** | Rebuilt with Ferrum-Frontend 00dea07: a workflow state's kept fields are read-only, its needed ones required. |
| **0.1.3** | Rebuilt with Ferrum-Frontend daeadda: actions say what they did. |
| **0.1.2** | Rebuilt with Ferrum-Frontend 80ff179: labels name their fields, and linked titles show their titles. |
| **0.1.1** | An objective's owner is whoever records it, unless they name someone. |
| **0.1.0** | Objectives aligned to their parents, key results measured by check-ins, progress rolled up, and the projects and portfolios behind them (FERRUM-ECOSYSTEM-011). |

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
