# 구현 계획: Terminal Inline Render

**브랜치**: `002-terminal-inline-render` | **날짜**: 2026-09-10 | **스펙**: [spec.md](./spec.md)
**입력**: `/specs/002-terminal-inline-render/spec.md`의 기능 명세

**참고**: 이 문서는 Spec Kit 계획 단계 산출물입니다. 실행 흐름은 `.specify/templates/plan-template.md`를 기준으로 합니다.

## 요약

터미널에서 선택한 텍스트가 Mermaid 차트일 때, kitty graphics protocol로 렌더된 차트를 그 터미널 안에 인라인으로 표시하는 경로를 추가한다. 렌더링은 새 브라우저 엔진을 도입하지 않고 **실행 중인 데스크톱 앱의 숨은 webview를 재사용**한다. 차트 판별은 네이티브에 이미 있는 `extract_mermaid_chart_source()`를 단일 기준으로 위임한다. 트리거는 클립보드와 zsh ZLE 위젯을 사용해 Ghostty·WezTerm·kitty에서 동일하게 동작시킨다.

## 기술 컨텍스트

**언어/버전**: Rust edition 2021 (CLI, 공유 crate, 네이티브) / Tauri 2.5, TypeScript 5.9 + React 19 (프론트엔드)  
**주요 의존성**: Mermaid 11.6(기존 워크스페이스 버전 재사용), Vite 6. CLI는 `libc`·`base64`·`serde`·`serde_json` 만 쓰고 인자 파싱을 직접 처리한다(`clap` 없음). 소켓은 `std::os::unix::net`, raw 모드와 창 크기는 `libc` 직접 호출  
**저장소**: 해당 없음. 렌더 결과 PNG는 `$TMPDIR/mermaid-live/` 아래 fingerprint 기반 캐시 파일로만 보관한다  
**테스트**: `cd apps/desktop/src-tauri && cargo test`(앱 35개), `cd crates && cargo test`(공유 crate 15개 + CLI 28개), `pnpm typecheck`, quickstart 수동 검증. 모두 기존 인라인 `#[cfg(test)]` 관례를 따른다. 인라인 이미지 표시는 실제 tty가 필요해 자동화하지 않는다  
**대상 플랫폼**: macOS. 터미널은 kitty graphics protocol을 지원하는 Ghostty·WezTerm·kitty  
**프로젝트 유형**: pnpm 모노레포의 데스크톱 앱 + 신규 CLI 앱  
**성능 목표**: 첫 차트 표시 3초 이내, 동일 차트 재표시 1초 이내(SC-002)  
**제약 조건**: 기존 클립보드 감시와 트레이 흐름을 대체하지 않고 표시 경로만 추가한다. 프론트엔드는 Feature-Sliced Design 경계를, 네이티브는 hexagonal architecture 경계를 유지한다. 차트 판별 규칙 자체는 변경하지 않는다  
**범위**: `crates/mermaid-core`(판별·크기 계약·소켓 프로토콜)와 `crates/mmdcat`(CLI), 네이티브 렌더 서비스(포트·유스케이스·인바운드 소켓·아웃바운드 rasterizer), 프론트엔드 rasterize 페이지와 공유 Mermaid 설정 모듈, 셸 통합 스니펫

## 헌법 체크

*게이트: Phase 0 리서치 전에 통과해야 하며, Phase 1 설계 후 다시 확인한다.*

constitution 파일은 아직 플레이스홀더 원칙을 포함하고 있으므로, `AGENTS.md`와 `CLAUDE.md`의 저장소 지침 외에 강제 가능한 프로젝트별 게이트는 없다.

- **모노레포/Tauri 구조**: PASS. 신규 CLI는 `pnpm-workspace.yaml`의 기존 `apps/*` 글롭 안에 두어 모노레포 구조를 유지한다.
- **프론트엔드 Feature-Sliced Design**: PASS. rasterize 화면은 `pages`에, 공유 Mermaid 설정은 `shared/lib`에 둔다. 하위 계층이 상위 계층을 import 하지 않는다.
- **네이티브 hexagonal architecture**: PASS. 소켓은 인바운드 어댑터, webview 래스터화는 아웃바운드 어댑터, 판별·캐시 결정은 유스케이스, 크기 값 객체는 도메인에 둔다. 도메인과 애플리케이션 계층에 Tauri 타입을 넣지 않는다.
- **작은 모듈 우선**: PASS. CLI는 입력·터미널 질의·프로토콜 출력·소켓 클라이언트를 각각 분리한다.
- **UI 규칙**: 해당 없음. rasterize 화면은 사용자에게 보이지 않는 렌더 전용 표면이며 새 UI 프리미티브를 만들지 않는다.

## 프로젝트 구조

### 문서 구조(이 기능)

```text
specs/002-terminal-inline-render/
├── plan.md
├── spec.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── render-socket.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### 소스 코드(저장소 루트)

```text
crates/                               # CLI 전용 workspace (app 과 분리)
├── mermaid-core/src/                 # 앱과 CLI 가 공유
│   ├── chart.rs                      # 차트 판별
│   ├── raster.rs                     # RasterSpec
│   └── protocol.rs                   # 소켓 요청·응답, socket_path
└── mmdcat/
    ├── shell/mmdcat.zsh
    └── src/
        ├── main.rs, options.rs, input.rs, fit.rs, png.rs, probe.rs
        ├── render_client.rs
        └── terminal/{tty.rs, capability.rs, metrics.rs, kitty_graphics.rs}

apps/
└── desktop/
    ├── src/
    │   ├── app/
    │   │   └── App.tsx               # rasterize 모드 분기
    │   ├── features/
    │   │   └── preview-diagram/ui/mermaid-preview.tsx   # 설정 추출
    │   ├── pages/
    │   │   └── rasterize/            # 신규
    │   └── shared/
    │       └── lib/mermaid-config.ts # 신규
    └── src-tauri/src/
        ├── domain/chart_raster.rs             # 신규
        ├── application/{ports.rs,use_cases.rs}
        ├── adapters/inbound/{render_socket.rs,tauri_commands.rs}
        ├── adapters/outbound/webview_rasterizer.rs  # 신규
        └── infrastructure/{render_service.rs,mod.rs}
```

**구조 결정**: CLI는 데스크톱 앱과 배포 단위가 다르므로 `crates/` 의 별도 Cargo workspace 로 분리한다. `src-tauri` 는 그 밖에 남기고 `mermaid-core` 를 path 의존성으로만 참조해, 앱의 target 디렉터리와 증분 빌드 상태를 보존한다. 앱의 `domain/mod.rs` 는 `mermaid_core::chart`·`mermaid_core::raster` 를 원래 모듈명으로 re-export 하므로 모든 `crate::domain::...` 경로가 그대로 유지된다. 네이티브에서는 소켓을 인바운드 어댑터로 두어 "Tauri 커맨드 외의 인바운드 전달 수단"이라는 기존 계층 정의를 그대로 따른다. `mermaid.initialize` 설정은 미리보기와 rasterize가 공유해야 하므로 `shared/lib`로 추출한다.

## Phase 0: 리서치

[research.md](./research.md)를 참고한다. 계획 단계의 미확정 사항은 모두 해소되었다.

- 전송 매체는 세 터미널 공통분모인 `t=d`를 쓴다. WezTerm의 파일 경로 방식 지원이 확인되지 않기 때문이다.
- 차트 판별은 CLI에 복제하지 않고 네이티브 도메인 함수에 위임해 단일 기준을 유지한다.
- 오프스크린 렌더는 이미 존재하는 숨은 webview 생성 경로를 재사용한다.
- 래스터화 경로에서만 `htmlLabels: false`가 필요하다. `foreignObject`가 `<img>` 기반 canvas 렌더에 나타나지 않기 때문이며, 이는 의도된 차이로 문서화한다.
- CLI↔앱 통신은 새 의존성이 없는 Unix 도메인 소켓을 쓴다.
- 선택→클립보드 반영은 Ghostty·WezTerm에서 기본 동작이고 kitty만 설정이 필요하다.

## Phase 1: 설계

[data-model.md](./data-model.md), [contracts/render-socket.md](./contracts/render-socket.md), [quickstart.md](./quickstart.md)를 참고한다.

설계 요약:

- CLI는 입력을 클립보드·stdin·파일 경로 중에서 해석하고, 터미널의 이미지 지원 여부와 셀 크기를 질의한 뒤 목표 픽셀 크기를 계산해 소켓으로 보낸다.
- 앱은 원문에서 차트를 추출하고, fingerprint와 목표 크기로 캐시를 조회한 뒤, 없으면 숨은 webview에서 래스터화한다.
- 프론트엔드 rasterize 화면은 공유 설정으로 Mermaid를 렌더하고 SVG를 canvas로 옮겨 PNG를 만들어 커맨드로 돌려준다.
- CLI는 PNG를 base64 4096바이트 청크로 나눠 `a=T,f=100,t=d,q=2`와 계산된 `c`·`r`로 출력한다.
- zsh 위젯이 단축키를 받아 CLI를 실행하고, 출력은 `/dev/tty`로 보내고 프롬프트를 복구한다.
- 차트가 아니면 앱이 그 사실만 응답하고 CLI는 아무것도 출력하지 않는다.

## 헌법 체크 - 설계 후

- **모노레포/Tauri 구조**: PASS. 신규 패키지가 기존 워크스페이스 글롭과 루트 `pnpm` 명령 체계 안에 들어간다.
- **프론트엔드 Feature-Sliced Design**: PASS. 렌더 화면은 `pages`, 공유 설정은 `shared/lib`에 있고 역방향 import가 없다.
- **네이티브 hexagonal architecture**: PASS. 도메인 값 객체에 Tauri 타입이 없고, 유스케이스는 포트에만 의존하며, Tauri·webview 접촉은 어댑터에 갇혀 있다.
- **작은 모듈 우선**: PASS. CLI 모듈이 역할별로 분리되어 있고 광범위한 유틸리티 파일을 만들지 않는다.
- **테스트**: PASS. 네이티브 유스케이스와 소켓 파싱에 단위 테스트를 추가한다. 표시 계층은 tty가 필요해 수동 검증으로 남긴다.

## 복잡도 추적

| 항목 | 왜 필요한가 | 완화 |
|---|---|---|
| CLI↔앱 프로세스 간 통신 추가 | 렌더링을 실행 중인 webview에 위임하기로 결정했으므로 경계를 넘는 호출이 불가피하다 | 새 의존성 없는 Unix 소켓과 한 줄 JSON으로 최소화하고, 계약을 `contracts/render-socket.md`에 고정한다 |
| 래스터화 경로의 `htmlLabels` 차이 | `foreignObject`가 canvas 래스터화에 렌더되지 않는 브라우저 제약 | 색·폰트·테마 변수는 공유 설정으로 동일하게 유지하고, 차이를 research.md에 명시한다 |
| 앱 실행 전제 | 렌더 엔진을 재사용하려면 앱이 떠 있어야 한다 | 미실행 시 CLI가 앱을 띄우고 재시도하며, 진단 명령으로 상태를 확인할 수 있게 한다 |
