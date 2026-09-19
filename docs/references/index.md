# References 인덱스

외부 라이브러리·기술의 **요약 캐시**. 에이전트가 기억에 의존해 API를 쓰지 않도록 하는 것이 목적이다 (AGENTS.md §6).

## 규칙

1. **우리 결정을 여기 적지 않는다.** 그건 `../design-docs/`에 간다
2. 각 문서 상단에 **출처 URL과 확인 날짜**를 적는다
3. 확실하지 않은 내용은 "미검증"이라고 명시한다
4. 문서 끝에 "우리 프로젝트와 관련해 주의할 점"을 둔다 — 캐시를 우리 맥락에 연결하는 부분

## 목록

| 문서 | 다루는 것 | 왜 필요한가 |
|---|---|---|
| [trystero-llms.txt](trystero-llms.txt) | Trystero API 전체, 전략 비교, TURN 설정 | 프로젝트의 심장. API를 틀리면 연결이 안 된다 |
| [webrtc-datachannel-llms.txt](webrtc-datachannel-llms.txt) | 채널 신뢰성 옵션, `bufferedAmount`, NAT 유형 | `ordered:false` 설정 하나가 지연 특성을 바꾼다 |
| [webgl2-llms.txt](webgl2-llms.txt) | 인스턴스 렌더링, GLSL ES 3.00, web-sys 바인딩 | 엔진 없이 직접 짜므로 (DD-004) |
| [cc0-asset-sources-llms.txt](cc0-asset-sources-llms.txt) | Kenney/OGA/itch.io/freesound, 라이선스 표 | 에셋을 상상해서 쓰지 않기 위해 (H7) |

## 아직 없는 것 (필요해지면 만든다)

- **WebAudio** — `PannerNode`, 모바일 자동재생 정책. DD-008 구현 시
- **Tiled TMX 형식** — 맵 변환 도구 작성 시 (DD-009)
- **wasm-bindgen** — 버전 핀 맞추기 등 함정이 있다. 빌드가 자주 깨지면

## 주의

이 문서들은 **작성 시점의 스냅샷**이다. 라이브러리가 업데이트되면 낡는다.

API가 문서와 다르게 동작하면:
1. 실제 소스/공식 문서를 확인한다
2. 이 캐시를 갱신한다 (확인 날짜도 갱신)
3. 그 다음에 코드를 고친다

**캐시를 믿고 버그를 찾느라 시간을 쓰는 게 가장 나쁜 경우다.**
