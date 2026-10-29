# RewriteAI — SaaS Text Rewriter

> AI-powered text rewriting with **OpenAI Structured Outputs**, **Axum backend**, and a **React + Tailwind** frontend.

---

## Features

| Feature | Description |
|---|---|
| **Manual Rewrite** | Choose tone (Professional / Casual / Persuasive / …) and style (Improve / Fix / Condense / …) |
| **Auto Rewrite** | Analyzes text first, then automatically picks the best strategy |
| **Text Analysis** | Quality score, readability level, issue detection, strengths |
| **Structured Outputs** | OpenAI JSON Schema enforced — no hallucinated fields, guaranteed parsing |
| **Auth** | JWT-based register/login, bcrypt passwords |
| **Quota system** | Per-plan rewrite limits (Free: 10, Pro: 500, Enterprise: ∞) |
| **Async jobs** | Rewrites run in background Tokio tasks |

---

## Architecture

```
┌─────────────────┐       ┌──────────────────────────────────────────┐
│  React Frontend │──────▶│           Axum Backend (Rust)            │
│  (Vite + TW)   │       │  /api/v1/auth   /api/v1/rewrite          │
└─────────────────┘       │  /api/v1/user                            │
                          └──────────────────┬───────────────────────┘
                                             │
                          ┌──────────────────▼───────────────────────┐
                          │          PostgreSQL (sqlx)                │
                          └──────────────────────────────────────────┘
                                             │
                          ┌──────────────────▼───────────────────────┐
                          │     OpenAI GPT-4o (Structured Outputs)   │
                          │     JSON Schema enforced responses        │
                          └──────────────────────────────────────────┘
```

---

## Quick Start

### Prerequisites
- Rust 1.76+
- Node.js 20+
- PostgreSQL 15+ (or Docker)
- OpenAI API key

### 1. Clone & configure

```bash
git clone https://github.com/your-org/saas-rewriter
cd saas-rewriter
cp .env.example .env
# Edit .env — add OPENAI_API_KEY and JWT_SECRET
```

### 2. Start with Docker (easiest)

```bash
OPENAI_API_KEY=sk-... docker compose up
```

Frontend: http://localhost:5173  
Backend: http://localhost:3001

### 3. Manual setup

**Database:**
```bash
createdb rewriteai
```

**Backend:**
```bash
cd backend
cargo run
```

**Frontend:**
```bash
cd frontend
npm install
npm run dev
```

---

## API Reference

All protected routes require `Authorization: Bearer <token>`.

### Auth
| Method | Path | Body | Description |
|---|---|---|---|
| POST | `/api/v1/auth/register` | `{email, password}` | Create account |
| POST | `/api/v1/auth/login` | `{email, password}` | Get JWT token |

### Rewrite
| Method | Path | Body | Description |
|---|---|---|---|
| POST | `/api/v1/rewrite` | `{text, tone, style}` | Create rewrite job (async) |
| POST | `/api/v1/rewrite/auto` | `{text}` | Auto-analyze + rewrite (sync) |
| POST | `/api/v1/rewrite/analyze` | `{text}` | Analyze only |
| GET | `/api/v1/rewrite/history` | — | Last 50 jobs |
| GET | `/api/v1/rewrite/:id` | — | Job status + result |

### User
| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/user/me` | Current user |
| GET | `/api/v1/user/usage` | Quota info |

---

## OpenAI Structured Outputs

This project uses OpenAI's **JSON Schema mode** (`"type": "json_schema"` with `"strict": true`), which guarantees the response matches the schema exactly — no need for fragile regex parsing.

```rust
// Example schema enforcement
let body = json!({
    "model": "gpt-4o-2024-08-06",
    "response_format": {
        "type": "json_schema",
        "json_schema": {
            "name": "rewrite_response",
            "strict": true,
            "schema": rewrite_response_schema()  // Defined in openai.rs
        }
    }
});
```

**Guaranteed fields in every rewrite:**
- `rewritten_text` — the actual rewrite
- `readability_score` (1–10)
- `tone_match_score` (1–10)  
- `improvements[]` — categorized list of what changed
- `word_count_original` / `word_count_rewritten`
- `warnings[]` — edge cases flagged by the model

---

## Auto Rewrite Flow

```
Input text
    │
    ▼
analyze_text()  ─── TextAnalysis (quality, issues, tone)
    │
    ▼
pick_best_strategy()  ─── selects Tone + Style heuristically
    │
    ▼
rewrite_text()  ─── RewriteResponse (structured)
    │
    ▼
Return both analysis + rewrite to client
```

---

## Project Structure

```
saas-rewriter/
├── backend/
│   ├── src/
│   │   ├── main.rs          # Axum server setup
│   │   ├── config.rs        # Env config
│   │   ├── error.rs         # AppError → HTTP responses
│   │   ├── models.rs        # DB types (User, RewriteJob, …)
│   │   ├── db.rs            # PgPool + migrations
│   │   ├── middleware/      # JWT auth middleware
│   │   ├── routes/
│   │   │   ├── auth.rs      # Register / login
│   │   │   ├── rewrite.rs   # Core rewrite endpoints
│   │   │   └── user.rs      # Profile / usage
│   │   └── services/
│   │       ├── openai.rs    # OpenAI client + structured schemas
│   │       └── auth.rs      # JWT encode/decode
│   └── migrations/
│       └── 20240101000000_initial.sql
├── frontend/
│   └── src/
│       ├── components/
│       │   ├── App.tsx       # Main rewriter UI
│       │   └── AuthPage.tsx  # Login / register
│       ├── hooks/
│       │   └── useAuth.ts    # Zustand auth store
│       └── lib/
│           └── api.ts        # Typed API client
├── docker-compose.yml
├── .env.example
└── README.md
```

---

## Plans & Quota

| Plan | Rewrites/month | Price |
|---|---|---|
| Free | 10 | $0 |
| Pro | 500 | $19/mo |
| Enterprise | Unlimited | Custom |

Quota is enforced server-side before each OpenAI call.

---

## License

MIT
