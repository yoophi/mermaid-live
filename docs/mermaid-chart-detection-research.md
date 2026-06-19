# Mermaid Chart Detection Research

## Purpose

클립보드에 있는 텍스트가 Mermaid chart인지 빠르게 판별하기 위한 조사 내용이다. 현재 목표는 텍스트를 `trim()` 한 뒤 첫 번째 단어를 기준으로 Mermaid 다이어그램 선언인지 확인하는 것이다.

## Official Syntax Rule

Mermaid 공식 문서의 `Diagram Syntax` 페이지는 Mermaid 다이어그램 정의가 다이어그램 타입 선언으로 시작한다고 설명한다. 예외적으로 frontmatter 설정이 다이어그램 선언 앞에 올 수 있다.

또한 Mermaid 내부 `detectType` 구현은 타입 판별 전에 다음 요소를 제거한다.

- YAML frontmatter
- Mermaid directive, 예: `%%{init: ...}%%`
- Mermaid comment, 예: `%% comment`

따라서 단순 `trim()` 후 첫 단어 판별은 기본 필터로는 충분하지만, 실제 사용자 클립보드 입력을 안정적으로 처리하려면 위 전처리를 고려하는 것이 좋다.

## Recommended Start Tokens

첫 번째 토큰이 아래 값 중 하나라면 Mermaid chart 후보로 볼 수 있다.

```ts
const MERMAID_START_TOKENS = new Set([
  "graph",
  "flowchart",
  "sequenceDiagram",
  "classDiagram",
  "stateDiagram",
  "stateDiagram-v2",
  "erDiagram",
  "journey",
  "gantt",
  "pie",
  "quadrantChart",
  "requirementDiagram",
  "gitGraph",
  "mindmap",
  "timeline",
  "zenuml",
  "sankey-beta",
  "xychart-beta",
  "block-beta",
  "packet-beta",
  "kanban",
  "architecture-beta",
  "radar-beta",
  "eventModel",
  "treemap-beta",
  "venn",
  "ishikawa",
  "wardley",
  "tree",
  "info",
]);
```

## Special Case: C4

C4 diagram은 선언이 `C4Context`, `C4Container`, `C4Component`, `C4Dynamic`, `C4Deployment`처럼 여러 형태로 시작할 수 있다. 그래서 정확한 토큰 목록을 모두 나열하기보다 첫 토큰이 `C4`로 시작하는지 확인하는 방식이 실용적이다.

```ts
const isC4Diagram = firstToken.startsWith("C4");
```

## Detection Guidance

`text.includes("graph")` 같은 포함 검사는 오탐 가능성이 높으므로 피하는 것이 좋다. 일반 문장 안에 Mermaid 키워드가 들어가도 chart로 판별될 수 있기 때문이다.

권장 흐름은 다음과 같다.

```ts
function isLikelyMermaidChart(text: string) {
  const normalized = text.trim();
  const firstToken = normalized.split(/\s+/)[0];

  return MERMAID_START_TOKENS.has(firstToken) || firstToken.startsWith("C4");
}
```

사용자 입력 품질을 더 높게 보려면 Mermaid의 실제 parser인 `mermaid.parse(text, { suppressErrors: true })`를 함께 사용하는 것이 가장 정확하다. 첫 토큰 검사는 빠른 후보 판별, parser 검증은 최종 확인 역할로 나누면 된다.

## Sources

- Mermaid Diagram Syntax: https://mermaid.js.org/intro/syntax-reference.html
- Mermaid Flowchart Syntax: https://mermaid.js.org/syntax/flowchart.html
- Mermaid detectType source: https://raw.githubusercontent.com/mermaid-js/mermaid/develop/packages/mermaid/src/diagram-api/detectType.ts
