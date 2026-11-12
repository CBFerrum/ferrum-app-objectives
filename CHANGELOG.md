# Changelog

## 0.7.2

Rebuilt with Ferrum-Frontend 45dad7f, the design system as tuned after the landed build was reviewed. The left menu's icons are chosen so that none repeats on more than three rows and each fits its row; their drawings load beside the first page instead of inside it, so the page starts lighter; a row that scrolls sideways fades at the edge where there is more, in every browser and with reduced motion; a printed record leaves out empty fields and keeps its totals with the table; menus and popovers sit inside the page's landmarks for assistive technology; a selection bar folds what does not fit into More on a narrow screen. Nothing else changes.

## 0.7.1

The interface follows the upgraded design system, rebuilt with Ferrum-Frontend 0a47157. Text uses the kit's type roles (nothing under 12px) and controls its height tokens; loading shows skeletons shaped like what is coming; empty, no matches, denied, offline and failed are told apart; status colours come from stored state ids. The interface check now runs the UI lint against an empty baseline. The home is rebuilt on the kit's work-home pattern: one page title, the main action as a button, queues with their counts, status at its natural width; its other own pages follow the same anatomy.

## 0.7.0

A key result measured by a certified metric (connector `objectives-metrics`) now reads the metric's value from
Metrics, for its owner and with their rights, instead of running the metric's query here. A metric composed of
others, which has no query, can now measure a key result, and one converted between currencies (Metrics 0.6.0)
measures it in its own currency, never as a sum of the amounts as they are. When Metrics has no value, a missing
exchange rate among other reasons, nothing is measured that day and Metrics' reason stays on the key result.
Whoever sets the metric must be able to read its value. tests/metrics_it.rs measures key results by a
converted metric and a composed one.

## 0.6.5

Rebuilt with Ferrum-Frontend c59c9a2: the names of its models, fields and manifest in Arabic and Hindi (100 each in ui/src/locales/names, which the platform serves to every page of the workspace, so a record's labels, the launcher and the bar read in the person's language). Translated by the model, to be reviewed by native speakers.

## 0.6.4

The assistant knows what its records mean in its pages (contributes.shell.assistant): their states, what counts and where figures come from, with questions it suggests asking there.

## 0.6.3

Rebuilt with Ferrum-Frontend 5b22e7b: its own words carry built-in Arabic and Hindi (23 words each, ui/src/locales, translated by the model and to be reviewed by native speakers), fetched for the page's language only. The labels it keeps in objects, its dialogs, its page titles and the words it keeps in variables go through t() as well, and the names and sizes it shows as they are keep their order right to left. Its pages are drawn once their words are in, and its entry carries only the parts of the kit it uses.

## 0.6.2

Rebuilt with Ferrum-Frontend 6537d65: an action's link, code or token shown once is kept in a dialog to copy, and on a page that reads right to left, English left untranslated keeps its own order. Its own words go through the kit's t() (24 places), so a workspace's own words for its language reach them, and Admin, Interface words lists them under it (dist/words.json).

## 0.6.1

Rebuilt with Ferrum-Frontend d5d98f2: a workspace's own words for its interface (Admin, Interface words) reach its pages as they load; an attachment field that holds a picture shows it; and a value another record already has is said so, where the form said the record had been changed by someone else.

## 0.6.0

- **Two more scales.** A closed objective's score is told on 1 to 5, running evenly from 1 (nothing)
  to 5 (all), or as a letter: A from 90, B from 80, C from 70, D from 60, F below (Objectives
  Settings). 0 to 100 and 0.0 to 1.0 stay. The grades (Missed, Progress Made, Achieved) keep the
  workspace's bands whatever the scale.
- **Key results graded.** Closing an objective scores and grades each of its key results, by its
  own progress, on the same scale (Score, Scored, Grade on the key result).
- **A query built, not typed.** A key result measured from the records takes its Studio query on
  Studio's builder in its form (`widget: studio_query`, which needs Ferrum with field widgets).

## 0.5.4

Rebuilt with Ferrum-Frontend 63280d3: right to left, its pages mirror as they read (icons that point along the line, margins and alignment by the line's start and end, arrow keys along it), and the records pages' words are in Arabic and Hindi.

## 0.5.3

Rebuilt with Ferrum-Frontend a64416a: a new record's box whose default is 1 shows ticked, as it is saved. An action's dialog says every refusal, the errors on what the action made (another record's rows) among them, and an action with parameters and a confirmation asks once.

## 0.5.2

The review page read key results and check-ins as one list: it asked for 2,000, the server pages at 500, so with more check-ins than that the progress since the day reviewed from was worked out from some of them. It now reads every page.

## 0.5.1

Rebuilt with Ferrum-Frontend a76295e: its pages show times on a person's own clock when they chose one on their Account page, while today and due dates stay on the workspace's day.

## 0.5.0

Weights and grades (FERRUM-ECOSYSTEM-011).

- **Weight** on a key result and on an objective: how much it counts towards the objective above
  it, against the others there (1 unless set; 0 counts for nothing; never below 0). Progress rolls
  up by these weights, and the review page weighs key results the same way.
- **Objectives Settings:** the scale a closed objective's score is told on (0 to 100, or 0.0 to
  1.0), and the bands it is graded by: Progress Made from 40 and Achieved from 70 unless set.
- **Closing** keeps the score as before, and now its score on the scale ("0.65 of 1.0") and its
  grade (Missed, Progress Made or Achieved).

## 0.4.0

The review meeting's page (FERRUM-ECOSYSTEM-011).

- **Review** (`/objectives/review`, linked from the home page): each active objective, those with
  a key result Off Track or At Risk first, then the furthest behind. Each shows:
  - its progress, and its key results' progress on the day reviewed from (two weeks back unless
    chosen) against now, with the change;
  - each key result's value against its target, then and now, its confidence, and its last
    check-in on the workspace's day.
- Progress on a past day is read from the check-ins the way the server reads it: the latest by day
  on or before it, else the start.
- The home page's check-ins due go by the workspace's day.

## 0.3.1

Rebuilt with Ferrum-Frontend 24c3951: its pages show times on the workspace's clock, naming the zone when it is not the browser's, a draft numbered on submit is headed as a draft, and a record of an app that is not installed says so.

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
