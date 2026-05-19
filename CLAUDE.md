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
