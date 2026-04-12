# Audit Fix Check — Prior Issues Re-Verification (Static-Only)

Date: 2026-04-12

Scope: Re-check of the 7 issues listed in `audit_report-1.md` using static code/document inspection only (no runtime execution, no tests run).

## Summary

- **Fixed:** 7
- **Partially Fixed:** 0
- **Not Fixed:** 0

---

## 1) High — End-to-end HTTPS not statically implemented at app listener layer

- **Current status:** **Fixed**
- **What changed:** Backend now has a first-class in-process TLS serving path and chooses it when transport mode is `InProcessTls`.
- **Evidence:**
  - `backend/src/main.rs:33` (`enforce_transport_security()` called)
  - `backend/src/main.rs:117`–`126` (transport-mode branch dispatches to `serve_tls`)
  - `backend/src/main.rs:145` (`serve_tls(...)` defined)
  - `backend/src/main.rs:193` (`rustls::ServerConfig` built)
  - `backend/src/main.rs:201` (TLS acceptor)
  - `backend/src/main.rs:301` onward (transport mode policy docs and enforcement)
- **Conclusion:** Static evidence now demonstrates both policy enforcement and concrete TLS listener implementation at app layer.

---

## 2) High — Duplicate detection used exact certificate match only

- **Current status:** **Fixed**
- **What changed:** Certificate matching now includes normalized similarity scoring in addition to exact match.
- **Evidence:**
  - `backend/src/services/outcome_service.rs:26` (`CERT_SIMILARITY_THRESHOLD`)
  - `backend/src/services/outcome_service.rs:32` (`normalize_certificate`)
  - `backend/src/services/outcome_service.rs:234`–`235` (normalized certificate comparison)
  - `backend/src/services/outcome_service.rs:237` (threshold check)
  - `backend/src/services/outcome_service.rs:243` ("certificate similarity" reason)
  - `backend/tests/unit_tests/outcome_tests.rs:252`–`256` (normalization tests)

---

## 3) Medium — Analytics filter surface narrow (period-only)

- **Current status:** **Fixed**
- **What changed:** Structured filter DTO is implemented and wired through service query logic, including `role`.
- **Evidence:**
  - `backend/src/handlers/analytics.rs:52`–`61` (`PeriodQuery` has `period`, `date_from`, `date_to`, `category`, `role`)
  - `backend/src/handlers/analytics.rs:70`–`77` (`fund_summary` passes all filter fields to service)
  - `backend/src/services/analytics_service.rs:134`–`141` (`get_fund_summary` accepts full filter set)
  - `backend/src/services/analytics_service.rs:151`, `:155`, `:159`, `:163` (date/category/role SQL predicates)
- **Conclusion:** The previously missing role/custom-filter wiring is now present for fund-summary analytics flow.

---

## 4) Medium — Placeholder dashboard route/page remained

- **Current status:** **Fixed**
- **What changed:** `/dashboard` now routes to a real `DashboardHome` component instead of placeholder.
- **Evidence:**
  - `frontend/src/app.rs:22` (`/dashboard` -> `DashboardHome`)
  - `frontend/src/app.rs:38` (`DashboardHome` implementation)
- **Note:** `frontend/src/pages/dashboard.rs` remains an intentionally empty module stub for tree consistency.
  - `frontend/src/pages/dashboard.rs:1`

---

## 5) Medium (Suspected Risk) — Session loading did not bind to IP/user-agent

- **Current status:** **Fixed**
- **What changed:** Session middleware now validates both user-agent and IP context against persisted session metadata.
- **Evidence:**
  - `backend/src/middleware/session.rs:99` (query includes `user_agent`, `ip_address`)
  - `backend/src/middleware/session.rs:115` onward (user-agent mismatch check + invalidation)
  - `backend/src/middleware/session.rs:130` onward (IP mismatch detection path)
- **Conclusion:** The originally reported absence of UA/IP session binding checks is resolved; implementation uses hard UA enforcement and tolerant IP policy.

---

## 6) Medium — Bulk update affected-count logic inaccurate

- **Current status:** **Fixed**
- **What changed:** Counting now tracks unique touched IDs instead of `max(rows_affected)`.
- **Evidence:**
  - `backend/src/services/knowledge_service.rs:507` (dedupe IDs)
  - `backend/src/services/knowledge_service.rs:512` (touched set)
  - `backend/src/services/knowledge_service.rs:526`, `:543`, `:557` (add touched IDs when updates affect rows)
  - `backend/src/services/knowledge_service.rs:562` (`Ok(touched_ids.len())`)

---

## 7) Low — Documentation depth thin for acceptance reproducibility

- **Current status:** **Fixed**
- **What changed:** README now includes non-Docker run path, configuration matrix, transport security modes, and verification checklist.
- **Evidence:**
  - `README.md:18` (run without Docker)
  - `README.md:39` (configuration matrix)
  - `README.md:56` (transport security modes)
  - `README.md:79` (verification checklist)

---

## Final Re-check Verdict

- All 7 previously reported items are now **statically addressed**.
- Residual note: session IP mismatch policy is intentionally tolerant (warn/log path), which is a security-policy choice rather than an implementation gap for the original finding.
