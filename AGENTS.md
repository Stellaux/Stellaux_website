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

## 🔁 Workflow (Every Change)

1. **/plan** – User describes a feature. Architect agent creates a plan.
2. **User reviews plan** – Approves or requests changes.
3. **/implement** – Developer agent follows the plan, writes code, runs `cargo check`.
4. **/qa** – QA agent runs checks. If passes → ready for PR. If fails → Developer fixes.
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