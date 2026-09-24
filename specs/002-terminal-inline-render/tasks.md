# 작업 목록: Terminal Inline Render

**입력**: `/specs/002-terminal-inline-render/`의 설계 문서  
**선행 문서**: plan.md, spec.md, research.md, data-model.md, contracts/render-socket.md, quickstart.md  
**테스트 방침**: 네이티브와 CLI 모두 인라인 `#[cfg(test)]` 관례에 따라 단위 테스트를 둔다(`cargo test`). 프론트엔드 테스트 러너는 설정되어 있지 않으므로 `pnpm typecheck`, `pnpm lint`로 확인한다. 인라인 이미지 표시는 실제 tty가 필요해 quickstart 수동 검증으로 확인한다.

**구성 방식**: 각 작업은 사용자 스토리별로 묶어 독립 구현과 독립 검증이 가능하도록 한다.

> **기록 주의**: 단계 1~6 은 최초 구현 시점의 기록이며 당시 CLI 는 `apps/cli` 의 TypeScript 패키지였다. 단계 7 에서 Rust 로 이관했으므로 현재 경로는 `crates/mmdcat` 이다. 완료된 작업의 경로는 당시 사실로 남겨 둔다.

## 형식: `[ID] [P?] [Story] 설명`

- **[P]**: 서로 다른 파일을 다루며 미완료 작업에 의존하지 않아 병렬 실행 가능
- **[Story]**: 사용자 스토리 작업에만 사용한다. 예: [US1], [US2], [US3]
- 모든 작업 설명에는 실제 파일 경로를 포함한다

## 단계 1: 설정

**목적**: 구현 전에 재사용 지점을 확정하고 신규 패키지 골격을 만든다.

- [X] T001 [P] `apps/cli/package.json`과 `apps/cli/tsconfig.json`을 만들어 `@mermaid-live/cli` 패키지를 스캐폴딩하고 실행명 `mmdcat`을 bin으로 노출한다. `pnpm-workspace.yaml`의 `apps/*` 글롭에 이미 포함되므로 워크스페이스 설정은 변경하지 않는다
- [X] T002 [P] 재사용 지점을 확인하고 변경 지점을 표시한다: `apps/desktop/src-tauri/src/domain/mermaid_chart.rs`의 `extract_mermaid_chart_source()`, `apps/desktop/src-tauri/src/adapters/outbound/native_window_manager.rs`의 `prewarm_temp_diagram_window()`와 `build_editor_window_with_url()`, `apps/desktop/src-tauri/src/adapters/outbound/temp_diagram_file.rs`, `apps/desktop/src/pages/editor/ui/editor-page.tsx`의 `sourceFile` 파라미터 규약
- [X] T003 [P] `specs/002-terminal-inline-render/quickstart.md`의 수동 검증 절차와 터미널별 설정을 확인하고 구현 후 실행할 항목을 준비한다

---

## 단계 2: 기반 작업

**목적**: 렌더 서비스의 계층 구조와 프론트엔드 래스터화 표면을 준비한다.

**중요**: 이 단계가 끝나야 CLI 쪽 사용자 스토리 구현을 안정적으로 진행할 수 있다.

- [X] T004 `apps/desktop/src/shared/lib/mermaid-config.ts`를 만들어 `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`의 `mermaid.initialize` 설정(theme `base`, `themeVariables`, `securityLevel`)을 추출하고, 미리보기가 이 공유 설정을 사용하도록 수정한다. 래스터화 전용 변형(`flowchart: { htmlLabels: false }`)을 같은 모듈에서 파생 형태로 제공한다
- [X] T005 [P] `apps/desktop/src-tauri/src/domain/chart_raster.rs`를 만들어 `RasterSpec`(width_px, height_px, scale)과 `RasterError`를 정의하고 `apps/desktop/src-tauri/src/domain/mod.rs`에 등록한다. Tauri 타입을 포함하지 않는다
- [X] T006 `apps/desktop/src-tauri/src/application/ports.rs`에 `ChartRasterizer` 포트를 추가한다. `DetectedChart`와 `RasterSpec`을 받아 PNG 바이트를 돌려주는 시그니처로 정의한다
- [X] T007 `apps/desktop/src-tauri/src/infrastructure/render_service.rs`를 만들어 진행 중인 렌더 요청 레지스트리(requestId → 응답 채널)를 구현하고 `apps/desktop/src-tauri/src/infrastructure/mod.rs`에 등록한다
- [X] T008 `apps/desktop/src/pages/rasterize/`에 래스터화 화면을 만들어 쿼리 파라미터(`sourceFile`, `w`, `h`, `scale`, `requestId`)를 읽고, 공유 설정으로 `mermaid.render`를 수행한 뒤 SVG를 canvas로 옮겨 PNG를 만들고 `deliver_chart_png` 커맨드로 결과를 돌려준다
- [X] T009 `apps/desktop/src/app/App.tsx`에서 `rasterize=1` 파라미터가 있으면 래스터화 화면으로 분기하도록 수정한다. 기존 에디터 흐름은 그대로 유지한다
- [X] T010 `apps/desktop/src-tauri/src/adapters/inbound/tauri_commands.rs`에 `deliver_chart_png(request_id, png_base64, error)` 커맨드를 추가하고 `apps/desktop/src-tauri/src/lib.rs`의 `invoke_handler`에 등록한다
- [X] T011 `apps/desktop/src-tauri/src/adapters/outbound/webview_rasterizer.rs`를 만들어 `ChartRasterizer`를 구현한다. `temp_diagram_file::write_temp_diagram_file()`로 `.mmd`를 저장하고 `build_editor_window_with_url()` 패턴으로 숨은 창을 만들어 래스터화 화면을 띄운 뒤 레지스트리 채널로 결과를 기다린다. 타임아웃 시 창을 정리한다
- [X] T012 `apps/desktop/src-tauri/src/application/use_cases.rs`에 `RenderChartToPng` 유스케이스를 추가한다. 원문 → `extract_mermaid_chart_source()` → `DetectedChart` → fingerprint와 목표 크기 기준 캐시 조회 → 미스 시 포트 위임 순서로 동작하고, 차트가 아니면 그 사실을 구분해 돌려준다
- [X] T013 `apps/desktop/src-tauri/src/adapters/inbound/render_socket.rs`를 만들어 `$TMPDIR/mermaid-live/render.sock` 리스너를 구현한다. `std::os::unix::net::UnixListener`만 사용하고, 시작 시 stale 소켓을 정리하며, `contracts/render-socket.md`의 요청·응답 스키마를 따른다. `apps/desktop/src-tauri/src/adapters/inbound/mod.rs`에 등록한다
- [X] T014 `apps/desktop/src-tauri/src/infrastructure/app_state.rs`에 `RenderChartToPng`를 배선하고 `apps/desktop/src-tauri/src/lib.rs`의 `setup()`에서 `clipboard_watcher::start` 옆에 소켓 리스너를 기동한다
- [X] T015 [P] `apps/desktop/src-tauri/src/application/use_cases.rs`와 `apps/desktop/src-tauri/src/adapters/inbound/render_socket.rs`에 단위 테스트를 추가한다. 차트 아님 판별, 캐시 히트, 요청 JSON 파싱 실패 처리를 다룬다

**체크포인트**: 앱만 띄운 상태에서 소켓에 요청을 보내면 PNG 또는 `notAChart` 응답이 돌아와야 한다.

---

## 단계 3: 사용자 스토리 1 - 터미널에서 선택한 차트를 그 자리에서 확인 (우선순위: P1) MVP

**목표**: 터미널에서 Mermaid 텍스트를 선택하고 단축키를 누르면 같은 화면에 차트가 인라인으로 표시된다.

**독립 검증**: `specs/002-terminal-inline-render/quickstart.md`의 Manual Verification 1·2·3번을 수행해 차트가 잘리지 않고 표시되고 라벨이 보이며 프롬프트가 유지되는지 확인한다.

### 구현

- [X] T016 [P] [US1] `apps/cli/src/input.ts`에 입력 해석을 구현한다. 인자가 없으면 `pbpaste`, `-`면 stdin, 그 외는 파일 경로로 읽는다
- [X] T017 [P] [US1] `apps/cli/src/terminal/capability.ts`에 kitty graphics 지원 탐지를 구현한다. raw 모드에서 graphics 질의와 DA1을 함께 보내고 짧은 타임아웃으로 응답을 판정한다
- [X] T018 [P] [US1] `apps/cli/src/terminal/metrics.ts`에 셀 픽셀 크기 질의를 구현한다. `CSI 16 t`를 먼저 시도하고 `CSI 14 t`를 폴백으로 쓰며, 둘 다 실패하면 미확정 상태를 돌려준다
- [X] T019 [P] [US1] `apps/cli/src/fit.ts`에 표시 크기 계산을 구현한다. `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx`의 `getFitZoom()` 규칙(패딩 적용, 폭·높이 비율 중 작은 값, 최소·최대 클램프)을 셀 단위로 이식하고 프롬프트 공간을 남긴다
- [X] T020 [US1] `apps/cli/src/terminal/kitty-graphics.ts`에 프로토콜 출력을 구현한다. `a=T,f=100,t=d,q=2`와 계산된 `c`·`r`을 첫 청크에 담고, base64를 4096바이트 이하(마지막 외 4의 배수)로 나누어 이후 청크는 `m` 키만 포함하며 `m=0`으로 끝낸다
- [X] T021 [US1] `apps/cli/src/render-client.ts`에 소켓 클라이언트를 구현한다. `contracts/render-socket.md`의 요청을 보내고 응답을 파싱한다
- [X] T022 [US1] `apps/cli/src/main.ts`에 실행 흐름을 조립한다. 입력 해석 → 지원 탐지 → 크기 계산 → 렌더 요청 → 프로토콜 출력 순서로 연결한다
- [X] T023 [US1] `apps/cli/shell/mmdcat.zsh`에 ZLE 위젯을 구현하고 비어 있는 `^Xv`에 바인딩한다(`^Xm`은 zsh 기본 `_most_recent_file`). 이미지는 `/dev/tty`로 출력하고 `zle reset-prompt`로 프롬프트를 복구한다
- [ ] T024 [US1] `specs/002-terminal-inline-render/quickstart.md`의 Manual Verification 1·2·3번을 수행한다. 특히 2번(`htmlLabels` 차이로 라벨이 사라지는지)을 가장 먼저 확인하고 결과를 구현 메모에 남긴다

**체크포인트**: 사용자 스토리 1만 구현해도 터미널에서 선택한 차트를 그 자리에서 볼 수 있다.

---

## 단계 4: 사용자 스토리 2 - Mermaid가 아닌 선택은 작업을 방해하지 않음 (우선순위: P2)

**목표**: 차트가 아닌 선택은 아무 출력도 만들지 않고, 렌더 실패는 한 줄로만 안내한다.

**독립 검증**: quickstart Manual Verification 4·5번을 수행해 일반 텍스트에서 출력이 없고 잘못된 문법에서 한 줄 안내만 나오는지 확인한다.

### 구현

- [X] T025 [US2] `apps/cli/src/main.ts`에서 `notAChart` 응답을 받으면 어떤 출력도 없이 정상 종료하도록 처리한다
- [X] T026 [US2] `apps/cli/src/main.ts`에서 `failed` 응답을 받으면 사유를 한 줄로만 표준 오류에 출력하도록 처리한다
- [X] T027 [US2] `apps/cli/src/input.ts`에서 빈 입력과 공백뿐인 입력을 요청 전에 걸러내고, `data-model.md`의 크기 상한을 적용한다
- [X] T028 [US2] `apps/desktop/src-tauri/src/application/use_cases.rs`에서 터미널 경로로 렌더한 차트의 fingerprint를 `DetectClipboardChart`의 `seen` 큐에 등록해 트레이 알림이 중복되지 않도록 한다(FR-011)
- [ ] T029 [US2] quickstart Manual Verification 4·5번을 수행하고, 추가로 터미널에서 차트를 선택했을 때 트레이 알림이 중복되지 않는지 확인한다

**체크포인트**: 잡음 없이 조용히 실패하고, 기존 트레이 흐름과 충돌하지 않는다.

---

## 단계 5: 사용자 스토리 3 - 사용하는 터미널이 달라도 같은 동작 (우선순위: P3)

**목표**: Ghostty·WezTerm·kitty에서 같은 선택·단축키 절차로 차트가 표시되고, 미지원 터미널에서는 안전하게 안내한다.

**독립 검증**: quickstart Manual Verification 8번을 세 터미널에서 수행한다.

### 구현

- [X] T030 [US3] `apps/cli/src/main.ts`에서 이미지 표시를 지원하지 않는 터미널을 감지하면 제어 문자를 출력하지 않고 이유를 한 줄로 안내한 뒤 종료하도록 처리한다(FR-006)
- [X] T031 [US3] `apps/cli/src/probe-command.ts`에 `--probe` 진단을 구현한다. 터미널 종류, graphics 지원 여부, 셀 픽셀 크기, 렌더 소켓 연결 상태를 출력한다(FR-013)
- [X] T032 [P] [US3] `apps/cli/README.md`에 터미널별 최소 설정을 정리한다. Ghostty는 추가 설정 없음, kitty는 `copy_on_select clipboard` 필수, WezTerm은 `enable_kitty_graphics = true` 명시
- [X] T033 [US3] `apps/cli/src/render-client.ts`에서 소켓 연결 실패 시 앱을 기동하고 짧게 재시도한 뒤, 그래도 실패하면 조치를 안내하도록 처리한다(FR-012)
- [ ] T034 [US3] quickstart Manual Verification 8번을 Ghostty·WezTerm·kitty에서 각각 수행하고 차이가 있으면 `research.md`에 기록한다

**체크포인트**: 세 터미널에서 같은 절차로 동작하고, 지원하지 않는 환경에서도 화면이 깨지지 않는다.

---

## 단계 6: 마무리

**목적**: 검증을 완료하고 성능·문서 항목을 확인한다.

- [X] T035 [P] `cd apps/desktop/src-tauri && cargo test`를 실행해 네이티브 단위 테스트를 통과시킨다
- [X] T035b [P] `pnpm --filter @mermaid-live/cli test`를 실행해 CLI 회귀 테스트를 통과시킨다. 인자 해석, 빈 환경변수 처리, 프로토콜 청크 규칙, fit 계산을 다룬다
- [X] T036 [P] `pnpm typecheck`와 `pnpm lint`를 실행해 신규 CLI 패키지와 프론트엔드 변경을 통과시킨다
- [ ] T037 quickstart Manual Verification 6·7번을 수행해 큰 차트의 높이 초과 여부와 재표시 성능(SC-002)을 확인한다
- [X] T038 [P] `README.md`에 터미널 인라인 표시 사용법을 한 단락으로 추가하고 `apps/cli/README.md`를 참조하게 한다

**체크포인트**: 모든 검증 명령이 통과하고 수동 검증 항목이 확인되었다.


---

## 단계 7: CLI 를 Rust 로 이관

**목적**: 배포 마찰, 소켓 계약 이중화, 차트가 아닐 때의 불필요한 IPC 를 구조적으로 없앤다. 사용자가 보는 동작은 바뀌지 않는다.

**독립 검증**: 이관 전 Node CLI 출력을 캡처해 두고 Rust 출력과 바이트 단위로 비교한다.

- [X] T039 이관 전 파리티 기준선을 캡처한다. flowchart·sequenceDiagram·펜스 블록·큰 차트·차트 아님·문법 오류 6건의 stdout·stderr 과 `--probe` 출력을 파일로 남긴다
- [X] T040 `crates/Cargo.toml` 에 workspace 를 만든다. `src-tauri` 는 밖에 남겨 앱의 target 디렉터리와 증분 빌드를 보존한다
- [X] T041 `crates/mermaid-core` 를 만들고 `src-tauri/src/domain/mermaid_chart.rs` 와 `chart_raster.rs` 를 **이동**한다(테스트 포함, 내용 변경 없음)
- [X] T042 `crates/mermaid-core/src/protocol.rs` 에 `RenderRequest`·`RenderResponse`·`socket_path`·`is_valid_request_id` 를 정의한다. `contracts/render-socket.md` 를 그대로 반영하고 직렬화 왕복 테스트를 둔다
- [X] T043 [P] `crates/mmdcat/src/{options,input,png,fit}.rs` 를 작성한다. TS 구현의 상수와 계산 규칙을 그대로 옮긴다
- [X] T044 `crates/mmdcat/src/terminal/tty.rs` 를 작성한다. `/dev/tty` 를 직접 열고 `tcgetattr`/`cfmakeraw`/`tcsetattr` 로 raw 모드를 다루며 복원을 `Drop` 에 맡긴다. 읽기는 `poll` 로 타임아웃한다
- [X] T045 `crates/mmdcat/src/terminal/metrics.rs` 를 작성한다. `TIOCGWINSZ` 로 셀 크기를 먼저 읽고 0 이면 `CSI 16 t` → `CSI 14 t` 질의로 폴백한다
- [X] T046 [P] `crates/mmdcat/src/terminal/{capability,kitty_graphics}.rs` 와 `render_client.rs`, `probe.rs` 를 작성한다. 프로토콜 출력 규약과 `--probe` 출력 형식을 그대로 유지한다
- [X] T047 `crates/mmdcat/src/main.rs` 를 작성한다. `mermaid_core::chart` 로 **로컬 판별**을 먼저 해 차트가 아니면 소켓을 열지 않는다
- [X] T048 `src-tauri` 를 공유 crate 에 연결한다. `Cargo.toml` 에 path 의존성을 추가하고 `domain/mod.rs` 를 re-export 파사드로 바꾸며 `render_socket.rs` 의 지역 wire 타입과 `json!` 리터럴을 `mermaid_core::protocol` 로 교체한다
- [X] T049 파리티를 검증한다. 6건 모두 stdout·stderr 바이트 일치, `--probe` 출력 일치를 확인한다
- [X] T050 `cargo install --path crates/mmdcat` 으로 `~/.cargo/bin/mmdcat` 설치를 확인한다
- [X] T051 `apps/cli/shell/mmdcat.zsh` 를 `crates/mmdcat/shell/` 로 옮기고 PATH 의 `mmdcat` 을 우선 쓰되 저장소 빌드로 폴백하게 한다. 세 키맵 바인딩과 `MMDCAT_KEY` 는 유지한다
- [X] T052 `apps/cli` 를 제거하고 `scripts/clean.mjs` 의 항목을 `crates/target` 으로 바꾸며 `pnpm install` 로 워크스페이스를 정리한다
- [X] T053 `research.md`·`plan.md`·`quickstart.md`·`contracts/render-socket.md` 와 루트 `README.md` 를 갱신한다

**체크포인트**: 출력이 바이트 단위로 동일하고, `mmdcat` 이 PATH 에서 실행되며, 차트가 아닌 입력은 소켓 없이 5ms 에 끝난다.


---

## 단계 8: 클립보드 감시 인라인 렌더링 (사용자 스토리 4)

**목적**: 복사만으로 차트가 터미널에 나타나게 한다. 감시는 앱의 기존 감시자를 구독해 CLI 에 감시 로직을 만들지 않는다.

**독립 검증**: 감시 모드를 띄워 두고 `pbcopy` 로 차트·비차트·같은 차트를 넣어 표시 여부를 확인한다.

- [X] T054 `crates/mermaid-core/src/protocol.rs` 에 `RequestKind`(기본 `render`)와 `RenderResponse::Watching`, `Rendered.label` 을 추가한다. 구버전 클라이언트 호환을 위해 `kind` 와 `text` 에 기본값을 둔다
- [X] T055 `infrastructure/render_service.rs` 에 `Subscribers` 레지스트리를 추가한다. 요청 id·크기·스트림을 보관하고 쓰기 실패 시 즉시 제거한다. 테스트는 `UnixStream::pair()` 로 작성한다
- [X] T056 `infrastructure/app_state.rs` 에 `subscribers` 를 배선한다
- [X] T057 `adapters/inbound/render_socket.rs` 를 `kind` 로 분기한다. `watch` 면 `watching` 을 보낸 뒤 구독자로 등록하고 EOF 까지 연결을 유지한다
- [X] T058 `adapters/inbound/render_socket.rs` 에 `deliver_to_subscribers` 를 추가한다. 구독자가 없으면 즉시 false, 있으면 별도 스레드에서 구독자별 크기로 렌더해 한 줄씩 보낸다
- [X] T059 `infrastructure/clipboard_watcher.rs` 의 `detect()` 직후에 구독자 전달 훅을 넣는다. 구독자가 받아갔으면 스테이징과 알림을 건너뛴다
- [X] T060 [P] `crates/mmdcat/src/options.rs` 에 `--watch`, `--clear` 를 추가한다
- [X] T061 `crates/mmdcat/src/render_client.rs` 에 `open_subscription` 을 추가한다. 읽기 타임아웃으로 크기 변경을 감지할 여지를 만든다
- [X] T062 `crates/mmdcat/src/watch.rs` 에 구독 루프를 작성한다. 줄 단위로 응답을 읽고, 헤더에 `libc::localtime_r` 로 구한 지역 시각과 라벨을 붙이며, `SIGWINCH` 에 재구독하고 연결이 끊기면 재시도한다
- [X] T063 `crates/mmdcat/src/main.rs` 에 `--watch` 분기와 도움말을 추가한다
- [X] T064 단발 경로의 파리티가 유지되는지 확인한다. 프로토콜 변경이 회귀를 만들지 않았음을 6건 바이트 비교로 확인한다
- [X] T065 종단 검증: 감시 모드를 띄우고 새 차트·비차트·같은 차트·다른 새 차트를 `pbcopy` 로 넣어 각각 표시·무표시·무표시·표시를 확인한다. 검증 전후로 사용자 클립보드를 백업·복원한다
- [X] T066 `contracts/render-socket.md`·`research.md`·`spec.md`(US4, FR-014~019, SC-007~008)·`quickstart.md`·`crates/mmdcat/README.md`·루트 `README.md` 를 갱신한다

**체크포인트**: 차트를 복사하면 터미널에 나타나고, 차트가 아닌 복사와 중복 복사에는 아무 일도 일어나지 않으며, 트레이에는 쌓이지 않는다.

---

## 남은 확인 (실제 터미널 필요)

T024, T029, T034, T037은 실제 tty에서만 확인할 수 있어 미완으로 남긴다. 소켓을 통한 렌더 경로와 CLI 출력 바이트는 자동 검증했으나, 터미널이 그 바이트를 이미지로 그리는 단계는 사람이 봐야 한다.

자동 검증으로 확인된 것:

- 앱 단위 테스트 41개 통과(공유 crate 로 이동한 10개는 `crates` 쪽에서 실행)
- Rust 이관 파리티: 6건의 stdout·stderr 과 `--probe` 출력이 Node 기준선과 바이트 단위로 일치
- `crates` 테스트 48개 통과(`cd crates && cargo test`): 이동한 판별·크기 테스트, 프로토콜 직렬화 왕복, PNG 헤더 파싱, 인자 해석, 빈 환경변수 처리, 프로토콜 청크 규칙(4096B 이하, 마지막 외 4의 배수, 첫 청크만 전체 제어키, `m=0` 종료), base64 왕복, fit 계산과 클램프, 터미널 응답 파싱
- 앱을 띄워 소켓으로 렌더 요청 → `flowchart`(한글 라벨 정상 렌더 확인), `sequenceDiagram`, 펜스 코드 블록, 잘못된 문법(실패 응답), 차트 아닌 텍스트(`notAChart`) 모두 기대대로 동작
- 캐시: 동일 요청 재렌더 2ms(첫 렌더 380ms), 크기 변경 시 재렌더
- 차트 아닌 입력에 대해 CLI 표준출력·표준오류 모두 0바이트
- 이미지 미지원 터미널에서 표준출력 0바이트(제어문자 미출력)와 한 줄 안내
- 연속 4회 렌더 후에도 앱 정상, 로그에 오류 없음
- 감시 모드 종단 검증: 새 차트 표시, 비차트 무표시, 같은 차트 중복 무표시, 다른 새 차트 표시. 푸시된 이미지의 청크 규칙·PNG 유효성·헤더(지역 시각과 라벨)와 `--clear` 의 화면 지우기 시퀀스까지 확인

터미널에서 직접 확인할 것:

- `mmdcat --probe`가 Ghostty·WezTerm·kitty에서 graphics 지원과 셀 크기를 올바르게 보고하는지
- 선택 후 `^Xv`로 차트가 인라인 표시되고 프롬프트가 복구되는지
- 세 터미널에서 동일하게 동작하는지

---

## 의존 관계

- 단계 1 → 단계 2 → 단계 3 순서로 진행한다.
- 단계 2의 T004~T014는 순서 의존이 있다. T005·T006이 T011·T012보다 앞서야 하고, T007이 T011보다, T008·T009·T010이 T011의 동작 확인보다 앞서야 한다.
- 단계 3의 T016~T019는 서로 다른 파일이므로 병렬 가능하다. T020~T022는 이들 결과를 조립하므로 뒤에 온다.
- 단계 4는 단계 3의 T022가 있어야 검증 가능하다.
- 단계 5는 단계 3이 끝난 뒤 진행한다. T032는 언제든 병렬 가능하다.
- 단계 6은 마지막에 수행한다.

## 병렬 실행 예시

```text
단계 1: T001, T002, T003 동시 진행
단계 2: T005 와 T004 동시 진행 후, T006 → T007 → T011 순차
단계 3: T016, T017, T018, T019 동시 진행 후 T020 → T021 → T022 → T023
단계 6: T035, T036, T038 동시 진행
```
