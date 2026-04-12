# Audit Fix Check — `audit_report-2.md` Findings Re-Verification (Static-Only)

Date: 2026-04-12
Scope: Re-check of the 5 findings from `/.tmp/audit_report-2.md` using static code/test inspection only (no runtime execution).

## Summary

- **Fixed:** 5
- **Partially Fixed:** 0
- **Not Fixed:** 0

---

## 1) Blocker — Checkout trust boundary allows client-side price tampering

- **Current status:** **Fixed**
- **What changed:** Checkout now resolves cart items from authoritative DB product rows and ignores client-supplied `unit_price`/`product_name`.
- **Evidence:**
  - `backend/src/services/store_service.rs:217` (`create_order` calls `resolve_cart_from_db`)
  - `backend/src/services/store_service.rs:289` (`resolve_cart_from_db` implementation)
  - `backend/src/services/store_service.rs:316` (server-derived `product_name`)
  - `backend/src/services/store_service.rs:318` (server-derived `unit_price`)
  - `backend/src/handlers/store.rs:144` (`preview_checkout` also uses server-side resolution)
  - `backend/tests/api_tests/store_api.rs:266` (`test_checkout_ignores_client_price_tampering`)
- **Conclusion:** The trust-boundary flaw is addressed: price/name authority is now server-side.

---

## 2) High — Mutation audit trail does not consistently record real before/after hashes

- **Current status:** **Fixed**
- **What changed:** Audit service now enforces hash-pair completeness; CSRF rotation is audited; regression tests assert hash presence.
- **Evidence:**
  - `backend/src/services/audit_service.rs:22` (`HASH_ENTITY_CREATED` sentinel)
  - `backend/src/services/audit_service.rs:24` (`HASH_ENTITY_DELETED` sentinel)
  - `backend/src/services/audit_service.rs:61`, `:62`, `:67`, `:71` (hash-gap fill logic)
  - `backend/tests/api_tests/auth_api.rs:327` (`test_refresh_csrf_writes_audit_record`)
  - `backend/tests/api_tests/auth_api.rs:343` (queries latest `csrf_token` audit row)
  - `backend/tests/api_tests/auth_api.rs:356` (`test_audit_hashes_always_populated`)
  - `backend/tests/api_tests/auth_api.rs:380` (asserts no row has null hash side)
- **Conclusion:** Prior inconsistency is materially addressed with enforced pair presence and targeted regression coverage.
- **Residual note:** Create/delete semantics still rely on canonical sentinels by design.

---

## 3) High — Analytics custom-filter requirement only partially surfaced in UI/API usage

- **Current status:** **Fixed**
- **What changed:** Role/date/category filters are now surfaced in analytics UI and carried through API + scheduling pipeline.
- **Evidence:**
  - `frontend/src/pages/analytics/dashboard.rs:17`, `:22`, `:52`, `:147`, `:148` (role filter state, apply wiring, and UI control)
  - `frontend/src/pages/analytics/reports.rs:19`, `:34`, `:123`, `:152`, `:183` (role control + schedule/export payload wiring)
  - `frontend/src/api/analytics.rs:121`, `:148` (`ScheduleReportRequest` + `AnalyticsFilter` include `role`)
  - `backend/src/handlers/analytics.rs:61`, `:170`, `:186` (`role` accepted for fund and scheduled report flows)
  - `backend/src/services/analytics_service.rs:234`, `:249`, `:471`, `:486` (role propagated into filtered generation + persisted filters)
- **Conclusion:** The previously missing filter-surface parity is now statically present end-to-end.

---

## 4) Medium — Duplicate-flag workflow not tightly coupled to mandatory side-by-side compare before submission

- **Current status:** **Fixed**
- **What changed:** Duplicate review now includes in-flow compare action that loads side-by-side data inside register flow, plus explicit acknowledgement gating before submit.
- **Evidence:**
  - `frontend/src/pages/outcomes/register.rs:41` (`inline_compare` state)
  - `frontend/src/pages/outcomes/register.rs:168`, `:169` (duplicate-row compare triggers `compare_outcomes` and stores result)
  - `frontend/src/pages/outcomes/register.rs:196` (inline side-by-side compare panel rendered in registration flow)
  - `frontend/src/pages/outcomes/register.rs:39`, `:183`, `:338`, `:346` (acknowledgement gate controls submit availability)
- **Conclusion:** Compare-and-acknowledge is now tightly integrated into pre-submit workflow rather than being only a separate manual tab flow.

---

## 5) Medium — Lockout query combines username OR IP, enabling broad IP-level denial scenarios

- **Current status:** **Fixed**
- **What changed:** Lockout is now account-centric (`username` only), with explicit regression test for same-IP different-user behavior.
- **Evidence:**
  - `backend/src/services/auth_service.rs:46` (`check_lockout` signature; `_ip` no longer used for lockout query)
  - `backend/src/services/auth_service.rs:50` (query is `WHERE username = ?`)
  - `backend/src/services/auth_service.rs:61` (lockout threshold response)
  - `backend/tests/api_tests/auth_api.rs:390` (`test_lockout_does_not_cross_accounts_on_same_ip`)
- **Conclusion:** Cross-account same-IP lockout risk identified in the finding is addressed.

---

## Final Re-check Verdict

All previously reported findings in `audit_report-2.md` are now **statically addressed**.

- The former Blocker remains resolved.
- The two previously partial findings (analytics filter surface and duplicate compare workflow coupling) are now fully implemented based on current static evidence.
