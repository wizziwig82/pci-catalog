# Project Reorganization Plan

## 1. Analysis of Current Structure

The project is a SvelteKit frontend (`src`) combined with a Tauri backend (`src-tauri`). The current structure includes:

*   **Root (`/`)**: Configuration files, documentation (`README.md`, `PRD.md`), setup scripts, miscellaneous logs/notes.
*   **`docs/`**: Project documentation (summaries, schema).
*   **`e2e/`**: End-to-end tests (Playwright).
*   **`messages/` & `project.inlang/`**: Internationalization.
*   **`scripts/`**: Helper scripts (MongoDB setup).
*   **`src/`**: SvelteKit Frontend.
    *   `lib/`: Contains components, server logic (`auth.ts`), stores, Tauri bindings, types, utils. This is a common SvelteKit pattern but can become cluttered.
    *   `routes/`: SvelteKit page routes.
    *   `stories/`: Storybook components.
*   **`src-tauri/`**: Tauri Backend (Rust).
    *   `src/`: Main Rust code, grouped by technical function (commands, errors, credentials, upload, audio, storage).
    *   `capabilities/`, `icons/`: Tauri configuration and assets.
*   **`static/`**: Static assets.
*   **`tests/`**: Unit and integration tests, separated by type and frontend/backend.

## 2. Identified Areas for Improvement

*   **`src/lib` Organization**: Too many different concerns mixed together.
*   **`src/lib/server`**: Limited use; server logic could be better integrated with SvelteKit conventions or feature slices.
*   **`src-tauri/src` Structure**: Grouping by technical function may become less clear than grouping by feature domain as the backend grows.
*   **Testing Structure (`tests/`)**: Good separation, but could benefit from mirroring source structure more closely for easier navigation.
*   **Root Directory Clutter**: Planning/documentation files (`PRD.md`, etc.) could be moved to `docs/`.

## 3. Recommended Reorganization Strategy (Hybrid Feature/Layer Approach)

```
/
├── docs/
│   ├── architecture/         # Diagrams, decisions (NEW)
│   ├── guides/               # How-tos (NEW)
│   ├── PRD.md                # Moved from root
│   ├── database-catalog-plan.md # Moved from root
│   └── ...                   # Existing docs
├── e2e/                      # Kept separate or moved under tests/
├── messages/
├── project.inlang/
├── scripts/
├── src/                      # SvelteKit Frontend
│   ├── hooks.server.ts
│   ├── hooks.client.ts       # (If needed)
│   ├── app.html, app.d.ts, app.css
│   ├── lib/                  # Shared Core Frontend Logic
│   │   ├── components/       # Shared, reusable UI (Presentational)
│   │   │   └── common/       # General purpose (Button, Input)
│   │   │   └── layout/       # Layout components (Header, Sidebar)
│   │   ├── stores/           # Global/shared Svelte stores
│   │   ├── types/            # Shared TypeScript types/interfaces
│   │   ├── utils/            # Shared utility functions
│   │   ├── config/           # App configuration constants (NEW)
│   │   └── tauri/            # Tauri API bindings (Keep as is)
│   ├── features/             # Feature-specific Modules (NEW)
│   │   ├── catalog/
│   │   │   ├── components/   # Feature-specific components
│   │   │   ├── stores/       # Feature-specific stores
│   │   │   ├── types.ts
│   │   │   └── utils.ts
│   │   ├── upload/           # (Structure mirrors catalog)
│   │   ├── settings/         # (Structure mirrors catalog)
│   │   └── auth/             # Authentication Feature
│   │       ├── components/
│   │       ├── stores/
│   │       └── server/       # Server-side auth logic (moved from lib/server)
│   ├── routes/               # SvelteKit Routes (imports from /lib and /features)
│   │   └── ...
│   └── stories/              # Storybook
├── src-tauri/                # Tauri Backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── error.rs          # Centralized errors
│   │   ├── config.rs         # Backend configuration (NEW)
│   │   ├── features/         # Feature-specific Backend Modules (NEW)
│   │   │   ├── catalog/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── commands.rs # Feature-specific Tauri commands
│   │   │   │   ├── storage.rs  # Feature-specific logic
│   │   │   │   └── ...
│   │   │   ├── upload/       # (Structure mirrors catalog)
│   │   │   ├── credentials/    # (Structure mirrors catalog)
│   │   │   └── mod.rs          # Export feature modules
│   │   └── core/               # Core Backend Services/Utils (NEW, optional)
│   │       ├── mod.rs
│   │       └── database.rs     # Example: Shared DB logic
│   ├── build.rs
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── ...
├── static/
├── tests/
│   ├── e2e/                  # Moved from root (Consolidate testing)
│   ├── integration/
│   │   └── ...
│   └── unit/
│       ├── frontend/         # Mirror src structure
│       │   ├── lib/
│       │   └── features/
│       │       └── catalog/
│       └── backend/          # Mirror src-tauri structure
│           └── features/
│               └── catalog/
└── ... (Root config files)
```

## 4. Rationale

*   **Feature Slices (`features/`)**: Improves code locality and navigation by grouping related frontend and backend code.
*   **Clearer `src/lib`**: Dedicates `lib` to truly shared, application-agnostic code.
*   **Consistent Backend Structure**: Applies feature slicing to Rust code for consistency.
*   **Centralized Documentation (`docs/`)**: Consolidates project knowledge.
*   **Logical Server Logic Placement**: Integrates server logic within feature slices or SvelteKit conventions.
*   **Improved Test Structure**: Mirrors source structure for easier test discovery.

This structure aims for better scalability, maintainability, and developer understanding.