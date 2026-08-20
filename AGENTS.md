# Agent Instructions

## Required Architecture

This project is a Tauri app in a pnpm monorepo. Preserve that structure when adding packages or commands.

Frontend work must follow Feature-Sliced Design:

- `app` wires providers and global layout.
- `pages` owns screen-level composition.
- `widgets` owns large reusable page regions.
- `features` owns user workflows.
- `entities` owns product-domain concepts.
- `shared` owns low-level UI, utilities, and adapters.

Native work must follow hexagonal architecture:

- Put core rules in `domain`.
- Put use cases and ports in `application`.
- Put Tauri commands in `adapters/inbound`.
- Put filesystem, persistence, process, and OS integrations in `adapters/outbound`.
- Put dependency wiring in `infrastructure`.

## UI Conventions

Use shadcn/ui-style primitives from `src/shared/ui`. Keep components accessible and keyboard-friendly. Avoid one-off component styling when a shared primitive already exists.

## Commands

- Install dependencies from the repository root with `pnpm install`.
- Run the frontend dev server with `pnpm dev`.
- Run Tauri with `pnpm tauri dev`.
- Typecheck with `pnpm typecheck`.
- Build with `pnpm build`.

## CalVer Versioning

이 프로젝트는 `YYYY.M.#` 형식의 CalVer(Calendar Versioning)를 사용합니다.

- `YYYY`는 릴리스 연도, `M`은 릴리스 월, `#`은 해당 월의 릴리스 순번입니다.
- 연도, 월, 순번을 zero-padding하지 않으며 버전과 태그에 `v` 접두사를 붙이지 않습니다.
- 릴리스 순번은 매월 `1`부터 시작하고, 같은 달에는 기존 태그의 가장 큰 순번에 1을 더합니다.
- 정식 릴리스 예시는 `2026.8.1`, `2026.8.2`, `2026.9.1`입니다. `2026.08.01`, `v2026.8.1`, `0.1.0`은 사용하지 않습니다.

개발 빌드는 선언된 기준 CalVer에 Git 상태를 빌드 과정에서만 주입해 `YYYY.M.#-{short-commit-hash}[-dirty]` 형식으로 식별합니다.

- commit hash는 `git rev-parse --short=7 HEAD`와 같은 7자리 값을 사용합니다.
- `git status --porcelain`에 staged, unstaged 또는 untracked 변경이 있으면 `-dirty`를 붙입니다. ignored 파일은 포함하지 않습니다.
- Git 정보를 읽을 수 없으면 `YYYY.M.#-unknown`을 사용합니다.
- 개발 식별자를 만들기 위해 버전 선언 파일이나 lockfile을 수정하지 않습니다.
- `dirty` 또는 `unknown` 빌드는 정식 또는 재현 가능한 빌드로 취급하지 않습니다.

정식 릴리스는 깨끗한 기본 브랜치에서 만들고, 프로젝트 버전과 `YYYY.M.#` 릴리스 태그가 일치해야 하며 개발 접미사를 포함하지 않아야 합니다. 버전 변경은 기능 변경과 분리한 별도 PR로 진행하고, 버전 선언 파일과 그에 따라 생성되는 lockfile만 포함합니다. 이 PR을 기본 브랜치에 squash merge한 후 최신 기본 브랜치 커밋에 태그와 동일한 이름의 릴리스를 생성합니다.

정식 버전 변경 시 다음 파일을 함께 맞춥니다.

- `package.json`
- `apps/desktop/package.json`
- `apps/desktop/src-tauri/Cargo.toml`
- `apps/desktop/src-tauri/Cargo.lock`
- `apps/desktop/src-tauri/tauri.conf.json`

<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan:
`specs/001-mermaid-zoom-fit/plan.md`

Write Spec Kit `spec.md`, `plan.md`, and `tasks.md` files in Korean.
<!-- SPECKIT END -->
