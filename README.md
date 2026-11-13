<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/banner-dark.svg">
  <img alt="Ferrum Objectives: goals and key results, measured from your own records" src="docs/assets/banner-light.svg" width="100%">
</picture>

<br>

[![Version](https://img.shields.io/badge/version-0.8.0-15803d?style=flat-square)](CHANGELOG.md)
[![Licence: AGPL-3.0](https://img.shields.io/badge/licence-AGPL--3.0-blue?style=flat-square)](LICENSE)
[![Ferrum platform](https://img.shields.io/badge/runs%20on-Ferrum-2f5bea?style=flat-square)](https://github.com/CBFerrum/Ferrum)
![Rust](https://img.shields.io/badge/Rust-server-b7410e?style=flat-square&logo=rust)
![TypeScript](https://img.shields.io/badge/React-TypeScript-3178c6?style=flat-square&logo=typescript&logoColor=white)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-data-336791?style=flat-square&logo=postgresql&logoColor=white)
![Tests](https://img.shields.io/badge/tests-6%20Rust%20%C2%B7%208%20UI-16a34a?style=flat-square)

### Set objectives, align them, and let the key results measure themselves from the records you already keep.

Objectives is the goals app for Ferrum: an objective tree for a company, its teams and its people, key results measured by check-ins or straight from your records and certified metrics, reminders, a review meeting page and a score when a cycle closes. It is for companies that run OKRs and want them next to their projects and their numbers, and it replaces a separate goals subscription.

[💡 Why](#-why-objectives) · [📸 Screenshots](#-screenshots) · [✨ Features](#-features) · [📊 Compare](#-how-it-compares) · [📍 Roadmap](#-roadmap) · [🚀 Install](#-install) · [🔧 Develop](#-develop)

</div>

## 💡 Why Objectives

- 🔗 **One data set with the rest of Ferrum.** A key result can count deals won, tickets closed or invoices raised from the records themselves, or read a certified metric from Ferrum Metrics. No export, no sync, no stale number typed by hand.
- 🔐 **Measured with the owner's rights.** A key result reads the records as its owner. Only someone who can read what it counts can own it.
- 🧭 **Alignment that cannot loop.** An objective aligns to one above it, within its period and a higher level. The platform refuses a parent that would make a loop.
- 🗓️ **Cycles that close for real.** Close a cycle and every objective in it closes with its score, graded on the scale your workspace chose. Nothing in a closed cycle changes afterwards.
- 🔒 **Private where it should be.** An objective can be private to its owner and the people listed on it, together with its key results and check-ins.
- 🚀 **The work behind the goal.** Key results name the projects that move them, and objectives name their portfolio, from Ferrum Projects.
- 🌍 **Arabic and Hindi built in**, right-to-left pages, and your workspace's own interface words.
- 🔓 **Open source and self-hosted.** AGPL-3.0, no per-seat price, your data in your PostgreSQL.

## 🎯 At a glance

| | |
|---|---|
| **Version** | 0.8.0 (package `objectives` 0.2.0; connector `objectives-metrics` 0.1.0) |
| **Kind** | Its own interface at `/objectives/` (the objective tree and the review page) with Ferrum's records screens for everything else |
| **What it adds** | 7 entities (Objective, Key Result, Check-in, Cycle, Objectives Settings and two small tables), 2 roles, 1 module, 2 menu pages, 1 notification kind, 1 daily upkeep job; 1 more entity field from the Metrics connector; no reports, no print formats |
| **Depends on** | Ferrum platform `>=0.3.2, <0.4`; Ferrum Projects `>=0.6` (installing Objectives installs it) |
| **Works with** | Projects (portfolios and projects), Metrics (certified metrics, optional connector), Home's create menu and notifications |
| **Languages** | English, Arabic and Hindi (25 interface words and 100 names in each), right-to-left |
| **Tests** | 6 Rust tests (against a real PostgreSQL), 8 interface tests |
| **Licence** | AGPL-3.0-only |

## 📸 Screenshots

<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/objectives-home-dark.webp"><img alt="Objectives — home" src="docs/assets/screens/objectives-home-light.webp" width="100%"></picture>

<p align="center"><b>🏠 The objective tree</b><br><sub>See every active objective with its progress and each key result's value, confidence and check-in due.</sub></p>

<table>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/objectives-objective-list-dark.webp"><img alt="Objectives — list" src="docs/assets/screens/objectives-objective-list-light.webp"></picture>
<p><b>🎯 Objectives</b><br><sub>Filter objectives by owner, cycle, level and status, as a list, board or calendar.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/objectives-key-result-list-dark.webp"><img alt="Key results — list" src="docs/assets/screens/objectives-key-result-list-light.webp"></picture>
<p><b>📏 Key results</b><br><sub>Compare each key result's current value with its target, progress and confidence.</sub></p>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/objectives-objective-detail-dark.webp"><img alt="Objective — open record" src="docs/assets/screens/objectives-objective-detail-light.webp"></picture>
<p><b>🧾 One objective</b><br><sub>Open an objective to see its key results, parent, cycle, people and history.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/objectives-review-dark.webp"><img alt="Objectives — Review" src="docs/assets/screens/objectives-review-light.webp"></picture>
<p><b>🗣️ The review meeting</b><br><sub>Go through objectives at risk first, with how far each key result moved since a day you choose.</sub></p>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/objectives-key-results-dark.webp"><img alt="Objectives — key results" src="docs/assets/screens/objectives-key-results-light.webp"></picture>
<p><b>📈 Key results against targets</b><br><sub>See each key result against its target, owner and confidence.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/objectives-check-ins-dark.webp"><img alt="Objectives — check-ins" src="docs/assets/screens/objectives-check-ins-light.webp"></picture>
<p><b>✅ Check-ins</b><br><sub>Read the updates on key results: value, confidence, date and who wrote them.</sub></p>
</td>
</tr>
</table>

<table>
<tr>
<td align="center" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/objectives-home-phone-dark.webp"><img alt="Objectives — home" src="docs/assets/screens/objectives-home-phone-light.webp" width="260"></picture>
<p><b>📱 The objective tree (phone)</b><br><sub>Check progress on a phone.</sub></p>
</td>
<td align="center" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/objectives-objective-list-phone-dark.webp"><img alt="Objectives — list" src="docs/assets/screens/objectives-objective-list-phone-light.webp" width="260"></picture>
<p><b>📱 Objectives (phone)</b><br><sub>Browse objectives on a phone.</sub></p>
</td>
</tr>
</table>


## ✨ Features

### 🎯 Objectives

- **Objective** records what is to be achieved: title, owner, period (start and end), parent, cycle, level, weight, notes, status, progress, score and grade. The owner is whoever records it, unless they name someone else.
- **Alignment.** An objective is aligned to one above it (`parent`) and falls within the period of that parent. The tree never loops.
- **Levels: Company, Team, Person.** An objective aligns only to one of a higher level than its own, when both have a level.
- **Status: Draft, Active, Closed.** A draft is being written and takes no check-ins. An active objective does not go back to draft. It closes only by **Close It**, which scores it. A closed objective is kept as it closed.
- **Weights.** An objective counts towards the one it is aligned to by its weight (1 unless set, 0 counts for nothing, never negative).
- **Rolled-up progress.** Progress is the average of its key results' progress and of the objectives aligned to it, each by weight, kept current up the tree as any of them moves.
- **Private objectives.** Tick **Private** and only the owner and the people listed on the objective (`people`) read it, its key results and its check-ins. Everyone else with an Objectives role still reads the open ones.
- **Portfolio.** An objective names the portfolio behind it (from Projects).
- **Discussion and history.** Like every Ferrum record, an objective has a discussion, an activity trail, attachments, tags and followers.

### 🗓️ Cycles

- **Cycle** is a named period (a quarter, a year) with dates and a state: **Planning**, **Active** (Start It) or **Closed** (Close It). A cycle cannot end before it starts.
- **Closing a cycle** closes every objective in it, each scored as Close It scores one, after a confirmation. Nothing in a closed cycle changes and nothing is added to it.
- **A default cycle.** Every new workspace starts with a cycle called Default, active. An objective that names no cycle falls in the default one. A workspace that already had objectives gets the same, holding them.
- **List filters** by cycle, level, owner and status on the objective list.

### 📏 Key results

- **Three kinds.** **Increase** goes from its start up to its target, **Decrease** from its start down to its target, **Milestone** is done or not (1 when done). An increase must aim above where it starts and a decrease below.
- **Progress** is how far along that way the key result is, from 0 to 100, to a tenth.
- **Value and confidence** are the latest check-in's by day. Confidence is **On Track**, **At Risk** or **Off Track**.
- **Weight** counts the key result towards its objective's progress, against the others there.
- **Owner.** Who checks in on it and is reminded: its objective's owner, unless someone is named.
- **Unit** is free text (customers, %, days).
- **The work that moves it.** A key result lists the projects behind it.
- **Closed objectives take no more key results.**

### ✅ Check-ins

- A check-in records a value, a confidence and a note on a day. The day falls within the objective's period and is never after today.
- Its latest check-in by day is the key result's value and confidence, and the day is kept as its last check-in.
- A milestone's value is 1 when done.
- A closed objective, and a draft, take no check-ins. The error says why.
- The author is kept on each check-in.

### 📡 Measured from the records

- **A query of its own.** A key result may name a Studio query of one measure, ungrouped, as a metric is defined (deals won, tickets closed), built with Studio's report builder in the form.
- **With the owner's rights.** The owner must be an active person here and able to read what the query counts.
- **When it is measured.** When it is saved, by itself once a day (a daily upkeep job), and whenever someone checks in to say how confident they are. Each is a check-in marked as measured.
- **Failures are kept.** A day it cannot be measured, the reason stays on the key result (`measure_error`) and the others are still measured.
- **Typed values are refused.** A check-in on such a key result leaves the value empty: it is measured, not typed.

### 🏷️ Measured by a certified metric (with Metrics)

With Ferrum Metrics installed, the connector **Objectives and Metrics** adds a **Certified Metric** field to a key result.

- Its value is the one Metrics gives the key result's owner, as the metric is defined now. A metric composed of others or converted between currencies reads exactly as it does in Metrics.
- A metric Metrics has no value for (a missing exchange rate, say) measures nothing that day, with Metrics' reason kept.
- A metric changed back to draft or deprecated measures nothing until certified again. A check-in is refused and says why.
- A key result names a certified metric or a query of its own, never both.

### 🔔 Reminders

- The owner of a key result checked in by hand is reminded in the app once a check-in is due: every **7 days** unless set otherwise on the key result (**Check In Every**, 0 for never). Reminded again only when the next check-in is due.
- Notification kind **Reminders to check in on your key results**, in-app by default. Each person chooses how it reaches them (in-app, email, and so on) in Ferrum's notification settings.
- Key results measured from the records need no reminders.

### 🏁 Closing, scoring and grading

- **Close It** keeps the objective's progress as its **score**, told on the workspace's scale, and a **grade**: Missed, Progress Made or Achieved. Each of its key results is scored and graded with it, by its own progress.
- **Objectives Settings** (one record per workspace): the scale **0 to 100**, **0.0 to 1.0**, **1 to 5** (evenly from 1 for nothing to 5 for all) or **Letters** (A from 90, B from 80, C from 70, D from 60, F below), and the bands: **Progress Made From** (40 unless set) and **Achieved From** (70 unless set).
- A closed objective takes no more key results or check-ins.

### 🧭 The objective tree and the review meeting

- **Home** shows the active objectives as an indented tree, each with a progress bar and its key results: value of target, confidence, a badge for how it is measured (from the records, or by a certified metric), a badge when a check-in is due, and the progress.
- **Review** (`/objectives/review`) is the page for a review meeting. It lists each active objective with its key results, those at risk first and then the furthest behind. Each shows how far it has come since the day reviewed from (two weeks back unless chosen), where it stands now, its confidence and its last check-in. Progress then is read from the check-ins, weighted as the server weighs it. A key result with no check-in on or before the day starts from its start value.
- Everything opens its record.

### 🔐 Roles and permissions

| Role | What it can do |
|---|---|
| **Objectives Owner** | Sets objectives and key results, checks in, closes them, plans and closes cycles. Reads Projects and Portfolios. Edits private objectives only as their owner. |
| **Objectives Viewer** | Follows objectives, key results, check-ins and cycles, but private ones only when they own them or are listed on them |

With the Metrics connector both roles also read certified metrics.

Private objectives are enforced by the permission rules themselves (path rules on the owner and the listed people), so lists, search, exports and the API all follow them.

### 🔌 For other apps and agents

- Every entity has Ferrum's REST and MCP API under each person's permissions, so an agent can read the tree or check in as the person it acts for.
- Export of objectives, key results and check-ins is a permission of the Objectives Owner role.
- A small assistant note tells Ferrum's assistant what the records mean (states, roll-up, key results) and suggests "How are this quarter's company objectives doing?", "Which key results have no check-in this week?" and "Which objectives are behind?".

### 🌍 Languages and accessibility

- Interface words and the names of models, fields and actions in **Arabic and Hindi**, to be reviewed by native speakers. Pages mirror for right-to-left languages.
- A workspace can override any interface word (Admin, Interface words).
- Loading, empty, no matches, denied, offline and failed states are told apart.

### 🧾 Personal data

An erasure or export request finds the people listed on private objectives by their address. The objective records are kept ("Objective records") because they are the company's.

## 🧭 How it works

```mermaid
flowchart TD
    C["Cycle: Planning, Active, Closed"] --> O["Objective"]
    O -->|"aligned to"| P["Parent objective"]
    O --> K["Key result"]
    K -->|"check-in by hand"| CI["Check-in: value, confidence"]
    K -->|"measured daily"| R["Records, or a certified metric"]
    R --> CI
    CI --> RU["Progress rolled up, by weight"]
    RU --> O
    O -->|"Close It"| S["Score and grade"]
    C -->|"Close It"| S
```

A key result moves by a check-in or is measured on its own. Either way its progress rolls up through the tree by weight. Closing an objective, or its cycle, freezes it with a score and a grade.

```mermaid
flowchart LR
    OB["Objectives"] --> PR["Projects: portfolios, projects"]
    OB --> ME["Metrics: certified metrics"]
    ME --> RE["ERP and other records"]
    OB --> RE
    OB --> HM["Home: notifications, create menu"]
```

Objectives reads other apps' records only as the key result's owner. Projects is required; Metrics is optional and adds the certified-metric field.

## 🧱 Data model

| Entity | What it is | Key fields or states |
|---|---|---|
| **Objective** | What is to be achieved in a period | title, owner, parent, cycle, level, private, people, portfolio, dates, weight, progress, status (Draft, Active, Closed), score, grade |
| **Key Result** | How an objective is measured | kind (Increase, Decrease, Milestone), start, target, current, unit, weight, progress, confidence, owner, measure, check-in every, projects |
| **Check-in** | Where a key result stands on a day | key result, date, value, confidence, note, author, measured |
| **Cycle** | A planned period | name, dates, status (Planning, Active, Closed), default |
| **Objectives Settings** | How a closed score is told | scale, progress made from, achieved from |
| Objective Person, Key Result Project | Small tables | person; project |

| Role | What it can do |
|---|---|
| Objectives Owner | Set, check in, close, plan cycles |
| Objectives Viewer | Follow |

- **Reports:** none of its own; the Review page and the objective list cover the review. Studio can build more.
- **Print formats:** none. Records print through Ferrum's record print.
- **Scheduled jobs:** one daily upkeep: measures key results from the records and sends check-in reminders.
- **Connector package** `objectives_metrics`: adds `metric` (Certified Metric) to Key Result.

## 🔌 Integrations

| With | How | Needs |
|---|---|---|
| **Ferrum Projects** | An objective names its portfolio; a key result names projects | Projects `>=0.6`, installed with it |
| **Ferrum Metrics** | The connector lets a key result be measured by a certified metric | Metrics installed; the owner must read the metric |
| **Records of any app** | A key result's query counts records the owner can read | The owner's rights |
| **Ferrum notifications** | Check-in reminders by kind | Each person's choice of channel |
| **Ferrum assistant and MCP** | Questions and agent access to the records | Permissions of the person |

## 📊 How it compares

Objectives is compared with the OKR and goals products Lattice (Goals), Betterworks, Perdoo, Weekdone and Workday (goals in Performance Enablement). There is no open-source OKR tool of comparable scope that we could confirm.

| Capability | Ferrum Objectives | Lattice | Betterworks | Perdoo | Weekdone | Workday |
|---|:-:|:-:|:-:|:-:|:-:|:-:|
| Objectives with key results | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡¹ |
| Alignment to a parent | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Company, team and person levels | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡¹ |
| Cycles that open and close | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡¹ |
| Close a cycle with scores for everything in it | ✅ | 🟡² | 🟡² | 🟡² | 🟡² | 🟡² |
| Private objectives | ✅ | ✅ | 🟡² | ✅ | 🟡² | 🟡² |
| Draft before publishing | ✅ | 🟡² | 🟡² | 🟡² | 🟡² | 🟡² |
| Weights in the roll-up | ✅ | 🟡² | 🟡² | 🟡² | 🟡² | 🟡² |
| Check-ins with confidence | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡² |
| Scoring and grades on a chosen scale | ✅ | 🟡² | 🟡² | 🟡² | 🟡² | 🟡² |
| Measured from your own records | ✅ | 🟡³ | 🟡³ | 🟡³ | 🟡³ | 🟡³ |
| Measured by a governed metric | ✅ | 🟡³ | 🟡³ | 🟡³ | 🟡³ | 🟡³ |
| Reminders to check in | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡² |
| Review meeting page | ✅ | 🟡² | 🟡² | 🟡² | 🟡² | 🟡² |
| Projects and portfolios behind goals | 🟡⁴ | 🟡² | 🟡² | 🟡² | 🟡² | 🟡² |
| Link goals to performance reviews | ❌ | ✅ | ✅ | 🟡² | 🟡² | ✅ |
| Alignment map or heat map | 🟡⁵ | ✅ | 🟡² | ✅ | 🟡² | 🟡² |
| Several parents or contributing objectives | ❌ | 🟡² | 🟡² | 🟡² | 🟡² | 🟡² |
| Objective templates and an OKR library | ❌ | ✅ | ✅ | ✅ | ✅ | 🟡² |
| Check in from Slack or Teams | ❌ | ✅ | ✅ | ✅ | ✅ | 🟡² |
| Check in by email | ❌ | 🟡² | 🟡² | 🟡² | ✅ | 🟡² |
| Native mobile apps | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| AI help writing goals | ❌ | ✅ | 🟡² | 🟡² | 🟡² | 🟡² |
| Key result types beyond increase, decrease, milestone | ❌ | ✅ | ✅ | ✅ | 🟡² | 🟡² |
| Roll unfinished objectives into the next cycle | ❌ | 🟡² | 🟡² | 🟡² | 🟡² | 🟡² |
| Arabic and Hindi, right to left | ✅ | 🟡² | 🟡² | 🟡² | 🟡² | ✅ |
| Open source | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Self-hosted | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| No per-seat price | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| One data set with projects, finance and HR | ✅⁶ | ➖ | ➖ | ➖ | ➖ | 🟡⁷ |

<sub>¹ Workday's goals sit inside its performance and talent suite and are not organised as OKRs by default. ² Present in some form, or not confirmed; check the product. ³ Through integrations and connectors, not from the company's own records. ⁴ Key results list their projects and objectives their portfolio, as links; progress is not drawn from project progress unless a key result's query measures it. ⁵ A tree of active objectives with progress, and a review page ordered by risk; no map. ⁶ With the Ferrum apps installed. ⁷ Within Workday's own suite.</sub>

### 🏆 Where Objectives stands out

- **Measures itself** from the company's own records, or from a certified metric with Metrics' definitions, with the owner's rights, once a day and at each check-in.
- **Cycles that close with scores**, on four scales, with grades you set.
- **Private objectives** enforced by the permission engine, not by hiding a column.
- **No loops, no future check-ins, no check-ins on closed work**: the rules are in the server.
- **A review page** built for the meeting.
- **Open source, self-hosted, one data set** with Projects, Metrics and the rest of Ferrum, and no per-seat price.

### 🚧 Where the leaders are still ahead

- Chat check-ins (Slack, Teams), email check-ins and native mobile apps.
- Links to performance reviews and one-to-ones.
- Alignment maps, heat maps by team and cycle, and company dashboards.
- Templates and an OKR library; AI help for writing objectives.
- More key result types and rolling unfinished work into the next cycle.

> [!NOTE]
> Leader columns are from public product information as of November 2026 and may change. Cells marked 🟡 for a leader mean the feature exists in some form or we could not confirm the details.

## 📍 Roadmap

### ✅ Recently shipped

- [x] **0.8.0** Cycles that close and freeze their objectives; private objectives; drafts; company, team and person levels.
- [x] **0.7.2** Rebuilt on the tuned design system: better menu icons, a lighter first page, a visible scroll cue, cleaner print.
- [x] **0.7.1** The redesigned interface: shaped loading, told-apart states.
- [x] **0.7.0** A key result measured by a certified metric reads Metrics' own value for its owner, so composed and currency-converted metrics measure it as Metrics defines them.
- [x] **0.6.5** Names in Arabic and Hindi; **0.6.4** the assistant knows its records.
- [x] **0.6.0** Scores on 1 to 5 or as letters; key results scored and graded with their objective; a key result measured from the records built with Studio's builder.

### 🔜 Next up

**Reaching people where they are**
- [ ] **Check in from chat and email.** A reminder that can be answered in Slack, Teams or by reply, so the number gets in.
- [ ] **A native or installable mobile app** for check-ins and approvals.

**Seeing the whole**
- [ ] **An alignment map and a heat map** by team, level, cycle and status, with export.
- [ ] **Filter Home by cycle, level and team**, and show closed cycles.
- [ ] **Roll unfinished objectives into the next cycle** when one closes.

**Connecting to work and people**
- [ ] **Progress from linked projects.** Let a key result take its progress from the projects behind it, not only through a query.
- [ ] **Link goals to reviews and one-to-ones** with Ferrum HR's appraisals.

### 🔭 Later

- [ ] **Several parents or contributing objectives**, for shared goals.
- [ ] **More key result types:** stay within a range, a percent, a currency, binary with sub-milestones.
- [ ] **Objective templates and an OKR library**, with a workspace's own.
- [ ] **Own words for each score band**, instead of the four scales (Missed, Progress Made, Achieved).
- [ ] **Confidence on the objective**, rolled up from its key results.
- [ ] **Dashboards**: scores by team and cycle, check-in discipline, objectives without key results.
- [ ] **Offline check-ins.**
- [ ] **AI help** to write and sharpen an objective, under the person's rights.
- [ ] **Email digest** of a team's objectives before a review.
- [ ] **Reports** in the Reports list (objective scorecard, key result history).

## 🚀 Install

Objectives is a native Ferrum app for platform `>=0.3.2, <0.4`.

1. In Ferrum, open the **App Manager** (or the store) and install **Objectives**. It requires Projects `>=0.6` and installs it.
2. For key results measured by a certified metric, also install **Metrics**. The connector **Objectives and Metrics** is offered when both are installed and ships with Objectives.
3. Give roles: **Objectives Owner** to the people who set goals and check in, **Objectives Viewer** to everyone else. An owner of a key result measured from the records needs rights to read what it counts; one measured by a metric needs Metric Reader.
4. Open **Objectives Settings** and choose the scale and the grade bands. Open **Cycles**: the Default cycle exists and is active; add your quarter or year and start it.
5. Create the company objective first, then the ones that serve it. Add key results, then check in.
6. Each person chooses how they get **Reminders to check in on your key results** in their notification settings.

Self-hosters who build their own Ferrum server image include the `objectives` app with the others.

## 🔧 Develop

```bash
# the database: an ordinary (non-superuser) role, so row-level security applies
createuser ferrum --pwprompt && createdb -O ferrum ferrum_objectives_test
createdb -O ferrum ferrum_objectives_metrics_test

export FERRUM_OBJECTIVES_TEST_DATABASE_URL=postgres://ferrum:ferrum@localhost:5432/ferrum_objectives_test
export FERRUM_OBJECTIVES_METRICS_TEST_DATABASE_URL=postgres://ferrum:ferrum@localhost:5432/ferrum_objectives_metrics_test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The interface lives in `ui/` and builds with Ferrum-Frontend's packages (clone Ferrum-Frontend into `ferrum/web` next to this repository, as CI does):

```bash
cd ui
npm run build       # ui/dist, committed
npm run check
npm run typecheck
npm test
```

- `tests/objectives_it.rs` walks a quarter: a company objective with two aligned under it; loops and long periods refused; key results that increase, decrease and finish; check-ins rolling progress up; a key result measured from projects as it is saved, at a check-in and by the day's upkeep; reminders; weights; objectives closed on the 0.0 to 1.0 scale; who may do what.
- `tests/metrics_it.rs` runs the connector with Metrics: a key result naming a certified metric measured as saved, at a check-in and each day; a draft metric refused; the metric changed back to draft measuring nothing with the reason kept.
- The tests also build on Projects, ERP and Metrics, which Cargo takes from GitHub. To work against local checkouts, patch them in `.cargo/config.toml` (git-ignored).
- With a dev server started with `FERRUM_DEV_UI=1`, `ferrum-app dev --ui` serves the pages from Vite behind the real bar and session, with hot reload.

## 📂 Project layout

```text
app.yaml                        the manifest: id, version, requirements, what it adds to Ferrum
connectors/metrics/app.yaml     the connector with Metrics
packages/objectives/            entities, roles, the default cycle; sql/ upgrades older workspaces
packages/objectives_metrics/    adds the Certified Metric field to Key Result
src/okr.rs                      roll-up, scoring, measuring, reminders, cycles, the rules
ui/src/                         Home (tree), Review (meeting), words in Arabic and Hindi
tests/                          integration tests against a real PostgreSQL
docs/PROVENANCE.md              where the design comes from
```

## 📜 Changelog

Recent releases are listed above; every version is in [CHANGELOG.md](CHANGELOG.md).

## 🤝 Contributing

Issues and pull requests are welcome at [github.com/CBFerrum/ferrum-app-objectives](https://github.com/CBFerrum/ferrum-app-objectives/issues). Please run the checks in Develop first. Objectives is its own design: see [docs/PROVENANCE.md](docs/PROVENANCE.md) before bringing in code from elsewhere.

## 📄 License

AGPL-3.0-only. See [LICENSE](LICENSE).

<div align="center">

<sub>Part of <a href="https://github.com/CBFerrum/Ferrum">Ferrum</a>, the open business platform · Made with 🦀 Rust and 🐘 PostgreSQL</sub>

</div>
