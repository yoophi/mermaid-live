# Mermaid Live

A Tauri desktop app for editing Mermaid diagrams with a live preview.

## Stack

- pnpm monorepo
- Tauri
- React + Vite + TypeScript
- shadcn/ui-style shared components
- Feature-Sliced Design frontend
- Hexagonal architecture native layer

## Getting Started

```bash
pnpm install
pnpm dev
```

For the native app:

```bash
pnpm tauri dev
```

## Cleanup

Remove development-generated resources such as `node_modules`, `apps/desktop/dist`,
and `apps/desktop/src-tauri/target`:

```bash
pnpm run clean
```
