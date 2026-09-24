# Contract: Render Socket

## Scope

이 계약은 CLI와 데스크톱 앱 사이의 로컬 렌더링 요청 경로를 정의한다. 외부 네트워크 API가 아니며, 같은 사용자 계정의 같은 머신 안에서만 사용한다.

## Shared types

이 계약은 `crates/mermaid-core/src/protocol.rs` 의 `RenderRequest`·`RenderResponse` 로 구현되며, 앱과 CLI 가 같은 타입을 공유한다. 아래 형식과 코드가 어긋날 수 없다. 소켓 경로도 같은 모듈의 `socket_path()` 가 결정한다.

## Transport

- Unix 도메인 소켓.
- 경로는 앱의 기존 임시 디렉터리 규칙 아래에 둔다: `$TMPDIR/mermaid-live/render.sock`. `MERMAID_LIVE_SOCKET` 으로 양쪽 모두 재정의할 수 있다.
- 요청과 응답은 각각 개행으로 끝나는 JSON 한 줄이다.
- 요청 하나에 응답 하나를 처리한 뒤 닫는다. **예외는 `watch` 요청이다** — 연결을 유지하고 앱이 감지한 차트를 줄 단위로 계속 밀어 준다.
- 앱은 시작 시 남아 있는 소켓 파일을 정리하고 새로 바인딩한다.

## Request

```json
{
  "kind": "render",
  "requestId": "string",
  "text": "string",
  "widthPx": 1200,
  "heightPx": 700,
  "scale": 2
}
```

- `kind`: 선택. `render`(기본) 또는 `watch`. 생략하면 `render` 이므로 watch 모드를 모르는 구버전 클라이언트도 유효한 요청을 보낸다.

- `requestId`: 필수. 응답 짝짓기용.
- `text`: `render` 에서 필수. 사용자가 선택한 원문이며 앱이 차트 추출과 판별을 수행한다. `watch` 에서는 쓰이지 않는다.
- `widthPx`, `heightPx`: 필수. 결과 이미지가 넘지 않아야 하는 최대 크기로, **device pixel 단위이며 `scale`이 이미 반영된 값**이다. 양의 정수이며 8192를 넘지 않는다.
- `scale`: 선택. 기본값 1. device pixel 비율. 논리 크기 기준 박스는 `widthPx / scale`이며, 렌더러는 논리 박스에 맞춘 뒤 `scale`배로 출력한다. 허용 범위는 0.25~4.0이다.

이 규칙 덕분에 클라이언트는 결과 이미지의 픽셀 크기를 `scale`로 나누어 차지할 셀 수를 계산할 수 있다.

## Response

**성공**

```json
{ "status": "rendered", "requestId": "string", "fingerprint": "string", "pngBase64": "string" }
```

**차트 아님**

```json
{ "status": "notAChart", "requestId": "string" }
```

**렌더 실패**

```json
{ "status": "failed", "requestId": "string", "message": "string" }
```

**구독 성립** (`watch` 요청의 첫 응답)

```json
{ "status": "watching", "requestId": "string", "clipboardWatchEnabled": true }
```

`clipboardWatchEnabled` 는 앱 트레이의 감시 토글 상태다. 꺼져 있으면 아무것도 도착하지 않으므로 클라이언트가 그 이유를 알릴 수 있다.

**구독 중 푸시**

```json
{ "status": "rendered", "requestId": "string", "fingerprint": "string", "label": "flowchart LR", "pngBase64": "string" }
```

`label` 은 차트의 짧은 이름이며 푸시에만 실린다. 단발 렌더 응답에는 없다.

## Behavior

**Given** 유효한 요청과 Mermaid 차트로 판별되는 `text`  
**When** 앱이 요청을 처리함  
**Then** 목표 크기에 맞춰 래스터화된 PNG를 `rendered` 상태로 응답한다.

**Given** `text`가 Mermaid 차트가 아님  
**When** 앱이 요청을 처리함  
**Then** `notAChart`를 응답하며 어떤 창도 만들지 않는다.

**Given** 같은 `text`와 같은 목표 크기의 요청이 이미 처리된 적 있음  
**When** 앱이 요청을 처리함  
**Then** 캐시된 이미지를 사용해 새 래스터화 없이 응답한다.

**Given** 차트로 판별되었으나 문법 오류로 렌더링이 실패함  
**When** 앱이 요청을 처리함  
**Then** `failed`와 한 줄 사유를 응답한다.

**Given** 래스터화가 정해진 시간 안에 끝나지 않음  
**When** 대기 시간이 초과됨  
**Then** `failed`를 응답하고 래스터화를 위해 만든 숨은 창을 정리한다.

**Given** 소켓이 존재하지 않거나 연결이 거부됨  
**When** CLI가 요청을 시도함  
**Then** CLI는 앱을 기동하고 짧게 재시도한 뒤, 그래도 실패하면 사용자에게 조치를 안내한다.

## Watch Behavior

**Given** `kind` 가 `watch` 인 요청  
**When** 앱이 요청을 처리함  
**Then** `watching` 을 보내고 연결을 유지하며, 클립보드 감시자가 새 차트를 찾을 때마다 그 구독자의 크기로 렌더해 `rendered` 를 한 줄 보낸다.

**Given** 구독자가 있는 상태에서 감시자가 차트를 감지함  
**When** 차트를 구독자에게 보냄  
**Then** 그 차트는 트레이에 스테이징하지도 알리지도 않는다. 사용자가 이미 터미널에서 봤기 때문이다.

**Given** 클라이언트가 연결을 끊음  
**When** 앱이 EOF 를 읽거나 쓰기가 실패함  
**Then** 그 구독자를 즉시 등록 해제한다.

**Given** 같은 차트를 다시 복사함  
**When** 감시자가 폴링함  
**Then** 이미 보여 준 차트이므로 다시 보내지 않는다(감지 단계의 seen 큐).

크기 변경은 프로토콜로 다루지 않는다. 클라이언트가 새 크기로 다시 구독한다.

## Non-Goals

- 원격 호스트에서의 렌더링.
- 동일 요청의 스트리밍 또는 부분 응답.
- 여러 요청을 한 연결에서 파이프라이닝. `watch` 는 응답만 여러 개이며 요청은 하나다.
- 구독 중 크기 변경 통보. 재구독으로 대신한다.
