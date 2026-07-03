# AGENTS.md – Project Instructions for AI Agents

This file governs all AI agents (Claude Code, Codex, Cursor, etc.) working on the `stellaux_server` project.  
**Read this before any planning or coding.**

---

## 🧭 Project Overview

- **Language**: Rust (stable)
- **Web Framework**: Axum
- **ORM**: SeaORM (entity definitions in `src/entity/`)
- **Database**: PostgreSQL (migrations in `src/migration/`) 
- **Architecture style**: Transitioning from flat routes (`src/domain/`) to layered DDD (`src/domains/`)

### Critical naming clarification

| Folder | Current purpose | Target purpose |
|--------|----------------|----------------|
| `src/domain/` | HTTP route handlers (misnamed) | **Deprecated** – move logic to `src/domains/*` |
| `src/domains/` | DDD layered scaffolding | **Active** – home for all business logic |

**Rule**: Do not add new code to `src/domain/`. Migrate existing features into `src/domains/{bounded_context}/` following the layout below.

---

## 🤖 Specialized Agents – Roles & Boundaries

We use four agent roles in sequence. Each agent has strict permissions.

### 1. Architect
- **Job**: Create a detailed plan for any non‑trivial change (>3 files or new feature).
- **Allowed tools**: `Read`, `Grep`, `ls`, `find` (inspection only). **No writes, no shell commands that change state**.
- **Output**: A plan in `plans/YYYY-MM-DD-feature-name.md` that lists:
  - Which bounded context (e.g., `auth`, `catalog`) is affected.
  - New files to create (with full paths under `src/domains/`).
  - Which layers (api, application, domain, infra, dto) will be modified.
  - Any changes to `src/common/` (only if truly cross‑cutting).
- **Gate**: Plan must be approved by the user before the Developer starts.

### 2. Developer
- **Job**: Implement the approved plan.
- **Allowed tools**: `Write`, `Edit`, `Bash` (restricted – see rules), `Grep`.
- **Constraints**:
  - Never write code in `src/domain/` (except to delete or redirect).
  - Never import from `src/domain/` into `src/domains/`.
  - Follow the exact file paths and layer responsibilities from the plan.
  - Add unit tests for domain logic (`#[cfg(test)]` inside each module).
- **Gate**: After writing, the Developer must run `cargo check` and `cargo test --lib` – no further action until QA.

### 3. QA Agent
- **Job**: Verify the change meets architectural and quality rules.
- **Allowed tools**: `Read`, `Grep`, `Bash` (only `cargo check`, `cargo test`, `cargo clippy`).
- **Checks**:
  - No layer violations (e.g., `application/` importing `infra/`).
  - No direct DB queries in `api/` or `application/` – must go through repository traits.
  - All new `domain/` models are pure (no `serde`, no `axum`, no `sqlx`).
- **Output**: A report `reports/QA-{feature}.md` with pass/fail. On fail, send back to Developer.

### 4. Deployment Agent (optional – when deploying)
- **Job**: Run database migrations and deploy the service.
- **Allowed tools**: `Bash` with a whitelist (`cargo run --bin migration`, `docker compose up`, etc.).
- **Gate**: Only after QA passes and user gives explicit `/deploy` command.

---

## 📐 Architectural Rules (Non‑negotiable)

### A. Directory boundaries (most important)
| Directory | Purpose | Who can write |
|-----------|---------|----------------|
| `src/domains/<context>/api/` | HTTP request/response wiring | Developer |
| `src/domains/<context>/application/` | Use cases | Developer |
| `src/domains/<context>/domain/` | Entities, value objects, repository traits | Developer (must keep pure) |
| `src/domains/<context>/infra/` | Concrete DB/HTTP/JWT implementations | Developer |
| `src/domains/<context>/dto/` | Serde request/response structs | Developer |
| `src/common/` | Shared config, DB pool, middleware, etc. | Developer (after architect approval) |
| `src/entity/` | SeaORM entity files – auto‑generated | **No agent may edit directly** |
| `src/migration/` | Migration files – manual only after schema change | Developer with caution |

### B. Layer dependency rules (clean architecture)
- ✅ `api/` → `application/` + `dto/` + `common/` (for state)
- ✅ `application/` → `domain/` (ports + models)
- ✅ `infra/` → `domain/` (implements ports) + external crates
- ✅ `domain/` → **no external dependencies** (not even `common/`)
- ❌ `application/` **cannot** import `infra/`
- ❌ `api/` **cannot** call `infra/` directly
- ❌ `domain/` **cannot** import `dto/`, `serde`, `axum`, `sqlx`

### C. Code style & testing
- All `domain/` models must have unit tests for invariants.
- Use `#[async_trait]` for repository ports.
- Errors: Use `thiserror` in `domain/` (business errors). Use `common::error::AppError` for HTTP layer.
- Naming: `{action}{Resource}UseCase` e.g., `LoginUserUseCase`, `CreateProductUseCase`.

---

## 📈 Observability (Non‑negotiable – build it in, never bolt it on)

**Role mindset**: Observability ships *with* the feature, in the same PR. A use case, endpoint, or job is not "done" until its metrics and tracing are in place and visible on `/metrics`. Treat missing metrics as a layer violation: QA fails the change.

### Stack & hosting

- **Metrics**: `metrics` + `axum-prometheus` (already in `Cargo.toml`) – expose a plain‑text `/metrics` endpoint for Prometheus scraping.
- **Dashboards**: Grafana (internal port `3000`) with a Prometheus data source.
- **Logging/Tracing**: `tracing` + `tracing-subscriber` (JSON in prod, pretty in dev, controlled by `RUST_LOG`, default `info`).
- **Hosting**: Prometheus (`9090`) and Grafana (`3000`) run as sidecars on the same VPS / `docker-compose.yml`, bound to `localhost` or the internal network. **Never exposed to the public internet** – operators reach Grafana via SSH tunnel or internal VPN only.

### Where metrics code lives (respect the layering)

| Concern | Location | Notes |
|---------|----------|-------|
| Shared Prometheus registry + recorder handle | `src/common/` (e.g. `common::observability`) | Single registry, initialized at startup |
| HTTP middleware (`axum-prometheus`) + `/metrics` route | `src/common/` wired in `api/` router | Plain‑text endpoint |
| Metrics **decorators** wrapping repository/provider ports | `src/domains/<context>/infra/` | Decorator pattern – wrap the real `impl`; **never put metric calls in `domain/` or `application/`** |
| `#[instrument]` tracing spans | `application/` use cases + `infra/` adapters | Keep `domain/` pure (no `tracing` import in `domain/` models) |

### Implementation rules (apply per module/feature)

1. **HTTP layer (Axum)** – middleware records: request counter (labels: `method`, `path`, `status`), request‑duration histogram, in‑flight requests gauge. Expose `/metrics`.
2. **Core domains** (e.g. `catalog`, `auth`) – for each port implementation (e.g. `ProductRepository::save`), the **infra decorator** records: call counter (labels: `context`, `method`, `result=success|failure`) and a duration histogram. Business logic in `application/`/`domain/` stays metrics‑free.
3. **Jobs / background work** – per job type record: enqueued, started, completed, failed counters; duration histogram; current queue depth gauge. Use the same registry.
4. **Database / outbound clients** (SeaORM pool, `reqwest`) – wrap with query/request count + latency; open‑connections gauge where available.
5. **Prometheus config** – scrape `/metrics` every `15s`; retain `15–30 days`; run as a systemd unit or `docker-compose` service.
6. **Grafana** – pre‑provision the Prometheus data source (`http://localhost:9090`) and ship a checked‑in dashboard JSON (request rate, error rate, P95 latency, job queue depth, resource usage).

### Fail‑open & toggles (non‑negotiable)

- The backend **must start and serve `/metrics` even if Prometheus/Grafana are down**. Observability never blocks feature delivery or request handling.
- Honor `DISABLE_METRICS=1` to no‑op the recorder for local dev.
- Never block a feature on a "fancy" dashboard – but always merge the metric code alongside the feature.

### Acceptance (QA gate)

A reviewer can run the backend locally, hit `/metrics`, and see at least request counters + duration histograms for the implemented endpoints, plus the per‑feature metrics defined for that change, and import the checked‑in Grafana dashboard without manual edits.

---

## 🔁 Workflow (Every Change)

1. **/plan** – User describes a feature. Architect agent creates a plan.
2. **User reviews plan** – Approves or requests changes.
3. **/implement** – Developer agent follows the plan, writes code, **wires the metrics decorator + `#[instrument]` immediately (not later)**, runs `cargo check`.
4. **/qa** – QA agent runs checks **and verifies the feature's metrics appear on `/metrics`**. If passes → ready for PR. If fails → Developer fixes.
5. **/deploy** (optional) – Deployment agent runs migrations + restarts service.

**For trivial changes** (typo, one‑line fix, renaming): skip planning. But still respect architectural rules.

---

## 🚫 Prohibited Actions (Harness‑enforced)

- Writing to `src/domain/` (except deletion).
- Running `DROP TABLE` or `TRUNCATE` in any SQL.
- Using `unsafe` without explicit approval in the plan.
- Committing secrets (`.env`, `*.pem`) – catch via `grep` in QA.

---

## 📝 Agent Interaction Example

**User**: “Add a logout endpoint that blacklists the JWT.”

**Architect** (creates `plans/2025-03-15-logout.md`):
```markdown
## Plan: Logout endpoint
1. Create `src/domains/auth/domain/ports.rs` – add `TokenBlacklistRepository` trait.
2. Create `src/domains/auth/infra/redis_blacklist_repo.rs` – implement with Redis.
3. Create `src/domains/auth/application/logout_use_case.rs` – invalidates token.
4. Add route in `src/domains/auth/api/routes.rs` – POST /auth/logout.
5. Update `src/common/app_state.rs` to include Redis client.

## 🗄️ SQL Migrations (Even with SeaORM)

We use **traditional SQL** as the single source of truth for the database schema. SeaORM entities are **generated from the SQL schema**, not the other way around.

### Where SQL migrations live
the-polished-standard/
├── shared/models/ # Canonical SQL schema (source of truth)
│ ├── 001_initial_catalog.sql
│ ├── 002_user_roles.sql
│ └── ...
├── stellaux_server/src/migration/ # SeaORM migration runner (only executes SQL)
│ ├── m20260516_000001_initial_catalog.rs # Embeds the SQL from shared/models/
│ └── ...


### Workflow for schema changes

1. **Write or modify SQL** in `shared/models/<NNN_description>.sql`  
   - Use plain, idempotent SQL (`CREATE TABLE IF NOT EXISTS`, `ALTER TABLE ADD COLUMN IF NOT EXISTS`).  
   - Include both `up` and `down` comments/scripts (or use separate `.up.sql` / `.down.sql`).

2. **Update SeaORM entity files** (do not hand‑edit)  
   ```bash
   # After changing SQL, regenerate entities
   sea-orm-cli generate entity -o stellaux_server/src/entity
   ```

---

## 📚 Documentation (write it as you build – never leave it for "later")

**Non‑negotiable**: Documentation ships *with* the change, in the same PR. A feature is not "done" until its plan, any schema changes, and its API surface are documented. QA fails an undocumented change.

### Where documentation lives

| Doc type | Location | Owner |
|----------|----------|-------|
| Planning documents | `plans/YYYY-MM-DD-feature-name.md` | Architect |
| QA reports | `reports/QA-{feature}.md` | QA Agent |
| Long‑lived reference / ADRs / runbooks | `docs/` | Developer |
| Canonical DB schema (source of truth) | `shared/models/<NNN_description>.sql` | Developer |
| API reference (OpenAPI) | **Generated at compile time** from `utoipa` annotations | Developer |

### 1. Planning documents (required for every non‑trivial change)

Before any code, the Architect writes `plans/YYYY-MM-DD-feature-name.md`. It must contain:

- **Context & goal** – what problem, which bounded context (e.g. `auth`, `catalog`).
- **Files to create/modify** – full paths under `src/domains/<context>/`, grouped by layer (api, application, domain, infra, dto) and any `src/common/` changes.
- **Proposed schema changes** – see §2 (link or inline the SQL).
- **Observability plan** – which metrics/spans this feature adds (see the Observability section).
- **Open questions / risks** – anything needing user approval.

Keep the plan updated if the implementation diverges; the plan is the record of intent, not a throwaway.

### 2. Proposed schema changes (document before you migrate)

Any change touching the database **must be written up in the plan first**, then implemented as SQL:

1. **Describe the change in the plan** – tables/columns added or altered, why, and the backfill/rollback story.
2. **Author the SQL** in `shared/models/<NNN_description>.sql` (the source of truth) using idempotent statements (`CREATE TABLE IF NOT EXISTS`, `ALTER TABLE ADD COLUMN IF NOT EXISTS`) with both `up` and `down`.
3. **Regenerate** SeaORM entities (`sea-orm-cli generate entity -o stellaux_server/src/entity`) – never hand‑edit `src/entity/`.
4. **Record the migration** in `src/migration/` (embeds the SQL). Reference the plan and the `shared/models/` file in the migration's doc comment.
5. **Never** `DROP TABLE` / `TRUNCATE` (see Prohibited Actions). Destructive changes need an explicit, approved plan entry.

### 3. API & Endpoints (OpenAPI via utoipa)

1. **Always annotate your handlers with macros** (`#[utoipa::path(...)]`) – this generates the OpenAPI spec at compile time. Use the dedicated `utoipa-axum` bindings for router wiring and the separate `utoipa-swagger-ui` crate to serve the interactive UI.
2. **Annotate DTOs** in `dto/` with `#[derive(ToSchema)]` so request/response bodies appear in the spec.
3. Every new or changed route under `src/domains/<context>/api/` must update its `#[utoipa::path(...)]` (method, path, params, responses, tags) in the same PR – an endpoint without an annotation is incomplete.
4. **Acceptance**: a reviewer can build the backend, open the Swagger UI, and see the new/changed endpoint with accurate request/response schemas.





