# 빠른 시작 및 검증: Mermaid 차트 파일 열기 및 저장

## 준비

저장소 루트에서 의존성을 설치한다.

```sh
pnpm install
```

테스트용 임시 디렉터리에 다음 파일을 준비한다.

- `valid.mmd`: 유효한 Mermaid 원문
- `valid.mermaid`: 유효한 Mermaid 원문
- `invalid.mmd`: 문법 오류가 있는 Mermaid 원문
- `empty.mmd`: 빈 파일
- `unsupported.md`: 지원하지 않는 확장자

## 자동 검증

```sh
pnpm typecheck
pnpm --filter @mermaid-live/desktop test
cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml -- --check
cargo test --locked --manifest-path apps/desktop/src-tauri/Cargo.toml
pnpm build
pnpm tauri build
```

핵심 자동 테스트:

- 제목 없는 문서의 편집 → dirty → 최초 저장 성공 → clean 전이
- 열린 문서의 편집 → 일반 저장 성공 → baseline/revision 갱신
- 취소·읽기 실패·쓰기 실패에서 문서 상태 불변
- `.mmd`, `.mermaid` 허용, 확장자 없음에 `.mmd` 적용, 다른 확장자 거부
- UTF-8 원문과 줄바꿈의 exact round trip
- disk 내용 변경 시 conflict 반환과 쓰기 없음
- 강제 덮어쓰기 성공 후 새 revision 반환
- 안전 저장 실패 시 기존 파일 내용 보존
- metadata만 달라지고 내용이 같을 때 비충돌
- one-shot close authorization의 window 격리와 단일 소비

## 수동 실행

```sh
pnpm tauri dev
```

## 사용자 시나리오 검증

### 새 문서 저장

1. 새 편집기에서 원문을 수정한다.
2. `Cmd/Ctrl+S`를 누른다.
3. 확장자 없이 파일명을 입력한다.
4. `.mmd`가 붙어 저장되고 창 제목이 파일명으로 바뀌며 dirty 표시가 사라지는지 확인한다.
5. 다시 수정하고 저장해 경로 선택 없이 같은 파일이 갱신되는지 확인한다.

### 파일 열기와 다른 이름으로 저장

1. `Cmd/Ctrl+O`로 `valid.mermaid`를 연다.
2. 현재 편집기 내용과 창 제목이 선택 파일로 바뀌는지 확인한다.
3. `Cmd/Ctrl+Shift+S`로 새 `.mmd` 경로에 저장한다.
4. 이후 일반 저장이 새 경로를 갱신하는지 확인한다.
5. `invalid.mmd`와 `empty.mmd`도 열리며 기존 preview 오류/빈 상태가 표시되는지 확인한다.

### 미저장 변경 보호

1. 문서를 수정한 뒤 다른 파일 열기와 window 닫기를 각각 시도한다.
2. `저장`, `저장 안 함`, `취소`가 모두 표시되는지 확인한다.
3. 저장 성공 시 원래 작업이 이어지는지 확인한다.
4. 저장 dialog 취소 또는 저장 실패 시 원래 작업도 중단되는지 확인한다.
5. 저장 안 함은 변경을 폐기하고, 취소는 현재 원문과 window를 유지하는지 확인한다.

### 외부 변경 충돌

1. 앱에서 `valid.mmd`를 연 뒤 원문을 수정한다.
2. 다른 편집기에서 같은 파일을 다른 내용으로 저장한다.
3. 앱에서 저장하고 `다시 불러오기`, `덮어쓰기`, `취소` 선택을 각각 검증한다.
4. 어떤 선택도 하기 전에는 disk 파일이 변경되지 않는지 확인한다.
5. 다시 불러오기는 외부 내용을 clean 상태로 적용하고, 덮어쓰기는 앱 내용을 저장하며, 취소는 앱의 dirty 내용을 유지하는지 확인한다.

### 오류와 형식 제한

1. `unsupported.md`가 열기 대상에서 제외되는지 확인한다.
2. Save As에서 명시적으로 지원하지 않는 확장자를 입력했을 때 저장이 차단되는지 확인한다.
3. 읽기 전용 경로 저장과 open 후 파일 삭제 상황에서 오류가 표시되고 현재 편집 내용이 유지되는지 확인한다.

## 완료 기준

- 명세의 SC-001부터 SC-007까지 검증할 수 있다.
- 자동 테스트와 빌드가 모두 통과한다.
- 기존 새 창, 새 탭, 창 합치기, 클립보드 chart bootstrap, 미리보기 동작에 회귀가 없다.

## 구현 검증 기록 (2026-08-01)

- `pnpm typecheck`: 통과
- `pnpm --filter @mermaid-live/desktop test`: 통과 (테스트 파일 2개, 테스트 4개)
- `cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml -- --check`: 통과
- `cargo test --locked --manifest-path apps/desktop/src-tauri/Cargo.toml`: 통과 (테스트 18개)
- `pnpm build`: 통과
- `pnpm tauri build`: 통과 (`.app`, `.dmg` 생성)
- 릴리스 실행 파일 스모크 기동: 통과
- 네이티브 파일 선택·저장 및 확인 대화상자를 직접 조작하는 수동 사용자 시나리오: 미실행

참고: 추가로 실행한 `pnpm lint`는 프로젝트의 ESLint 설정에 TypeScript/TSX 파서가 연결되어 있지 않아 기존 소스를 포함한 모든 TypeScript 파일에서 parsing error로 실패했다. 위 완료 기준의 필수 검증 명령에는 포함되지 않는다.
