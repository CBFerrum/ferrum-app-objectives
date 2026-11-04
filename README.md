# Objectives

Portfolio and objectives for [Ferrum](https://github.com/CBFerrum/Ferrum):
- objectives aligned to the objectives above them, each in a period;
- key results measured from a start to a target by check-ins;
- progress rolled up the tree;
- the projects and portfolios behind them, from Projects.

| | |
|---|---|
| 🎯 App | `objectives`, native, in the server |
| 🧩 Package | `objectives` (needs Projects) |
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
- **Rolled up.** An objective's progress is the average of its key results' and of the
  objectives aligned to it, kept current up the tree as any of them moves.
- **Closing.** A closed objective keeps its progress as its score, and takes no more key results
  or check-ins.
- **Behind them.** Key results name the projects that move them, and objectives their portfolio.

## Develop

```bash
cargo test                       # needs FERRUM_OBJECTIVES_TEST_DATABASE_URL's database
cd ui && npm run build && npm run check
```

`tests/objectives_it.rs` walks a quarter of objectives:
- a company objective with two aligned under it, and loops and long periods refused;
- key results that increase, decrease and finish;
- check-ins rolling progress up the tree;
- an objective moved, and one closed;
- who may do what.

## Not yet

- Weights on key results, and scoring scales other than 0 to 100.
- Key results measured from other records (deals won, tickets closed) without check-ins.
- Reminders to check in, and a review meeting's view.

## Changelog

| Version | Highlights |
|---|---|
| **0.1.0** | Objectives aligned to their parents, key results measured by check-ins, progress rolled up, and the projects and portfolios behind them (FERRUM-ECOSYSTEM-011). |

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
