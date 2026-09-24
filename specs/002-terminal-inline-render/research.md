# Research: Terminal Inline Render

## Decision: Use `t=d` (direct transmission) for the kitty graphics protocol

kitty graphics protocol은 전송 매체로 `t=d`(이스케이프 코드에 직접 포함), `t=f`(파일 경로), `t=t`(임시 파일), `t=s`(공유 메모리)를 정의한다. 파일 경로 방식은 base64 오버헤드가 없어 매력적이지만 터미널별 지원이 갈린다.

- kitty: 네 가지 모두 지원.
- Ghostty 1.3.1: 바이너리에 `tty-graphics-protocol` 문자열이 존재해 임시 파일 방식의 안전 규칙까지 구현한 것으로 확인된다.
- WezTerm: kitty 이미지 프로토콜 구현이 "largely implemented" 수준으로 평가되며, 파일 경로 방식 지원이 확인되지 않는다.

세 터미널의 공통분모는 `t=d`이므로 이를 채택한다. 대가는 base64 인코딩 비용과 청크 분할이며, 로컬 파이프로 수백 KB를 보내는 수준에서는 문제가 되지 않는다.

**청크 규칙**: base64 인코딩 후 4096바이트 이하로 나눈다. 마지막을 제외한 모든 청크 크기는 4의 배수여야 한다. 첫 청크만 전체 제어 키를 갖고, 이후 청크는 `m` 키만 포함하며 마지막 청크는 `m=0`으로 끝낸다.

## Decision: Delegate chart detection to the desktop app instead of duplicating it

차트 판별 로직은 `apps/desktop/src-tauri/src/domain/mermaid_chart.rs`에 이미 존재하며, 펜스 코드 블록 추출과 frontmatter·directive·주석 전처리를 모두 처리한다. CLI에 같은 토큰 목록을 다시 구현하면 두 곳이 어긋날 수 있다.

CLI는 판별을 전혀 하지 않고 원문을 그대로 앱에 보내며, 앱이 `extract_mermaid_chart_source()`로 판별한 뒤 "차트 아님" 또는 이미지로 응답한다. 판별 기준이 한 곳에만 존재하게 되어 FR-002를 구조적으로 보장한다.

렌더링을 위해 어차피 앱이 실행 중이어야 하므로, 판별을 위임해도 새로운 의존이 생기지 않는다.

## Decision: Reuse the existing hidden webview for offscreen rendering

Mermaid는 텍스트 크기 측정을 위해 DOM이 필요하다. 새 브라우저 엔진(puppeteer 등)을 도입하는 대신 실행 중인 앱의 webview를 재사용한다.

`native_window_manager.rs`의 `prewarm_temp_diagram_window()`가 이미 `visible=false`로 webview 창을 만들어 차트를 미리 렌더하는 경로를 갖고 있다. 같은 방식으로 rasterize 전용 숨은 창을 만들고, 렌더 결과를 기존 `save-diagram` 패턴(백엔드 요청 → 프론트엔드 `invoke` 응답)으로 돌려받는다.

부수 효과로 Mermaid 버전과 테마 정의가 데스크톱 미리보기와 자동으로 일치한다(FR-010).

## Decision: Disable `htmlLabels` on the rasterize path only

Mermaid의 flowchart 기본값 `htmlLabels: true`는 라벨을 SVG `<foreignObject>`로 그린다. SVG를 `<img>`로 불러 canvas에 그리는 래스터화 경로에서는 `foreignObject`가 렌더되지 않아 라벨이 사라진다.

따라서 rasterize 경로에서만 `flowchart: { htmlLabels: false }`를 적용해 라벨을 SVG `<text>`로 그린다. 데스크톱 미리보기와 라벨 렌더링 방식이 미세하게 달라지는 것은 **의도된 차이**로 기록한다. 색·폰트·테마 변수는 공유 설정 모듈을 통해 동일하게 유지된다.

폰트는 `"Avenir Next"`를 이름으로 참조하며 `Segoe UI, sans-serif` 폴백 체인을 유지해, 폰트 해석에 실패해도 레이아웃이 무너지지 않게 한다.

## Decision: Feed the SVG to the canvas as a data URL, not a blob URL

구현 중 발견한 제약이다. WebKit은 blob URL로 불러온 SVG 이미지를 canvas에 그리면 canvas를 오염된 것으로 취급해 `toDataURL()`이 `SecurityError: The operation is insecure`로 실패한다. Tauri의 webview가 WebKit이므로 이 경로는 그대로 막힌다.

같은 SVG를 `data:image/svg+xml;charset=utf-8,<percent-encoded>` 로 불러오면 canvas가 읽기 가능한 상태로 남아 PNG 추출이 성공한다. 따라서 rasterize 경로는 blob URL을 쓰지 않는다.

이 선택은 되돌리기 쉽지 않다. blob URL로 "정리"하려는 시도는 런타임에서만 실패하므로, 이 주석과 결정 기록을 남겨 재발을 막는다.

## Decision: Paint an opaque light background into the raster

앱의 테마는 밝은 도형에 어두운 글자(`primaryTextColor: #1f2933`)를 쓰고 `background`는 `transparent`다. 터미널은 배경이 어두운 경우가 많아, 투명 배경 그대로 내보내면 글자가 배경에 묻혀 읽을 수 없다.

따라서 래스터화 시 canvas를 흰색으로 채운 뒤 차트를 그린다. 터미널 테마와 무관하게 가독성이 보장된다.

## Decision: Use a Unix domain socket for CLI-to-app requests

CLI와 앱 사이 통신 수단으로 로컬 HTTP 서버, Unix 도메인 소켓, 파일 감시 폴링을 검토했다.

Unix 도메인 소켓을 택한다. `std::os::unix::net::UnixListener`만 사용하므로 **새 crate 의존성이 없고**, 포트 관리나 충돌 문제가 없으며, 소켓 파일 경로가 기존 `$TMPDIR/mermaid-live/` 규칙에 자연스럽게 들어간다. 요청·응답은 개행으로 구분된 JSON 한 줄이다.

앱 시작 시 남아 있는 소켓 파일을 정리해 재시작 후에도 연결이 가능하도록 한다.

## Decision: Trigger from the clipboard through a zsh widget

터미널에서 선택한 텍스트를 얻는 방법은 터미널마다 다르다. Ghostty에는 `write_selection_file` 액션이 있고 kitty에는 `launch --stdin-source=@selection`이 있지만, 셋 모두에서 같은 방식으로 동작하는 경로는 **클립보드**다.

- Ghostty: `copy-on-select` 기본값이 `true`이므로 추가 설정이 필요 없다.
- WezTerm: 마우스 선택 완료 시 클립보드로 복사되는 것이 기본 동작이다.
- kitty: `copy_on_select` 기본값이 `no`이므로 `copy_on_select clipboard` 설정이 **필수**다.

트리거는 zsh ZLE 위젯에 단축키를 바인딩해 구현한다. 셸 계층에 있으므로 터미널 종류와 무관하게 같은 키가 동작한다. 이미지는 `/dev/tty`로 직접 출력하고 `zle reset-prompt`로 프롬프트를 복구해 FR-008을 만족한다.

## Decision: Compute the fit size in cells, mirroring the desktop fit rule

`mermaid-preview.tsx`의 `getFitZoom()`은 사용 가능한 영역에서 패딩을 뺀 뒤 폭·높이 비율 중 작은 값을 취하고 최소/최대 배율로 클램프한다. 터미널에서도 같은 규칙을 셀 단위로 적용해 표시 크기 감각을 맞춘다.

셀 픽셀 크기는 `CSI 16 t`(셀 크기)를 먼저 시도하고 `CSI 14 t`(창 픽셀 크기)를 폴백으로 쓴다. 두 질의가 모두 실패하면 열 수만 사용해 `c=` 하나만 지정한다. 프로토콜 명세상 `c`와 `r` 중 하나만 주면 나머지는 종횡비를 유지하도록 계산되므로, 이 폴백에서도 왜곡은 발생하지 않는다.

## Decision: Suppress duplicate tray notifications for terminal-rendered charts

`copy-on-select`가 켜진 터미널에서 차트를 선택하면 기존 `clipboard_watcher`도 같은 내용을 감지한다. 다만 감시자는 창을 자동으로 열지 않고 스테이징과 알림만 수행하므로 충돌은 알림 중복에 한정된다.

터미널 경로로 렌더한 차트의 fingerprint를 감시자의 "이미 본 것" 목록에 등록해 중복 알림을 억제한다. 기존 `DetectClipboardChart`의 `seen` 큐를 그대로 활용한다.

## Decision: Ship the CLI as a Rust binary sharing a crate with the app

최초 구현은 TypeScript(Node) CLI였다. 동작은 맞았지만 사용 과정에서 세 가지가 문제로 드러나 Rust 로 이관했다.

- **배포 마찰** — `pnpm build` 로 `dist` 를 만들어야 하고, 다른 패키지의 의존성이 아닌 독립 워크스페이스 패키지라 pnpm 이 bin 을 링크하지 않아 PATH 에 없었고, mise 로 관리되는 Node 에 의존했다. `cargo install --path crates/mmdcat` 이 `~/.cargo/bin` 으로 설치하며 이 문제가 사라진다.
- **계약 이중화** — 소켓 JSON 스키마가 TS 클라이언트(수동 파싱)와 Rust 서버(`json!` 리터럴)에 각각 존재했다. 이제 `mermaid-core::protocol` 의 serde 타입 하나를 양쪽이 공유하므로 어긋날 수 없다.
- **불필요한 IPC** — 판별을 전부 앱에 위임했으므로 차트가 아닌 텍스트에도 소켓 왕복이 필요했다. CLI 가 `mermaid_core::chart` 로 먼저 판별하면 차트가 아닐 때 소켓을 열지 않는다. 측정 결과 5ms 에 무출력 종료하며 앱이 떠 있지 않아도 조용히 실패한다. 앱은 받은 텍스트에 대해 자기 판별을 계속 수행한다.

Cargo 구조는 **`crates/` 에 별도 workspace** 를 두고 `src-tauri` 는 밖에 남겼다. 루트 workspace 로 합치면 target 디렉터리가 루트로 이동해 당시 3.7GB 였던 앱 빌드 캐시가 무효화되고 Tauri 전체 재빌드가 발생한다. path 의존성은 workspace 멤버십을 요구하지 않으므로 두 workspace 가 같은 crate 를 공유할 수 있다. 대가는 공유 crate 의 중복 컴파일인데 의존성이 serde 뿐이라 작다.

Rust 로 바뀌면서 개선된 지점:

- 셀 픽셀 크기를 `libc::ioctl` 의 `TIOCGWINSZ` 로 커널에서 바로 읽는다. 0 을 보고하는 터미널을 위해 기존 `CSI 16 t` → `CSI 14 t` 질의를 폴백으로 남겼다. 흔한 경우에 터미널 왕복이 사라진다.
- raw 모드를 `tcgetattr`/`cfmakeraw`/`tcsetattr` 로 직접 다룬다. 질의마다 `stty` 프로세스를 두 번 띄우던 것이 없어지고, 복원을 `Drop` 에 맡겨 조기 반환에도 원래 모드가 돌아온다.

**파리티 검증**: 이관 전 Node CLI 의 출력을 flowchart·sequenceDiagram·펜스 블록·큰 차트·차트 아님·문법 오류 6건에 대해 캡처해 두고, Rust 구현의 출력과 바이트 단위로 비교해 stdout·stderr 모두 전부 일치함을 확인했다. `--probe` 출력도 완전히 일치한다. 앱 재시작을 넘어서도 일치했으므로 래스터화가 재현 가능하다는 것도 함께 확인됐다.

## Decision: Subscribe to the app's clipboard watcher rather than polling in the CLI

복사만으로 차트가 보이는 감시 모드(`mmdcat --watch`)를 추가할 때, CLI 가 직접 클립보드를 폴링하는 방법과 앱의 기존 감시자를 구독하는 방법을 검토했다.

**구독을 택했다.** 앱은 이미 `NSPasteboard` 의 changeCount 를 폴링해 **내용을 읽지 않고** 변경을 감지한다. 이는 macOS 붙여넣기 권한 경고를 피하려는 의도적 설계이며, CLI 가 같은 일을 하려면 objc2 와 AppKit 의존성을 들여와야 한다. 게다가 감지 규칙(중복 제거용 seen 큐, 크기 상한)이 양쪽에 생기면 어긋난다.

구독 방식의 부수 효과로 `RasterCache` 와 래스터라이저가 그대로 적용되고, seen 큐가 경로를 가로질러 동작한다 — 단발 렌더로 이미 본 차트를 복사해도 감시 모드가 다시 보여 주지 않는다.

대가는 두 가지다. 소켓 계약이 "요청 하나에 응답 하나" 에서 `watch` 만 스트리밍을 허용하도록 바뀌었고, 트레이의 감시 토글이 `--watch` 에도 적용된다. 후자는 스위치가 하나인 편이 옳다고 보았고, `watching` 응답에 토글 상태를 실어 CLI 가 시작 시 알리게 했다.

## Decision: A chart shown in a subscribed terminal is not staged in the tray

구독자가 받아간 차트는 스테이징도 알림도 하지 않는다. 이미 눈으로 본 차트에 트레이 배지가 쌓이면 알림의 의미가 없어진다.

되돌릴 수 없는 성질이 있다 — 그 차트는 트레이에 남지 않으므로 나중에 큰 창으로 열 수 없다. 의도한 절충이며 `crates/mmdcat/README.md` 에 명시했다.

## Decision: Run the desktop app as a resident menu bar accessory

렌더링이 실행 중인 앱의 webview 에 의존하므로 앱이 떠 있어야 한다. 실제 사용에서 이것이 두 번 문제가 됐다 — 앱을 띄우는 것을 잊으면 첫 렌더가 실패하고, `/Applications` 에 남아 있던 구버전 번들이 자동 기동 대상이 되어 소켓을 열지 않은 채 14초를 기다리게 했다.

CLI 가 앱 없이 자체적으로 렌더하는 방법(headless Chrome)도 검토했지만, 렌더 경로가 둘로 늘고 Chrome 바이너리와 mermaid.js 자산을 따로 관리해야 하며, 감시 모드는 앱의 감시자를 구독하므로 그 경로로는 쓸 수 없다.

**앱을 배경 상주로 만드는 편을 택했다.** 두 가지 변경으로 충분하다.

- `tauri.conf.json` 의 `app.windows` 를 비워 시작 시 창을 만들지 않는다. 트레이만 남는다. `window_lifecycle::setup_window_management` 는 창을 요구하지 않고 메뉴와 트레이만 설정하므로 변경이 필요 없었고, 창 진입점은 트레이의 "새 창" 이 이미 제공한다.
- `src-tauri/Info.plist` 에 `LSUIElement` 를 두어 Dock 아이콘과 앱 메뉴 없이 메뉴바에만 존재하게 한다. 배경 렌더러에 Dock 아이콘은 소음이다. 이 키를 지우면 원래대로 돌아온다.

로그인 시 자동 실행은 `scripts/install-launch-agent.mjs` 가 LaunchAgent 를 설치한다. `KeepAlive` 는 `SuccessfulExit=false` 로 두어 크래시는 복구하되 트레이의 "종료" 는 존중한다.

부수 효과로 `mmdcat` 의 자동 기동 경로가 거의 쓰이지 않게 되고, 구버전 번들이 가로채는 문제도 사라진다. 다만 **GUI 세션이 필요하다는 제약은 그대로다** — 원격 SSH 세션에서는 동작하지 않는다.

## Known Limitation: very tall charts

fit 규칙은 폭과 높이를 모두 표시 영역 안에 넣으므로, 세로로 매우 긴 차트는 높이에 맞춰 축소되면서 폭이 크게 줄어든다. 24개 노드를 세로로 잇는 `flowchart TD`를 80x24 터미널에서 요청하면 결과가 37px 폭까지 좁아진다.

이는 명세된 동작이며(FR-004, 그리고 프롬프트를 밀어내지 않아야 한다는 edge case), kitty 프로토콜이 화면을 넘어가는 배치의 동작을 정의하지 않는다는 점에서도 안전한 선택이다. 대신 세로로 긴 차트는 터미널 창을 키우거나 데스크톱 창에서 보는 편이 낫다는 점을 `crates/mmdcat/README.md`에 안내한다.

## Sources

- kitty terminal graphics protocol: https://sw.kovidgoyal.net/kitty/graphics-protocol/
- WezTerm kitty image protocol tracking issue: https://github.com/wezterm/wezterm/issues/986
- WezTerm features: https://wezterm.org/features.html
- Ghostty 기본 설정 확인: `ghostty +show-config --default --docs`
- kitty 기본 설정 확인: `~/.config/kitty/kitty.conf` (`copy_on_select no`)
