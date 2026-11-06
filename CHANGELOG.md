# Changelog

## 0.3.0

Key results measured by a certified metric (FERRUM-ECOSYSTEM-011, -010).

- **Objectives and Metrics** (`objectives-metrics`, package `objectives_metrics`): a connector that
  ships with Objectives and is offered where Metrics is installed. A key result may name a certified
  metric (Certified Metric) instead of a query of its own, never both. It is measured as the metric
  is defined now, with the key result's owner's rights: when saved, at each check-in, and by the
  day's upkeep.
- A metric that is no longer certified (changed back to draft, or deprecated) measures nothing.
  The day's upkeep keeps why on the key result, a check-in is refused saying so, and naming one is
  refused.
- Objectives' owners and viewers read certified metrics, to name them and see them named.
- The page marks key results measured by a certified metric, and never asks them for a check-in.

## 0.2.2

Rebuilt with Ferrum-Frontend bb60115: a record page reads an empty number or check as the server does, so an action whose condition needs one shows, and a workspace's settings open as their one record.

## 0.2.1

The workspace's day, not UTC's: a check-in may be dated up to the workspace's today, and the day's
measure and reminders count days where the workspace is. Its test runs in a workspace whose date is
not UTC's.

## 0.2.0

Key results measured from the records, and reminders to check in (FERRUM-ECOSYSTEM-011).

- A key result may name a Studio query of one measure, ungrouped, as a metric is defined. It is
  read with the key result's owner's rights (its objective's owner, unless someone is named), so
  saving one is refused when its owner cannot read what it counts. It is measured as it is saved,
  by itself once a day while its objective is active and in its period, and whenever someone
  checks in. Such check-ins have their value measured, not typed, and keep the confidence and note
  given. When the day's measure fails, the reason stays on the key result and the others are
  still measured.
- The owner of a key result checked in by hand is reminded in the app (`objectives.check_in`)
  once a check-in is due: every 7 days, or as set (0 for never). They are reminded again only
  when the next one is due.
- Key results show their owner and the day of their latest check-in. The tree marks those
  measured from the records and those due a check-in.
- Rebuilt with Ferrum-Frontend 4a3681c.

## 0.1.4

Rebuilt with Ferrum-Frontend 00dea07: in a workflow state that names what may change, the other fields show read-only, and the fields the state needs show required.

## 0.1.3

Rebuilt with Ferrum-Frontend daeadda: after an action runs, a record page shows what the action said it did rather than only that it is done.

## 0.1.2

Rebuilt with Ferrum-Frontend 80ff179: a field's label names its control for assistive technology, and a record whose title is a link shows that record's title, not its id.

## 0.1.1

From a browser walk: an objective's owner is whoever records it, unless they name someone (it was
left empty and refused).

## 0.1.0

Portfolio and objectives (FERRUM-ECOSYSTEM-011).

- Objectives aligned to their parents, within their periods.
- Key results that increase, decrease or are done, moved by their latest check-in.
- Progress as the average of an objective's key results and the objectives under it, rolled up.
- Closing with a score; closed objectives take nothing more.
- Key results linked to Projects' projects, and objectives to portfolios.
- Its own interface at `/objectives/`: the tree.
