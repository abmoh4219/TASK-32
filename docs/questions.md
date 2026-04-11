# questions.md — ScholarVault Research & Commerce Operations Portal
# Business Logic Questions & Clarifications
# Format: Question → My Understanding/Assumption → Solution Implemented

---

## 1. Category Tree — Strict Tree or DAG (Multiple Parents)?

**Question:** The prompt says "multi-level category tree" but also mentions "cycles" and "orphan referenced nodes." A pure tree cannot have cycles by definition, but the cycle-detection language implies nodes could have multiple parents (making it a directed acyclic graph — DAG). Which structure is intended?

**My Understanding:** The mention of cycle detection and orphan checks strongly implies a DAG where a knowledge point or category node could potentially be linked to multiple parents. The prompt explicitly says to "block changes when it would create cycles" — this only makes sense in a graph structure, not a strict tree.

**Solution Implemented:** DAG structure. `categories` table uses `parent_id` for the primary display hierarchy. Node merge/migration operations run a DFS cycle check before execution. Reference counts include: direct knowledge points, child category nodes, and indirectly linked questions — all shown in the UI before the save is confirmed.

---

## 2. Contribution Share Precision — What Happens with Fractional Percentages?

**Question:** The prompt states contribution shares "must total exactly 100%." If three contributors each receive 33.33%, the floating-point sum is 99.99% — not exactly 100%. How should this be handled?

**My Understanding:** The prompt's use of "exactly 100%" implies the system must enforce strict equality. Using floating-point arithmetic would make this unreliable. Storing shares as integers (whole percentage points, 1–100) avoids this entirely and is the safest approach for an exact-equality constraint.

**Solution Implemented:** `share_percentage` stored as INTEGER in SQLite (whole percentage points). Service-layer validation: `SUM(share_percentage) == 100` exactly. The UI shows a running total bar that turns green at 100 and red if under or over. The outcome cannot be submitted unless the sum is exactly 100.

---

## 3. Duplicate Detection — What Threshold Makes Something a "Duplicate"?

**Question:** The prompt says "duplicates are flagged during entry using title, abstract snippet, and certificate number similarity checks." It does not specify the similarity algorithm or the threshold above which something is considered a duplicate.

**My Understanding:** The prompt uses "flagged" not "blocked" — meaning duplicates are surfaced for human review, not automatically rejected. The threshold needs to be high enough to catch real duplicates without too many false positives. Certificate number matching should be exact (after normalization), while title and abstract use string similarity.

**Solution Implemented:** Using Jaro-Winkler similarity. Title: threshold ≥ 0.85. Abstract snippet (first 200 chars): threshold ≥ 0.80. Certificate number: exact match after normalizing (lowercase, strip spaces and hyphens). Any match triggers the side-by-side compare view. Results include similarity scores so the reviewer can judge.

---

## 4. Side-by-Side Compare View — Which Fields Are Compared?

**Question:** The prompt says a "side-by-side compare view before submission" is shown when duplicates are detected. It does not specify which fields appear in this comparison.

**My Understanding:** The duplicate was detected based on title, abstract snippet, and certificate number — so those fields must appear. But the reviewer also needs to see the outcome type, contributors, and status to make a meaningful decision about whether it is truly a duplicate.

**Solution Implemented:** Side-by-side view shows: type, title, abstract snippet, certificate number, contributors (names + share %), evidence file count, registration date, and current status. Differences are highlighted in amber. The reviewer can choose to proceed with submission or discard as a duplicate.

---

## 5. "Best Offer" at Checkout — How Is "Best" Defined?

**Question:** The prompt says "at checkout the UI automatically applies the best offer." When multiple promotions are eligible (and not mutually exclusive), how is "best" determined? Highest absolute discount? Highest percentage? Highest priority?

**My Understanding:** The prompt says promotions have a "priority" field. Priority is the most explicit ordering mechanism provided. When mutual exclusion groups are resolved (one per group), the remaining eligible promotions are ranked by priority (higher number = higher priority) and the highest-priority valid offer is applied.

**Solution Implemented:** Step 1: filter promotions within effective time window. Step 2: within each mutual exclusion group, keep only the highest-priority promotion. Step 3: from remaining eligible promotions across all groups, apply the one that produces the greatest discount value for the cart. If tie on discount value, the higher-priority promotion wins. All applied promotions and discount amounts are shown per line item.

---

## 6. Mutual Exclusion Groups — Can a Promotion Belong to Multiple Groups?

**Question:** The prompt says promotions have "mutual exclusion groups." It does not clarify whether a promotion can belong to more than one group simultaneously.

**My Understanding:** Mutual exclusion means "only one promotion from this group applies per checkout." Allowing a promotion to belong to multiple groups would create complex conflicts. The simpler and more practical interpretation is one group per promotion — if you want two separate exclusion rules, create two separate promotions.

**Solution Implemented:** Each promotion has one `mutual_exclusion_group` field (a string label, nullable). NULL means the promotion is not in any exclusion group. The exclusion logic: for each distinct group value, only the highest-priority promotion in that group is eligible for application.

---

## 7. Account Lockout Duration — How Long Is the Lockout?

**Question:** The prompt specifies "account lockout after 5 failed attempts in 15 minutes" but does not state how long the lockout lasts. Is it 15 minutes? Until manual admin reset? Until the 15-minute window expires naturally?

**My Understanding:** The most natural interpretation is that the lockout lasts for the remainder of the 15-minute window. Once 15 minutes have passed since the first failed attempt in the sequence, the attempt counter resets and login is permitted again. No manual admin intervention required.

**Solution Implemented:** Lockout is implicit — `login_attempts` records are queried for the past 15 minutes. If ≥ 5 failed attempts exist in that window, the account is locked. After 15 minutes from the first attempt in the window, those records fall outside the query window and the account unlocks automatically. Admin can also manually clear attempts via the audit panel.

---

## 8. "Discrimination Bands" — What Are the Valid Ranges?

**Question:** The prompt mentions filtering knowledge points by "discrimination bands" alongside difficulty (1–5). It does not define what discrimination bands are or their valid ranges.

**My Understanding:** In educational assessment, item discrimination (often called the discrimination index) measures how well a question distinguishes between high and low performers. Typical ranges are -1.0 to +1.0, where values > 0.3 are considered good, 0.1–0.3 acceptable, and < 0.1 poor or negative discrimination.

**Solution Implemented:** `discrimination` stored as REAL in SQLite (range -1.0 to 1.0). Filter UI offers band presets: Poor (< 0.1), Acceptable (0.1–0.3), Good (0.3–0.5), Excellent (> 0.5). Custom range input also available. The column validation enforces -1.0 ≤ discrimination ≤ 1.0.

---

## 9. Backup Daily vs Monthly Classification — How Is Monthly Determined?

**Question:** The prompt says the system "retains 30 daily versions and 12 monthly archives." It does not explain how a backup transitions from "daily" to "monthly" or which daily backup becomes the monthly archive.

**My Understanding:** The standard approach in backup systems is: the last daily backup of each calendar month is promoted to a monthly archive. This gives exactly one monthly snapshot per month while the daily backups cover the most recent 30 days.

**Solution Implemented:** Backup type is determined at creation time: if today is the last day of the calendar month, the backup is tagged as `monthly`; otherwise it is tagged as `daily`. Lifecycle cleanup keeps the 30 most recent daily backups and 12 most recent monthly backups. Financial and IP-related audit records are preserved regardless of the cleanup schedule per the Admin-configured retention policy.

---

## 10. Restore Sandbox Validation — What Does "Validate" Mean Before Activation?

**Question:** The prompt says "one-click restore into a validation sandbox before activation." It does not specify what validation occurs in the sandbox or what criteria must pass before activation is allowed.

**My Understanding:** The sandbox restore is meant to let the Admin verify the backup is intact before replacing the live database. At minimum this should include: integrity check (SHA-256 hash verification), schema version compatibility check (migration version matches), and a basic read test (can query key tables). The Admin reviews the results and then confirms activation.

**Solution Implemented:** Sandbox restore process: (1) decrypt and extract backup to `/tmp/restore-sandbox/`; (2) run `PRAGMA integrity_check` on the restored SQLite file; (3) verify SHA-256 hash matches the stored backup hash; (4) run `SELECT COUNT(*) FROM users` as a basic read test; (5) return `SandboxValidationReport` with pass/fail for each check. Admin sees the report before clicking "Activate Restore."

---

## 11. Financial and IP Records Preservation — Which Records Qualify?

**Question:** The prompt says lifecycle cleanup should "preserve audited financial and IP records according to an Administrator-configured retention policy." It does not define exactly which records qualify as "financial" or "IP."

**My Understanding:** Financial records logically include: fund transactions, orders, order items, and export logs related to financial reports. IP records logically include: outcome registrations (papers, patents, software copyrights), evidence files, and their associated audit log entries. The Admin should be able to configure how long each category is retained separately.

**Solution Implemented:** Retention policy config stored in DB: `{ financial_retention_years: u32, ip_retention_years: u32 }`. Lifecycle cleanup treats records as preserved if: `entity_type IN ('fund_transaction', 'order', 'order_item', 'export_log')` for financial, or `entity_type IN ('outcome', 'evidence_file')` for IP. Audit log entries referencing these entities are also preserved.

---

## 12. Approval Cycle Time — What Defines Start and End?

**Question:** The prompt mentions "approval cycle time" as a dashboard metric. It does not specify what events mark the start and end of an approval cycle, or which entity types have approval cycles.

**My Understanding:** An approval cycle applies to outcomes (paper/patent/competition/software copyright) that go through a review process. The cycle starts when a Reviewer submits an outcome for approval (`status = 'submitted'`) and ends when an approver marks it approved or rejected. The cycle time is the elapsed time between these two events.

**Solution Implemented:** `approval_cycle_records` table records: `submitted_at` (when status changes to 'submitted'), `approved_at` (when status changes to 'approved' or 'rejected'), `approver_id`, `cycle_time_minutes` (computed on close). Dashboard shows: average cycle time, median cycle time, distribution histogram, and slowest pending items. Filters available by outcome type and date range.