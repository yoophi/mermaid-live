# 리서치: Mermaid 차트 파일 열기 및 저장

## 결정 1: 프론트엔드 문서 entity가 편집 상태의 권위 소스다

**결정**: 각 webview는 `source`, `baselineSource`, 선택적 파일 연결을 가진 하나의 문서 모델을 소유하고 `source !== baselineSource`로 dirty 상태를 계산한다.

**근거**: CodeMirror 입력은 프론트엔드에서 발생하며 즉시 상태에 반영되어야 한다. 저장, 열기, 다시 불러오기, 다른 이름으로 저장, 취소가 모두 같은 baseline과 후속 작업을 공유하므로 하나의 순수 상태 모델로 두면 전이를 독립적으로 테스트할 수 있다. 창/탭마다 webview가 있으므로 별도 전역 문서 registry도 필요 없다.

**검토한 대안**:

- `EditorPage`의 source 문자열만 확장: 빠르지만 파괴적 작업의 순서와 실패 후 상태를 테스트하기 어렵다.
- Rust `AppState`에 실시간 편집 상태 저장: 모든 키 입력에 IPC가 필요하고 window별 registry가 추가되어 과도하다.
- open/save별 독립 hook: pending intent, dirty 상태, conflict 후속 작업이 중복되어 일관성이 깨지기 쉽다.

## 결정 2: 열기·저장·닫기 보호를 하나의 FSD feature로 조정한다

**결정**: `features/manage-chart-document`가 메뉴 이벤트, Tauri command 호출, 손실 방지, 충돌 결정, close handshake, 창 제목 동기화를 조정한다. `entities/chart-document`는 순수 타입과 상태 전이만 제공한다.

**근거**: 이 동작들은 동일 문서 상태와 pending 사용자 의도를 공유하는 하나의 워크플로다. page는 controller를 조합하고 workspace에는 source와 edit callback만 전달하면 된다.

**검토한 대안**:

- `features/open-diagram`, `features/save-diagram`, `features/protect-unsaved`로 분할: 단순 UI 액션처럼 보이지만 상태 전이와 후속 의도가 강하게 결합되어 feature 간 직접 의존을 만들 수 있다.
- page에 모든 조정 로직 배치: 화면 조합과 사용자 워크플로 책임이 섞인다.

## 결정 3: 파일 규칙과 조건부 저장은 native application 경계에 둔다

**결정**: domain은 지원 확장자, 파일 revision, 저장 결과와 오류를 정의한다. application use case는 repository port를 통해 open/save/force-save를 수행한다. Tauri command는 DTO 변환만 담당하고 dialog 및 filesystem은 outbound adapter에 둔다.

**근거**: 현재 raw command의 `read_to_string`과 saver의 dialog·확장자·`std::fs::write` 결합은 핵심 규칙을 테스트하기 어렵게 한다. port를 사용하면 fake repository로 충돌과 실패 상태를 빠르게 검증할 수 있고 저장소의 hexagonal architecture 규칙을 충족한다.

**검토한 대안**:

- 기존 Tauri command를 계속 확장: 구현량은 적지만 domain/application 경계를 우회하고 오류 분기가 UI 문자열에 결합된다.
- 프론트엔드 파일 API 사용: desktop 권한·native dialog·안전한 교체를 여러 플랫폼에서 일관되게 다루기 어렵다.

## 결정 4: 외부 변경은 내용 지문으로 판정한다

**결정**: open 및 save 성공 결과에 `FileRevision`을 포함한다. revision의 권위 값은 파일 바이트의 SHA-256이며 byte length와 modified time은 진단용 보조 정보로만 사용한다. 저장 직전 현재 disk revision을 예상 revision과 비교한다.

**근거**: mtime과 크기만 사용하면 짧은 간격의 같은 크기 변경이나 파일 교체를 놓칠 수 있다. 지문은 baseline 원문 전체를 IPC마다 다시 보내지 않으면서 실제 내용 변경을 안정적으로 식별한다. metadata만 바뀌고 bytes가 같으면 충돌로 보지 않는다.

**검토한 대안**:

- 수정 시각과 크기: 빠르지만 false negative가 가능하다.
- baseline 원문과 disk 원문의 직접 비교: 충돌 정확성은 높지만 매 저장 요청에 baseline 전체를 전달해야 한다.
- 지속적 file watcher: 명세는 저장 직전 확인만 요구하며 자체 저장 이벤트 억제와 lifecycle 복잡도가 불필요하다.

## 결정 5: 저장은 조건부 결과와 명시적 강제 덮어쓰기를 사용한다

**결정**: 연결 파일 저장 요청에는 예상 revision이 필수다. 결과는 `saved`, `conflict`, `cancelled` 또는 typed failure다. conflict는 disk snapshot을 포함하며 덮어쓰기 선택 후에만 `force=true` 요청을 허용한다.

**근거**: 사용자의 결정 전에는 디스크와 편집 상태 어느 쪽도 바꾸지 않는다. 성공 응답 후에만 frontend baseline/revision을 갱신하면 실패·취소 경로도 결정적이다.

**검토한 대안**:

- 외부 변경 발견 즉시 저장 실패: 안전하지만 사용자가 앱 안에서 해결할 수 없다.
- 자동 병합: 단일 Mermaid 원문에 대한 합의된 병합 UX가 없고 범위를 크게 늘린다.
- 외부 변경 무시: FR-017/018과 데이터 손실 기준을 위반한다.

## 결정 6: 원본 직접 쓰기 대신 임시 sibling 파일 교체를 사용한다

**결정**: 대상과 같은 디렉터리에 고유한 임시 파일을 배타적으로 만들고 전체 내용을 기록·flush·sync한 뒤 대상 파일을 교체한다. 실패 시 원본은 유지하고 임시 파일은 정리한다. 플랫폼별 replace 차이는 outbound adapter에 캡슐화하고 tempdir 통합 테스트로 검증한다.

**근거**: 기존 `std::fs::write`는 원본을 먼저 truncate하므로 부분 쓰기나 프로세스 실패에서 내용을 잃을 수 있다. 같은 디렉터리 교체는 파일시스템 경계를 넘는 rename 문제도 피한다.

**검토한 대안**:

- 원본에 직접 쓰기: 가장 단순하지만 보존 요구를 충족하지 못한다.
- 임시 디렉터리에서 작성 후 이동: 다른 파일시스템이면 원자적 rename을 보장하지 못한다.
- 매번 백업 파일 유지: 복구본을 제외하기로 한 범위를 벗어난다.

## 결정 7: native close 요청과 프론트엔드 승인 사이에 handshake를 둔다

**결정**: native `CloseRequested`를 보류하고 해당 webview에 close intent를 보낸다. 프론트엔드가 clean 상태이거나 저장/폐기 결정을 완료하면 close 승인 command를 호출한다. infrastructure는 window별 one-shot authorization을 보관해 다음 close만 통과시킨다.

**근거**: native 계층만 OS close를 안정적으로 막을 수 있고 프론트엔드만 현재 dirty 상태를 안다. one-shot 승인은 무한 close 이벤트 루프와 다른 창의 승인 재사용을 방지한다.

**검토한 대안**:

- browser `beforeunload`: Tauri webview와 custom 세 방향 선택에서 동작이 일관되지 않다.
- Rust가 dirty 상태를 소유: 편집 상태 IPC 동기화가 추가된다.
- 닫을 때 자동 저장: 사용자가 합의한 명시적 저장 정책과 충돌한다.

## 결정 8: native dialog의 세 방향 custom 버튼을 사용한다

**결정**: 설치된 `tauri-plugin-dialog`의 custom Yes/No/Cancel 결과를 adapter에서 각각 저장/저장 안 함/취소 및 다시 불러오기/덮어쓰기/취소로 매핑한다. domain/application은 Tauri dialog 타입을 알지 않는다.

**근거**: OS 파일 선택창과 일관된 키보드 접근성을 유지하며 별도 modal primitive를 추가하지 않아도 된다. adapter mapping은 단위 테스트가 가능하다.

**검토한 대안**:

- 프론트엔드 custom dialog: 디자인 자유도는 높지만 이번 기능만을 위한 modal primitive와 focus management가 추가된다.
- 두 단계 이진 dialog: 인지 부담이 커지고 취소 의미가 모호해진다.

## 결정 9: 문서 모델과 native 파일 규칙을 각각 자동 테스트한다

**결정**: frontend 순수 문서 상태 전이에 Vitest를 도입하고, native domain/application에는 fake repository 단위 테스트와 tempdir 통합 테스트를 둔다.

**근거**: dirty/baseline/association 전이와 충돌·실패 시 원본 보존은 회귀 위험이 높다. 수동 검증만으로 모든 분기를 반복 확인하기 어렵다.

**검토한 대안**:

- 기존 typecheck와 Rust test만 사용: 프론트엔드 상태 기계를 실행 검증하지 못한다.
- 전체 native UI 자동화 우선: 메뉴와 OS dialog 자동화는 플랫폼별로 취약하고 초기 비용이 크다.
