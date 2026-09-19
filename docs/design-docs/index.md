# Design Docs 인덱스

기술 결정의 **근거**를 담는 곳. "무엇을 만드는가"는 `../product-specs/`에 있다.

## 형식

각 문서는 ADR(Architecture Decision Record)에 가깝게 쓴다:

```
- 상태: 제안됨 | 채택됨 | 폐기됨 | 대체됨(→ DD-NNN)
## 문제      — 무엇이 문제였나
## 결정      — 무엇으로 정했나
## 대안      — 검토했으나 안 쓴 것과 그 이유
## 리스크    — 받아들인 비용
```

**결정이 뒤집히면 문서를 지우지 않는다.** 상태를 `폐기됨`으로 바꾸고 이유를 덧붙인다. 왜 그 길로 안 갔는지가 다음 사람에게 필요한 정보다.

## 목록

| ID | 문서 | 상태 | 요약 |
|---|---|---|---|
| — | [core-beliefs.md](core-beliefs.md) | 살아있음 | 모든 결정의 전제. 8개 믿음 |
| DD-001 | [language-choice.md](language-choice.md) | **채택됨 (조건부)** | Rust+WASM vs JS. 탈출 조건 포함. "성능 때문에"는 근거가 아니다 |
| DD-002 | [authority-and-visibility.md](authority-and-visibility.md) | 채택됨 | 호스트 권위 + 피어별 가시성 필터링. 월핵 차단의 핵심 |
| DD-003 | [netcode-model.md](netcode-model.md) | 채택됨 | 예측·재조정·보간. 30Hz 틱 / 15Hz 스냅샷 |
| DD-004 | [rendering-pipeline.md](rendering-pipeline.md) | 채택됨 | 엔진 미채택, WebGL2 직접. 왜 Bevy/macroquad가 아닌가 |
| DD-005 | [input-abstraction.md](input-abstraction.md) | 채택됨 | `InputIntent` 단일 표현. 모바일 터치 설계 |
| DD-006 | [repo-layout.md](repo-layout.md) | 채택됨 | 정확한 디렉터리 트리·빌드 명령·CI 단계. **백지에서 시작할 때 읽는다** |
| DD-007 | [host-handoff.md](host-handoff.md) | 채택됨 (타협 포함) | 호스트 이탈 대응. 사전순 선출 + 저해상도 Shadow |
| DD-008 | [audio-model.md](audio-model.md) | **제안됨** | 소리가 곧 정보 채널. 정보층/표현층 분리. G1 검증 대기 |
| DD-009 | [map-design.md](map-design.md) | 채택됨 | 손으로 만든 맵. 절차적 생성 기각. 64×40 |

> DD-001은 원래 결번이었다. 결정은 프로젝트 시작 시점에 내려졌는데 근거가 기록되지 않아
> "왜 WASM인가"가 나중에 다시 질문으로 돌아왔다. 그래서 뒤늦게 채웠다.
> **기록되지 않은 결정은 결정이 아니다** (CB-7). 결번을 발견하면 비워두지 말고 채운다.

## 상태가 '제안됨'인 것

DD-008(오디오)만 제안 상태다. 이 문서의 전제(PRODUCT_SENSE G1)가 플레이테스트에서 검증되기 전까지 확정하지 않는다. **G1이 틀리면 게임의 절반을 다시 설계해야 한다** — 그래서 확정으로 적지 않는다.

## 아직 안 쓴 것 (필요해지면 씀)

- 리플레이 — 결정적 시뮬(CB-6) 덕에 입력 로그만으로 가능하지만 우선순위 낮음
- 절차적 맵 생성 — DD-009에서 기각. 맵 3~4개로도 지겨워지면 그때 다시
- 관전자 모드 — 유령이 그 역할을 겸한다. 순수 관전이 필요해지면
