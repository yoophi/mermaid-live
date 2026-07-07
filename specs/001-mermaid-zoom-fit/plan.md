# 구현 계획: Mermaid Zoom Fit

**브랜치**: `001-mermaid-zoom-fit` | **날짜**: 2026-07-07 | **스펙**: [spec.md](./spec.md)
**입력**: `/specs/001-mermaid-zoom-fit/spec.md`의 기능 명세

**참고**: 이 문서는 `/speckit.plan` 명령으로 작성된 구현 계획입니다. 실행 흐름은 `.specify/templates/plan-template.md`를 기준으로 합니다.

## 요약

기존 Mermaid 미리보기에서 다이어그램이 현재 표시 영역에 가장 적절한 크기로 자동 렌더링되도록 개선한다. 구현은 현재 인라인 미리보기 화면, 줌 컨트롤, 팬 동작을 유지하면서 AW 참조 구현의 방식을 반영한다. 핵심은 렌더링된 SVG의 원본 비율을 유지하고, 제한된 부모 영역 안에 맞추며, 렌더링 완료 및 컨테이너 크기 변경 후 fit 크기를 다시 계산하는 것이다.

## 기술 컨텍스트

**언어/버전**: TypeScript 5.9, React 19, Rust/Tauri 2는 존재하지만 이 기능에서는 변경하지 않을 예정  
**주요 의존성**: Mermaid 11.6, Vite 6, Tailwind CSS 4, lucide-react, 기존 shadcn/ui 스타일 `Button` 프리미티브  
**저장소**: 해당 없음  
**테스트**: `pnpm typecheck`, `pnpm build`, fit 동작에 대한 집중 브라우저/수동 검증. 현재 프로젝트에는 별도 프론트엔드 단위 테스트 실행기가 설정되어 있지 않음  
**대상 플랫폼**: Vite 기반 웹뷰 프론트엔드를 사용하는 Tauri 데스크톱 앱  
**프로젝트 유형**: pnpm 모노레포의 데스크톱 앱  
**성능 목표**: 대표 다이어그램에서 렌더링 또는 미리보기 영역 크기 변경 후 1초 이내에 fit 상태가 안정화됨  
**제약 조건**: 현재 표시 방식을 유지한다. 전체 화면 모달을 새로 도입하지 않는다. Feature-Sliced Design 경계를 유지한다. 이후 작업에서 네이티브 의존성이 발견되지 않는 한 네이티브 hexagonal architecture 영역은 변경하지 않는다  
**범위**: 기존 미리보기 기능 1개에 한정한다. 대상 파일은 `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`와 Mermaid 미리보기 CSS가 있는 `apps/desktop/src/app/styles/global.css`이다

## 헌법 체크

*게이트: Phase 0 리서치 전에 통과해야 하며, Phase 1 설계 후 다시 확인한다.*

constitution 파일은 아직 플레이스홀더 원칙을 포함하고 있으므로, `AGENTS.md`의 저장소 지침 외에 강제 가능한 프로젝트별 게이트는 없다.

- **모노레포/Tauri 구조**: PASS. 계획은 작업을 `apps/desktop` 내부로 유지하고 루트 `pnpm` 명령을 사용한다.
- **프론트엔드 Feature-Sliced Design**: PASS. 이 동작은 기존 `features/preview-diagram` 워크플로와 공유 Mermaid SVG 스타일에 속한다.
- **네이티브 hexagonal architecture**: PASS. 네이티브 코드 변경은 계획하지 않는다.
- **UI 규칙**: PASS. 기존 shadcn/ui 스타일 `Button` 컨트롤을 계속 사용한다.

## 프로젝트 구조

### 문서 구조(이 기능)

```text
specs/001-mermaid-zoom-fit/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── mermaid-preview-ui.md
└── tasks.md
```

### 소스 코드(저장소 루트)

```text
apps/desktop/
├── src/
│   ├── app/
│   │   └── styles/
│   │       └── global.css
│   ├── features/
│   │   └── preview-diagram/
│   │       └── ui/
│   │           └── mermaid-preview.tsx
│   ├── shared/
│   │   └── ui/
│   │       └── button.tsx
│   └── widgets/
│       └── diagram-workspace/
│           └── ui/
│               └── diagram-workspace.tsx
└── src-tauri/
    └── src/
```

**구조 결정**: 이 기능은 미리보기 워크플로를 변경하므로 `features/preview-diagram` 안에서 구현한다. 기존 줌 컨트롤은 `shared/ui/button.tsx`의 버튼 프리미티브를 계속 사용하므로 해당 파일은 변경하지 않는다. 부모 워크스페이스의 크기 제약 조정이 필요한 경우가 아니라면 `widgets/diagram-workspace`는 변경하지 않는다. `src-tauri`는 변경하지 않는다.

## Phase 0: 리서치

[research.md](./research.md)를 참고한다. 계획 단계의 미확정 사항은 모두 해소되었다.

- AW 참조 동작은 SVG viewBox 기반 fit 크기 조정으로 매핑된다.
- 현재 프로젝트에는 이미 zoom/pan 상태와 fit 버튼이 있으며, 부족한 부분은 신뢰할 수 있는 최초 fit 및 resize fit 동작이다.
- 전체 영역 fit을 사용할 때 Mermaid가 SVG에 넣는 인라인 `max-width`를 CSS에서 재정의해야 한다.
- transform 기반 zoom만으로 fit 크기를 표현하면 overflow 레이아웃에 반영되지 않으므로 fit 계산의 중심 방식으로 사용하지 않는다.

## Phase 1: 설계

[data-model.md](./data-model.md), [contracts/mermaid-preview-ui.md](./contracts/mermaid-preview-ui.md), [quickstart.md](./quickstart.md)를 참고한다.

설계 요약:

- 렌더링된 SVG의 원본 크기는 `viewBox`에서 먼저 측정하고, 없으면 width/height 속성을 fallback으로 사용한다.
- padding을 고려한 사용 가능 미리보기 영역을 기준으로 fit zoom을 계산한다.
- Mermaid 렌더링이 성공할 때마다 계산된 fit zoom을 적용한다.
- 미리보기 영역 크기가 변경되면 resize 관찰 전략을 사용해 fit을 다시 계산한다.
- 자동 fit이 적용될 때 pan을 초기화해 다이어그램이 중앙에 유지되도록 한다.
- 수동 zoom 버튼과 fit 버튼은 유지한다.
- 현재 오류 상태와 빈 source 상태를 유지한다.

## 헌법 체크 - 설계 후

- **모노레포/Tauri 구조**: PASS. 계획된 소스 변경은 `apps/desktop` 내부에 머문다.
- **프론트엔드 Feature-Sliced Design**: PASS. 기능 로직은 `features`에 남고, 저수준 버튼 프리미티브는 `shared`에 남는다.
- **네이티브 hexagonal architecture**: PASS. 네이티브 코드 변경은 없다.
- **UI 규칙**: PASS. 기존 아이콘 버튼과 접근성 label을 유지한다.

## 복잡도 추적

헌법 위반이나 추가 아키텍처 복잡도는 도입하지 않는다.
