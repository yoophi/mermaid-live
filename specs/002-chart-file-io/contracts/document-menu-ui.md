# 계약: 문서 메뉴와 사용자 경험

## 메뉴 항목

### 열기…

- 위치: File/파일 메뉴
- 단축키: `Cmd/Ctrl+O`
- 동작: 현재 활성 편집기에 open intent를 전달한다.
- dirty 문서이면 파일 선택 전에 손실 방지 결정을 완료한다.

### 저장

- 위치: File/파일 메뉴
- 단축키: `Cmd/Ctrl+S`
- 연결 파일이 있으면 같은 파일에 조건부 저장한다.
- 제목 없는 문서이면 Save As dialog를 연다.

### 다른 이름으로 저장…

- 위치: File/파일 메뉴
- 단축키: `Cmd/Ctrl+Shift+S`
- 현재 파일 연결 유무와 관계없이 Save As dialog를 연다.
- 성공하면 새 경로를 현재 문서 연결로 사용한다.

## 손실 방지 dialog

### 표시 조건

- dirty 문서에서 열기 또는 window 닫기를 요청했을 때 표시한다.

### 내용

- 현재 display name을 포함한다.
- 저장되지 않은 변경이 사라질 수 있음을 명확히 알린다.

### 버튼

- `저장`: 저장 성공 후 pending intent 계속
- `저장 안 함`: baseline 이후 변경을 폐기하고 pending intent 계속
- `취소`: pending intent 중단, 문서 유지

### 제약

- 저장 dialog 취소 또는 저장 실패는 `취소`와 동일하게 pending intent를 중단한다.
- 키보드로 모든 버튼에 접근할 수 있어야 한다.

## 외부 변경 충돌 dialog

### 표시 조건

- 연결 파일 저장 직전 예상 revision과 disk revision이 다를 때 표시한다.

### 버튼

- `다시 불러오기`: conflict disk snapshot을 현재 source/baseline/binding에 적용
- `덮어쓰기`: 현재 source를 동일 경로에 명시적으로 force save
- `취소`: disk와 문서 상태 모두 유지

### 제약

- 기본 경로가 조용한 덮어쓰기가 되어서는 안 된다.
- 다시 불러오기 적용 전에 재검증이 실패하면 현재 문서를 유지하고 오류를 표시한다.

## 창 제목

```text
제목 없음 — Mermaid Live
diagram.mmd — Mermaid Live
● diagram.mmd — Mermaid Live
```

- `●`는 dirty 예시이며 플랫폼에서 명확한 동등 표시로 바꿀 수 있다.
- open/save/reload 성공 후 즉시 갱신한다.
- 취소 또는 실패에서는 기존 제목을 유지한다.
- 클립보드 bootstrap 문서는 제목 없는 문서로 취급하며 temp 파일명을 표시하지 않는다.

## 오류 표시

- 파일 읽기/쓰기/교체 실패는 사용자가 인지할 수 있는 dialog로 표시한다.
- 오류 메시지는 대상 기본 파일명과 사용자가 취할 수 있는 다음 행동을 포함한다.
- 내부 stack, 원시 OS 코드, 불필요한 절대 경로는 기본 메시지에 노출하지 않는다.

## 범위 밖

- 최근 파일 메뉴
- 자동 저장 및 복구
- drag and drop
- Finder/Explorer 파일 연결
- Markdown 파일
- 여러 Mermaid 차트가 포함된 한 문서
