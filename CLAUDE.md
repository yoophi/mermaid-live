# Mermaid Live Agent Guide

## Project Shape

This repository is a pnpm monorepo. The desktop product lives in `apps/desktop` and is a Tauri application with a React/Vite frontend.

## Frontend Architecture

The frontend uses Feature-Sliced Design.

- `src/app`: application shell, providers, global styles, and app-level wiring.
- `src/pages`: route-level screens.
- `src/widgets`: composed UI regions that combine features and entities.
- `src/features`: user actions and workflows.
- `src/entities`: domain-facing frontend models and small business primitives.
- `src/shared`: reusable UI, utilities, API adapters, and cross-cutting primitives.

Use `shadcn/ui` conventions for shared UI components. Components generated or maintained for shadcn belong under `src/shared/ui`, with utilities in `src/shared/lib`.

## Native Architecture

The Tauri native side uses hexagonal architecture.

- `domain`: core domain types and rules.
- `application`: use cases and ports. Use cases depend on ports, not infrastructure.
- `adapters/inbound`: Tauri commands and other inbound delivery mechanisms.
- `adapters/outbound`: concrete implementations of application ports.
- `infrastructure`: runtime composition, state, and framework-specific setup.

Keep business rules out of Tauri command handlers. Commands should parse input, call application services, and map results back to the frontend.

## Engineering Rules

- Prefer small, explicit modules over broad utility files.
- Keep frontend imports directionally aligned with FSD. Lower layers must not import higher layers.
- Keep native domain and application layers free from Tauri-specific types.
- Add tests around shared behavior and native use cases as the app grows.

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
