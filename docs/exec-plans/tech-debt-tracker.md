# 기술 부채 트래커

**부채를 만드는 건 괜찮다. 기록하지 않는 게 문제다.**

에이전트가 "일단 이렇게 해두고 나중에"라고 판단했으면 반드시 여기 한 줄 남긴다. 플레이스홀더 코드로 숨기지 않는다 (AGENTS.md §6).

---

## 형식

```
| ID | 위치 | 내용 | 왜 미뤘나 | 터지는 조건 | 상태 |
```

**"터지는 조건"이 가장 중요하다.** 언제 이 부채가 실제 문제가 되는지 적어두면, 그때가 오기 전에 손댈 수 있다.

---

## 열린 부채

| ID | 위치 | 내용 | 왜 미뤘나 | 터지는 조건 | 상태 |
|---|---|---|---|---|---|
| TD-001 | 에셋 전반 | `asset-manifest.md`의 파일 경로가 하나도 검증되지 않음. 공개 페이지 정보만 기록됨 | M1은 에셋 없이 진행 | 코드에서 에셋 파일을 처음 참조할 때 | **열림** |
| TD-002 | `docs/generated/net-protocol.md` | 생성 문서인데 손으로 쓴 초안 상태. 생성기(`tools/gen_protocol_doc.rs`) 없음 | `wire.rs` 구현 전 | `wire.rs` 작성 시 | **열림** |
| TD-003 | 아트 디렉션 | 파스텔(Scribble/Top-down) vs 1비트 미결정. DESIGN.md는 파스텔 전제로 쓰임 | 프로토타입 없이 정할 수 없음 | M2 아트 스파이크 | **열림** |
| TD-004 | 사운드 | 게임플레이 SFX(발소리/대시/잡힘) 출처 미확보 | Kenney Interface Sounds에 없음 | M2에서 소음 시스템 구현할 때 | **열림** |
| TD-005 | RELIABILITY §3 | `HostShadow`가 4타일 정밀도 위치를 후보 호스트에게 노출. DD-002와 긴장 관계 | 호스트 이양의 다른 해법을 못 찾음 | 플레이테스트에서 악용 사례 발생 시 | **열림 (의도된 타협)** |
| TD-006 | 성능 수치 전반 | DD-004의 성능 예산, net-protocol의 대역폭이 전부 계산값/목표값. 측정 안 됨 | 구현 전 | M1/M3 측정 시 갱신 | **열림** |
| TD-007 | 브라우저 차이 | `webrtc-datachannel-llms.txt`의 Safari/Firefox 동작 차이가 일반론. 실측 아님 | 기기 미확보 | M1 매트릭스 채울 때 | **열림** |
| TD-008 | 밸런스 수치 | hide-and-seek / rumble의 모든 숫자가 미검증 추측치 | 플레이테스트 전 | M4 플레이테스트 (exec-plan 004) | **열림** |
| TD-009 | DD-008 오디오 | 문서 상태가 '제안됨'. PRODUCT_SENSE G1이 검증돼야 확정된다 | 플레이테스트 전 | exec-plan 002의 G1 실험 | **열림 (의도됨)** |
| TD-010 | DD-009 맵 | 맵 테마(사무실/학교/마트) 미확정. 에셋 스타일(TD-003)과 연동됨 | 에셋 확정 전 | 맵 제작 착수 시 (M4) | **열림** |
| TD-011 | `tools/` 전체 | `pack_atlas` / `check_map` 미작성 (`check_size.sh`, `check_assets.sh`는 001에서 작성함) | 코드 이전 단계 | 각 도구를 처음 필요로 하는 계획에서 | **열림** |
| TD-012 | `crates/net`, `crates/sim`, `crates/render` | `wire.rs`/`host.rs`/`client.rs`(net), `map.rs`/`vis.rs`(sim), `atlas.rs`(render) 미작성. 001은 호스트 권위·시야·지도·텍스처가 없어서 필요 없었음 | 001 범위 밖 (exec-plans/active/001 "하지 않는 것") | M2(002 core-loop) 착수 시 | **열림** |
| TD-013 | `web/index.html` CSP | `connect-src`가 `wss: https:`로 넓게 열려 있음. Trystero nostr 전략의 정확한 릴레이 호스트 목록으로 좁혀야 하는데, 그 목록이 라이브러리 내부(`node_modules/trystero/src/nostr.js`의 `defaultRelayUrls`)에 있고 버전마다 바뀔 수 있어 지금은 고정하지 않음 | CSP를 정밀하게 좁히려면 릴레이 목록을 우리가 직접 핀(pin)해야 하는데, 그러면 트리스테로 업데이트마다 깨질 위험이 생김 | 실제 공격 표면 검토가 필요해지는 시점 (배포 전 보안 리뷰) | **열림** |
| TD-014 | `web/net.js`, `web/main.js` | ICE 연결 실패(대칭 NAT)를 감지해 "이 네트워크에서는 연결이 안 돼요" 안내를 띄우는 UI가 없다. Trystero `getPeers()`는 **이미 연결된** 피어만 주기 때문에, 연결 시도 중 실패하는 피어의 `RTCPeerConnection`에는 애초에 접근할 공개 API가 없다(0.21.8 기준) | 001 체크리스트가 요구하지 않음 (연결 매트릭스는 사람이 눈으로 판정). ARCHITECTURE.md §5가 약속하는 안내 UI는 더 큰 작업 | 001 연결 매트릭스에서 실패율이 50%를 넘는 조합이 나올 때 (그때 어차피 이 UI가 필요해짐) | **열림** |
| TD-015 | `e2e/connection.spec.js` | 이 샌드박스 OS(Ubuntu 20.04)가 Playwright 공식 지원 밖이라 `npx playwright install chromium`이 실패한다. 그래서 이 E2E 테스트를 **로컬에서 실제로 실행해 통과를 확인하지 못했다** — 코드 리뷰(API 시그니처 대조)만 했다 | 샌드박스 환경 제약 | CI(ubuntu-latest, Playwright 공식 지원)에서 처음 돌 때. 실패하면 여기 상태를 갱신 | **열림 (미검증)** |

---

## 해결된 부채

| ID | 내용 | 어떻게 해결됐나 |
|---|---|---|
| — | **DD-001 결번** — 언어 선택(Rust+WASM vs JS) 결정의 근거가 기록되지 않아 "왜 WASM인가"가 나중에 질문으로 돌아옴 | `design-docs/language-choice.md` 작성. 탈출 조건까지 명시. 교훈: **결번을 발견하면 비워두지 말고 채운다** |
| — | **맵 크기 불일치** — DD-009에서 64×40으로 줄였는데 ARCHITECTURE·DD-003·DD-004·hide-and-seek이 128×128로 남아 있었음 | 4개 문서 동시 수정. 교훈: **수치를 바꾸는 결정은 그 수치를 인용한 문서를 전부 찾아 같이 고친다** (CB-7) |
| — | **`rust-toolchain.toml`이 1.82로 고정돼 있었는데 `wasm-bindgen-cli 0.2.128`(Cargo.toml에 정확히 핀 고정)의 MSRV는 1.86** — 로컬 샌드박스에는 이미 다른 경로로 설치된 wasm-bindgen 바이너리가 있어서 안 걸렸지만, GitHub Actions에서 `cargo install wasm-bindgen-cli`를 처음부터 하니 바로 터졌다 (Deploy 워크플로 최초 실행 실패) | `rust-toolchain.toml`을 1.86으로 올렸다(repo-layout.md도 같이 수정). 교훈: **로컬에 이미 깔려있는 도구는 그 도구가 어떤 툴체인으로 빌드됐는지 확인하지 않으면 버전 고정이 실제로 지켜지는지 알 수 없다 — CI의 "깨끗한 상태에서 처음부터"가 이런 걸 잡아낸다** |
| — | **CSP `script-src 'self'`가 WASM 컴파일 자체를 막음** — `WebAssembly.instantiateStreaming()`이 "violates ... script-src 'self'"로 CSP 위반 처리됐다. 로컬 수동 확인(브라우저로 직접 열어보지 않음)으로는 못 잡고 Playwright E2E가 처음으로 잡아냈다 | `'wasm-unsafe-eval'`을 script-src에 추가 (`'unsafe-eval'`과 다르다 — 임의 eval은 여전히 막힘, WASM 컴파일만 허용하는 전용 키워드). 교훈: **E2E를 브라우저에서 실제로 돌려보지 않으면 CSP처럼 "코드는 맞는데 배포 환경에서만 깨지는" 버그를 못 잡는다 — 001에서 E2E를 만들어둔 게 여기서 값을 했다** |
| — | **Trystero 참조 캐시가 실제 설치 버전(0.21.8)과 어긋남** — `docs/references/trystero-llms.txt`가 `room.onPeerJoin = fn`(대입식), `joinRoom`의 3번째 인자를 `{onJoinError, onPeerHandshake}` 옵션 객체, `isInitiator` 플래그 존재를 전제로 적혀 있었다. 실제로는 `room.onPeerJoin(fn)`(함수 호출), 3번째 인자는 `onJoinError` 콜백 그 자체, `onPeerHandshake`/`isInitiator`는 이 버전에 아예 없음 | `npm install` 후 `node_modules/trystero/src/{room,strategy,peer}.js`를 직접 읽고 캐시를 고쳤다. `fast` 채널을 누가 만들지는 `selfId` 사전순 비교로 직접 정하는 것으로 설계 변경(`web/net.js`). 교훈: **레퍼런스 캐시도 캐시일 뿐이다 — 실제 패키지를 설치할 수 있으면 소스를 직접 대조한다** (AGENTS.md §6) |

---

## 부채로 치지 않는 것

명시적으로 "안 하기로 한 것"은 부채가 아니다. 범위 결정이다.

- 랭킹/전적 — 서버가 없어서 구조적으로 불가 (PRODUCT_SENSE)
- 9명 이상 — 풀메시 한계. 의도적 상한 (ARCHITECTURE §7)
- 호스트 치트 — 받아들인 리스크 (SECURITY §3)
- 맵 에디터, 음성 채팅, 봇 — v1 범위 밖

이것들을 여기 적으면 트래커가 "안 하는 것들"로 뒤덮여 진짜 부채가 안 보인다.

---

## 운영 규칙

1. 새 부채는 **만든 PR에서 같이 추가**한다. 나중에 정리하지 않는다
2. "터지는 조건"이 발생하면 상태를 **긴급**으로 바꾸고 exec-plan을 만든다
3. 해결되면 지우지 말고 "해결된 부채"로 옮긴다. 왜 그랬는지가 기록으로 남는다
4. 열린 부채가 15개를 넘으면 **새 기능 작업을 멈추고** 정리 스프린트를 한다
