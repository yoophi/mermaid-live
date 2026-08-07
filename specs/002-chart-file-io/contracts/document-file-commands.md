# 계약: 문서 파일 명령

## 목적

프론트엔드 문서 워크플로와 native 파일 애플리케이션 계층 사이의 안정적인 요청·응답 경계를 정의한다. 필드 이름은 JSON 직렬화 기준 camelCase를 사용하며 경로는 native 계층이 반환한 불투명 절대 경로로 취급한다.

## 공통 DTO

```ts
interface FileRevision {
  contentHash: string;
  byteLength: number;
  modifiedAt: number | null;
}

interface DocumentFileBinding {
  path: string;
  fileName: string;
  extension: "mmd" | "mermaid";
  revision: FileRevision;
}

interface DiagramFileSnapshot {
  source: string;
  binding: DocumentFileBinding;
}
```

## `open_diagram_file`

native open dialog에서 지원 파일을 선택하고 원문 snapshot을 읽는다.

### 요청

```ts
{}
```

### 성공 응답

```ts
type OpenDiagramFileOutcome =
  | { status: "opened"; snapshot: DiagramFileSnapshot }
  | { status: "cancelled" };
```

### 규칙

- 선택 가능 형식은 `.mmd`, `.mermaid`다.
- 사용자가 dialog를 취소하면 `cancelled`이며 오류가 아니다.
- Mermaid 문법 오류나 빈 원문도 `opened`로 반환한다.
- 경로가 사라짐, 권한 없음, 비 UTF-8, 읽기 실패는 typed command error로 반환한다.

## `save_diagram_file`

신규 경로 선택 또는 연결 파일의 조건부 저장을 수행한다.

### 요청

```ts
interface SaveDiagramFileRequest {
  source: string;
  targetPath: string | null;
  expectedRevision: FileRevision | null;
  force: boolean;
  suggestedFileName: string;
}
```

### 성공 응답

```ts
type SaveDiagramFileOutcome =
  | { status: "saved"; snapshot: DiagramFileSnapshot }
  | { status: "conflict"; diskSnapshot: DiagramFileSnapshot }
  | { status: "cancelled" };
```

### 규칙

- targetPath가 null이면 native Save As dialog를 연다.
- 신규 파일명에 확장자가 없으면 `.mmd`를 붙인다.
- `.mmd`, `.mermaid` 이외의 명시적 확장자는 오류다.
- targetPath와 expectedRevision이 모두 있으면 저장 직전에 disk revision을 비교한다.
- revision 불일치이며 force가 false면 파일을 쓰지 않고 `conflict`를 반환한다.
- force가 true이면 명시적 사용자 승인 이후 요청으로 간주하되 경로 유효성과 안전한 교체 규칙은 동일하게 적용한다.
- 원본 교체와 새 revision 계산이 모두 성공한 뒤에만 `saved`를 반환한다.
- 실패 또는 취소에서 프론트엔드는 baseline과 binding을 갱신하지 않는다.

## `prompt_unsaved_changes`

dirty 문서를 교체하거나 닫기 전에 사용자의 결정을 받는다.

### 요청

```ts
interface UnsavedChangesPromptRequest {
  fileName: string;
  intent: "open" | "close";
}
```

### 응답

```ts
type UnsavedChangesDecision = "save" | "discard" | "cancel";
```

## `prompt_external_conflict`

외부 변경 conflict 처리 방법을 받는다.

### 요청

```ts
interface ExternalConflictPromptRequest {
  fileName: string;
}
```

### 응답

```ts
type ExternalConflictDecision = "reload" | "overwrite" | "cancel";
```

## `authorize_window_close`

현재 webview가 dirty 보호를 완료한 뒤 해당 window의 다음 close 요청 한 번을 승인한다.

### 요청

```ts
{}
```

### 응답

```ts
void
```

### 규칙

- 승인은 호출 window label에만 귀속된다.
- 승인 토큰은 다음 close 요청에서 한 번 소비된다.
- 다른 window나 이후 close 요청에 재사용할 수 없다.

## 오류 계약

명령 오류는 최소한 다음 category와 사용자 표시 가능한 message를 가진다.

```ts
interface DocumentFileError {
  category:
    | "unsupportedExtension"
    | "missing"
    | "permissionDenied"
    | "invalidUtf8"
    | "readFailed"
    | "writeFailed"
    | "replaceFailed";
  message: string;
}
```

오류가 반환되면 현재 source, baseline, binding, dirty 상태는 그대로 유지한다.
