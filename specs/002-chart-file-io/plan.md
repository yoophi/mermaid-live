# 구현 계획: Mermaid 차트 파일 열기 및 저장

**브랜치**: `002-chart-file-io` | **날짜**: 2026-08-01 | **명세**: [spec.md](./spec.md)
**입력**: `/specs/002-chart-file-io/spec.md`의 기능 명세

**참고**: 이 문서는 `/speckit.plan` 명령으로 작성되었으며 `.specify/templates/plan-template.md`의 흐름을 따른다.

## 요약

현재 편집기를 단일 차트 문서 작업 공간으로 확장해 `.mmd`와 `.mermaid` 파일을 열고, 최초 저장·일반 저장·다른 이름으로 저장을 지원한다. 프론트엔드는 문서 원문, 저장 기준 원문, 파일 연결, dirty 상태를 하나의 문서 모델로 관리한다. 네이티브 영역은 확장자 정책, 내용 지문 기반 외부 변경 판정, 조건부 저장, 안전한 파일 교체를 도메인·애플리케이션 계층에 두고 파일 선택창과 로컬 파일시스템을 outbound 어댑터로 격리한다. 메뉴 이벤트와 창 닫기 요청은 inbound 어댑터가 프론트엔드 워크플로에 전달한다.

## 기술 컨텍스트

**언어/버전**: TypeScript 5.9, React 19, Rust 2021 edition(현재 Rust 1.95 toolchain)  
**주요 의존성**: Tauri 2, `@tauri-apps/api` 2, `tauri-plugin-dialog` 2.7, CodeMirror 6, 기존 shadcn/ui 스타일 프리미티브, `sha2` 0.10, `tempfile` 3, Vitest 4  
**저장소**: 사용자가 선택한 로컬 `.mmd`, `.mermaid` UTF-8 파일; 서버나 데이터베이스 없음  
**테스트**: Rust 단위·임시 디렉터리 통합 테스트, 문서 상태 모델용 Vitest 도입, `pnpm typecheck`, `pnpm build`, `cargo test --locked`, Tauri 수동 검증  
**대상 플랫폼**: Tauri 데스크톱 앱(macOS 우선, Windows/Linux 메뉴 단축키와 파일 교체 의미도 보존)  
**프로젝트 유형**: pnpm 모노레포의 Tauri 데스크톱 앱  
**성능 목표**: 대표 로컬 파일을 선택한 뒤 3초 이내 편집 가능, 저장 요청 후 사용자 확인을 제외하고 1초 이내 완료, 편집 입력 중 문서 상태 갱신으로 체감 지연 없음  
**제약 조건**: 오프라인 동작, UTF-8 원문·줄바꿈 비변형, `.mmd`/`.mermaid`만 허용, 현재 활성 편집기 교체, 저장 성공 전에 baseline·파일 연결 갱신 금지, 미저장 내용과 기존 파일의 조용한 손실 금지  
**범위**: 단일 차트 문서 워크플로 1개, 기존 다중 창/탭과 클립보드 임시 문서 흐름 유지; 자동 저장·복구본·최근 파일·드래그 앤 드롭·OS 파일 연결 제외

## 헌법 체크

*게이트: Phase 0 리서치 전에 통과해야 하며 Phase 1 설계 후 다시 확인한다.*

프로젝트 constitution은 아직 플레이스홀더 상태이므로 강제 가능한 별도 원칙은 없다. 저장소 `AGENTS.md`의 규칙을 실행 게이트로 적용한다.

- **pnpm 모노레포/Tauri 구조**: PASS. 변경은 기존 `apps/desktop` 패키지와 루트 명령 안에 유지한다.
- **프론트엔드 Feature-Sliced Design**: PASS. 문서 개체는 `entities`, 열기·저장·충돌·닫기 조정은 하나의 `features` 워크플로, 화면 조합은 `pages`에 둔다.
- **네이티브 hexagonal architecture**: PASS. 파일 규칙은 `domain`, use case와 port는 `application`, Tauri command/menu는 `adapters/inbound`, dialog와 파일시스템은 `adapters/outbound`, wiring은 `infrastructure`에 둔다.
- **UI 규칙과 접근성**: PASS. 파일 선택은 네이티브 dialog를 재사용하고 세 방향 결정 dialog는 명확한 버튼 라벨과 키보드 조작을 제공한다.
- **데이터 안전성**: PASS. 조건부 저장과 임시 sibling 파일 교체로 외부 변경 및 부분 쓰기로 인한 조용한 손실을 방지한다.

## 프로젝트 구조

### 문서 구조(이 기능)

```text
specs/002-chart-file-io/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── document-file-commands.md
│   └── document-menu-ui.md
└── tasks.md                    # /speckit.tasks에서 생성
```

### 소스 코드(저장소 루트)

```text
apps/desktop/
├── package.json
├── src/
│   ├── entities/
│   │   └── chart-document/
│   │       ├── index.ts
│   │       └── model/
│   │           ├── chart-document.ts
│   │           └── chart-document.test.ts
│   ├── features/
│   │   └── manage-chart-document/
│   │       ├── api/chart-file.ts
│   │       ├── model/use-chart-document.ts
│   │       └── index.ts
│   └── pages/editor/ui/editor-page.tsx
└── src-tauri/
    ├── Cargo.toml
    └── src/
        ├── domain/
        │   └── chart_document.rs
        ├── application/
        │   ├── ports.rs
        │   └── use_cases.rs
        ├── adapters/
        │   ├── inbound/
        │   │   ├── tauri_commands.rs
        │   │   └── window_menu.rs
        │   └── outbound/
        │       ├── chart_file_dialog.rs
        │       └── local_chart_file_repository.rs
        └── infrastructure/
            ├── app_state.rs
            ├── document_window_lifecycle.rs
            └── window_lifecycle.rs
```

**구조 결정**: 현재 `features/save-diagram`과 `diagram_file_saver.rs`의 책임을 확장하지 않고, 열기·저장·dirty 보호·충돌 해결이 공유하는 상태와 후속 작업을 `manage-chart-document` 워크플로로 통합한다. 기존 단순 save hook과 saver는 새 흐름이 기능을 흡수한 뒤 제거한다. 클립보드에서 생성한 임시 파일은 원문 bootstrap 용도로만 읽고 사용자 파일 연결로 취급하지 않는다.

## Phase 0: 리서치

[research.md](./research.md)의 결정을 따른다.

- 문서 상태의 권위는 편집 입력과 같은 webview의 프론트엔드 문서 entity에 둔다.
- 네이티브 저장 use case는 예상 파일 revision을 받는 조건부 저장으로 설계한다.
- 외부 변경은 수정 시각만으로 판단하지 않고 파일 바이트의 SHA-256 내용 지문으로 판정한다.
- 원본 파일을 직접 truncate하지 않고 같은 디렉터리의 임시 파일을 완성한 뒤 교체한다.
- 창 닫기는 native close 요청을 일단 보류하고 프론트엔드 문서 워크플로의 승인 후 한 번만 통과시키는 handshake로 처리한다.
- 파일 선택과 세 방향 확인은 설치된 native dialog 기능을 adapter에서 사용한다.
- 상태 전이는 순수 모델 테스트, 파일 규칙과 저장은 Rust fake/tempdir 테스트로 검증한다.

## Phase 1: 설계

[data-model.md](./data-model.md), [contracts/document-file-commands.md](./contracts/document-file-commands.md), [contracts/document-menu-ui.md](./contracts/document-menu-ui.md), [quickstart.md](./quickstart.md)를 따른다.

설계 흐름:

1. 메뉴의 열기/저장/다른 이름으로 저장 요청을 활성 webview에 전달한다.
2. 프론트엔드 문서 controller가 dirty 상태를 확인하고 필요한 사용자 결정을 먼저 얻는다.
3. open command는 파일 dialog, 확장자 검증, UTF-8 읽기와 revision 계산을 수행해 snapshot을 반환한다.
4. save command는 연결 파일의 예상 revision과 현재 디스크 revision을 비교한다. 같으면 안전하게 교체 저장하고 새 revision을 반환하며, 다르면 conflict snapshot을 반환한다.
5. conflict에서 다시 불러오기는 disk snapshot을 적용하고, 덮어쓰기는 명시적 force 요청으로 저장하며, 취소는 상태를 바꾸지 않는다.
6. 성공 결과를 받은 뒤에만 문서 baseline과 파일 연결을 갱신하고 창 제목을 동기화한다.
7. 닫기 요청은 clean 문서 또는 저장/폐기 승인을 받은 dirty 문서만 one-shot close authorization을 통해 완료한다.

## 헌법 체크 - 설계 후

- **pnpm 모노레포/Tauri 구조**: PASS. 새 테스트 의존성도 기존 desktop workspace package에만 추가한다.
- **프론트엔드 Feature-Sliced Design**: PASS. entity는 순수 문서 상태만, feature는 사용자 workflow만, page는 조합만 담당한다.
- **네이티브 hexagonal architecture**: PASS. Tauri와 `std::fs` 타입은 domain/application 내부로 유입되지 않으며 port DTO로 변환된다.
- **UI 규칙과 접근성**: PASS. 운영체제 dialog와 표준 단축키를 사용하고 모든 파괴적 선택에 취소 경로가 있다.
- **데이터 안전성**: PASS. 충돌 또는 쓰기 실패에서는 baseline, 파일 연결, 현재 편집 원문을 갱신하지 않는다.

## 복잡도 추적

헌법 위반은 없다. 문서 controller, 조건부 저장, one-shot close authorization은 각각 여러 독립 기능이 아니라 명세가 요구하는 저장 손실 방지를 위한 최소 상태 조정 장치다.
