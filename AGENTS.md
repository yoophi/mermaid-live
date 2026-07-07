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

<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan:
`specs/001-mermaid-zoom-fit/plan.md`

Write Spec Kit `spec.md`, `plan.md`, and `tasks.md` files in Korean.
<!-- SPECKIT END -->
