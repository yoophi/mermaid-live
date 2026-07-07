# 작업 목록: Mermaid Zoom Fit

**입력**: `/specs/001-mermaid-zoom-fit/`의 설계 문서  
**선행 문서**: plan.md, spec.md, research.md, data-model.md, contracts/mermaid-preview-ui.md, quickstart.md  
**테스트 방침**: 별도 TDD 요청이나 프론트엔드 테스트 러너 설정이 없으므로 자동 테스트 추가 대신 `pnpm typecheck`, `pnpm build`, quickstart 수동 검증으로 확인한다.

**구성 방식**: 각 작업은 사용자 스토리별로 묶어 독립 구현과 독립 검증이 가능하도록 한다.

## 형식: `[ID] [P?] [Story] 설명`

- **[P]**: 서로 다른 파일을 다루며 미완료 작업에 의존하지 않아 병렬 실행 가능
- **[Story]**: 사용자 스토리 작업에만 사용한다. 예: [US1], [US2], [US3]
- 모든 작업 설명에는 실제 파일 경로를 포함한다

## 단계 1: 설정

**목적**: 구현 전에 현재 구조와 참조 동작을 확인한다.

- [X] T001 [P] AW 참조 구현의 fit CSS와 zoom 처리 방식을 `/Users/yoophi/project/agentic-workspace/packages/markdown-annotation-react/src/styles.css` 및 `/Users/yoophi/project/agentic-workspace/apps/agentic-workbench/src/features/agent-run/ui/agent-run-markdown.tsx`에서 확인하고 적용할 동작을 `specs/001-mermaid-zoom-fit/research.md` 기준으로 정리한다
- [X] T002 [P] 현재 Mermaid 미리보기의 렌더링, zoom, pan, fit 버튼 흐름을 `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 확인하고 변경 지점을 표시한다
- [X] T003 [P] 검증 명령과 수동 확인 절차를 `specs/001-mermaid-zoom-fit/quickstart.md` 기준으로 확인하고 구현 후 실행할 항목을 준비한다

---

## 단계 2: 기반 작업

**목적**: 모든 사용자 스토리가 공유하는 fit 계산과 SVG 표시 기반을 정리한다.

**중요**: 이 단계가 끝나야 사용자 스토리별 구현을 안정적으로 진행할 수 있다.

- [X] T004 `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 SVG 원본 크기 측정 로직이 `viewBox`를 우선 사용하고 width/height 속성을 fallback으로 쓰도록 `readSvgBaseSize`를 점검 및 보강한다
- [X] T005 `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 표시 영역 크기가 0이거나 측정 불가능한 경우 0 크기 또는 숨김 상태로 고착되지 않도록 `getElementSize`와 fit fallback 처리를 보강한다
- [X] T006 `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 자동 fit과 수동 fit이 같은 계산 경로를 사용하도록 fit 계산 및 적용 함수를 정리한다
- [X] T007 `apps/desktop/src/app/styles/global.css`에서 Mermaid SVG가 컨테이너 크기와 원본 비율을 안정적으로 따르도록 `.mermaid-preview svg` 규칙을 AW 참조 방식에 맞춰 점검 및 보강한다

**체크포인트**: SVG 원본 크기, 표시 영역 크기, fit zoom 계산이 하나의 일관된 경로로 준비되어야 한다.

---

## 단계 3: 사용자 스토리 1 - 차트가 처음부터 적정 크기로 보임 (우선순위: P1) MVP

**목표**: Mermaid 차트가 최초 표시될 때 현재 표시 영역 안에 잘리지 않고 적정 크기로 자동 표시된다.

**독립 검증**: `specs/001-mermaid-zoom-fit/quickstart.md`의 작은/넓은/높은 차트 샘플을 표시했을 때 최초 렌더링 상태에서 전체 차트가 보이고 중앙에 위치해야 한다.

### 구현

- [X] T008 [US1] `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 Mermaid 렌더링 성공 직후 SVG 측정이 완료된 다음 자동 fit zoom을 적용하도록 `useLayoutEffect` 흐름을 수정한다
- [X] T009 [US1] `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 자동 fit 적용 시 `pan`을 `{ x: 0, y: 0 }`으로 초기화해 최초 표시가 중앙에 오도록 한다
- [X] T010 [US1] `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 작은 차트가 불필요하게 축소되지 않도록 `getFitZoom`의 확대/축소 정책을 요구사항 FR-003, FR-004에 맞게 조정한다
- [X] T011 [US1] `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 base size가 준비되기 전 visibility 처리로 인해 차트가 영구적으로 숨겨지지 않도록 렌더링 상태 전환을 점검한다
- [ ] T012 [US1] `specs/001-mermaid-zoom-fit/quickstart.md`의 작은/넓은/높은 Mermaid 샘플로 최초 렌더링 fit을 수동 검증하고 결과를 구현 메모에 반영한다

**체크포인트**: 사용자 스토리 1만 구현해도 대표 Mermaid 차트가 최초 표시 시 잘리지 않고 적정 크기로 보인다.

---

## 단계 4: 사용자 스토리 2 - 현재 표시 방식이 유지됨 (우선순위: P2)

**목표**: 차트 크기 개선을 적용하되 기존 인라인 미리보기 방식, zoom 컨트롤, pan 동작, 오류 표시 흐름은 유지한다.

**독립 검증**: 기존 Mermaid 미리보기 진입 방식으로 차트를 열었을 때 전체 화면 모달이나 별도 화면 전환이 생기지 않고, 기존 버튼과 오류 상태가 계속 동작해야 한다.

### 구현

- [X] T013 [US2] `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 `Maximize2`, `Minus`, `Plus` 버튼 UI와 기존 fit 버튼 동작을 유지한 채 자동 fit 초기화와 충돌하지 않도록 이벤트 흐름을 점검한다
- [X] T014 [US2] `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 수동 zoom in/out 후 사용자가 조정한 zoom과 pan이 즉시 자동 fit에 의해 덮어써지지 않도록 상태 전환 조건을 보강한다
- [X] T015 [US2] `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 빈 source와 Mermaid 렌더링 실패 시 기존 empty/error UI가 유지되고 fit 계산이 실행되지 않도록 예외 흐름을 점검한다
- [X] T016 [US2] `apps/desktop/src/widgets/diagram-workspace/ui/diagram-workspace.tsx`에서 `MermaidPreview` 호출 방식이 기존 인라인 표시 흐름을 유지하는지 확인하고 불필요한 모달/라우팅 변경이 없음을 검증한다
- [ ] T017 [US2] `specs/001-mermaid-zoom-fit/contracts/mermaid-preview-ui.md`의 Manual Controls 및 Error And Empty States 항목을 기준으로 기존 표시 방식 유지 여부를 수동 검증한다

**체크포인트**: 사용자 스토리 1과 2가 함께 적용되어도 사용자는 기존 화면 흐름 그대로 개선된 차트를 확인할 수 있다.

---

## 단계 5: 사용자 스토리 3 - 표시 영역 변화에 맞게 다시 맞춰짐 (우선순위: P3)

**목표**: 앱 창이나 미리보기 영역 크기가 바뀌면 Mermaid 차트가 새 영역에 맞춰 1초 이내에 다시 fit된다.

**독립 검증**: 차트가 표시된 상태에서 앱 창 또는 소스/미리보기 분할 크기를 바꾸면 차트가 새 표시 영역 안에 들어오고 중앙에 유지되어야 한다.

### 구현

- [X] T018 [US3] `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 `ResizeObserver` 기반으로 `viewportRef` 크기 변경을 감지하는 effect를 추가한다
- [X] T019 [US3] `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 표시 영역 크기 변경 시 현재 SVG base size를 재사용하거나 재측정한 뒤 자동 fit zoom을 다시 적용하도록 구현한다
- [X] T020 [US3] `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 resize 기반 자동 fit이 사용자의 진행 중인 drag/pan 동작과 충돌하지 않도록 `dragStartRef` 상태를 고려한다
- [X] T021 [US3] `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`에서 `ResizeObserver` cleanup을 추가해 컴포넌트 언마운트 또는 source 변경 시 observer가 누수되지 않도록 한다
- [ ] T022 [US3] `specs/001-mermaid-zoom-fit/quickstart.md`의 resize 검증 절차로 앱 창 및 패널 크기 변경 후 1초 이내 refit 여부를 수동 검증한다

**체크포인트**: 모든 사용자 스토리가 독립적으로 동작하며, 최초 렌더링과 영역 변경 모두에서 차트가 적정 크기로 유지된다.

---

## 단계 6: 마무리 및 교차 검증

**목적**: 품질 검증과 문서 정합성을 확인한다.

- [X] T023 [P] `specs/001-mermaid-zoom-fit/spec.md`의 FR-001부터 FR-010까지 구현 결과와 누락 여부를 대조한다
- [ ] T024 [P] `specs/001-mermaid-zoom-fit/contracts/mermaid-preview-ui.md`의 Initial Render, Large Diagram, Small Diagram, Resize, Manual Controls, Error And Empty States 계약을 수동 검증한다
- [X] T025 `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`와 `apps/desktop/src/app/styles/global.css`의 변경 범위를 정리하고 불필요한 리팩터링이나 모달 도입이 없는지 확인한다
- [X] T026 `package.json` 기준으로 `pnpm typecheck`를 실행해 TypeScript 타입 검사를 통과시킨다
- [X] T027 `package.json` 기준으로 `pnpm build`를 실행해 프로덕션 빌드를 통과시킨다
- [ ] T028 `specs/001-mermaid-zoom-fit/quickstart.md`의 전체 수동 검증 절차를 완료하고 발견 사항을 구현 결과에 반영한다

---

## 의존성 및 실행 순서

### 단계 의존성

- **설정(단계 1)**: 즉시 시작 가능
- **기반 작업(단계 2)**: 설정 완료 후 진행하며 모든 사용자 스토리를 막는 공통 선행 단계
- **사용자 스토리 1(단계 3)**: 단계 2 완료 후 시작. MVP 범위
- **사용자 스토리 2(단계 4)**: 단계 2 완료 후 시작 가능하나, 실제 검증은 US1의 자동 fit 동작 위에서 수행하는 것이 가장 명확함
- **사용자 스토리 3(단계 5)**: 단계 2 완료 후 시작 가능하나, US1의 fit 계산 경로를 재사용함
- **마무리(단계 6)**: 구현하려는 모든 사용자 스토리 완료 후 진행

### 사용자 스토리 의존성

- **US1 (P1)**: 단계 2 이후 독립 구현 가능. MVP
- **US2 (P2)**: 단계 2 이후 독립 점검 가능. US1의 변경이 기존 흐름을 깨지 않았는지 확인
- **US3 (P3)**: 단계 2 이후 구현 가능. US1의 fit 계산 함수를 resize 시점에 재사용

### 사용자 스토리 내부 순서

- 공통 fit 계산 기반을 먼저 정리한다
- 렌더링 완료 후 자동 fit을 적용한다
- 기존 UI/오류 흐름 보존을 확인한다
- resize observer를 추가하고 cleanup을 확인한다
- quickstart 기준으로 독립 검증한다

### 병렬 실행 가능 지점

- T001, T002, T003은 서로 다른 문서/참조 파일 확인 작업이므로 병렬 가능
- T023, T024는 서로 다른 문서 기준 검증이므로 병렬 가능
- US2의 `diagram-workspace.tsx` 확인(T016)은 US1 구현과 파일 충돌 없이 병렬 검토 가능

---

## 병렬 실행 예시: 사용자 스토리 1

```bash
Task: "apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx에서 렌더링 성공 후 자동 fit 적용 흐름 수정"
Task: "specs/001-mermaid-zoom-fit/quickstart.md의 작은/넓은/높은 Mermaid 샘플 준비"
```

## 병렬 실행 예시: 사용자 스토리 2

```bash
Task: "apps/desktop/src/widgets/diagram-workspace/ui/diagram-workspace.tsx에서 MermaidPreview 호출 방식 확인"
Task: "specs/001-mermaid-zoom-fit/contracts/mermaid-preview-ui.md 기준으로 수동 컨트롤 계약 검증"
```

## 병렬 실행 예시: 사용자 스토리 3

```bash
Task: "apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx에서 ResizeObserver cleanup 구현"
Task: "specs/001-mermaid-zoom-fit/quickstart.md에서 resize 수동 검증 절차 준비"
```

---

## 구현 전략

### MVP 우선(사용자 스토리 1)

1. 단계 1 설정 작업을 완료한다.
2. 단계 2 기반 작업을 완료한다.
3. 단계 3 사용자 스토리 1을 구현한다.
4. `quickstart.md`의 작은/넓은/높은 차트 샘플로 최초 fit을 독립 검증한다.
5. 이 시점에서 사용자는 가장 중요한 개선인 "처음부터 적정 크기로 보이는 차트"를 확인할 수 있다.

### 점진적 전달

1. 설정과 기반 작업 완료 후 fit 계산 기반을 안정화한다.
2. US1을 적용해 최초 표시 품질을 개선한다.
3. US2를 적용해 기존 표시 방식과 수동 조작 흐름을 보존한다.
4. US3을 적용해 창/패널 크기 변경에 대응한다.
5. 단계 6에서 typecheck, build, quickstart 전체 검증을 수행한다.

### 다중 작업자 전략

1. 한 작업자가 `mermaid-preview.tsx`의 공통 fit 계산 경로를 담당한다.
2. 다른 작업자가 `contracts/mermaid-preview-ui.md`와 `quickstart.md` 기준의 수동 검증 케이스를 준비한다.
3. 공통 기반이 끝난 뒤 한 작업자는 US2 보존 검증, 다른 작업자는 US3 resize 처리를 진행할 수 있다.

## 참고

- [P] 작업은 서로 다른 파일을 다루거나 직접 의존성이 없어 병렬 가능한 작업이다.
- [US1], [US2], [US3] 라벨은 스펙의 사용자 스토리와 연결된다.
- 모든 사용자 스토리는 독립적으로 완료 및 검증 가능해야 한다.
- 모달 전환, 별도 화면 이동, 네이티브 코드 변경은 이번 범위에 포함하지 않는다.
