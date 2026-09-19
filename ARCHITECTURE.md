# ARCHITECTURE.md — PEEKABOOM

시스템이 어떤 조각으로 나뉘고, 그 조각들이 어떻게 맞물리는지. 개별 결정의 *근거*는 `docs/design-docs/`에 있다.

---

## 1. 전체 그림

```
┌─────────────────────────────────────────────────────────────┐
│  GitHub Pages (정적 파일만)                                  │
│  index.html · app.js · peekaboom_bg.wasm · assets/*.png/ogg │
└─────────────────────────────────────────────────────────────┘
                 │ 최초 1회 다운로드
                 ▼
┌─────────────────────────────────────────────────────────────┐
│  브라우저 탭 (= 하나의 피어)                                  │
│                                                              │
│  ┌──────────┐   JS ↔ WASM 경계 (wasm-bindgen)                │
│  │ web/*.js │◄──────────────────────────────────┐           │
│  │ Trystero │                                    │           │
│  └────┬─────┘                                    │           │
│       │                              ┌───────────▼─────────┐ │
│       │                              │ crates/app          │ │
│       │                              │ (조립 + 루프)        │ │
│       │                              └──┬───┬───┬────────┬─┘ │
│       │                                 │   │   │        │   │
│       │                       ┌─────────▼┐ ┌▼──────┐ ┌───▼──┐│
│       │                       │ net      │ │render │ │input ││
│       │                       └────┬─────┘ └───┬───┘ └──┬───┘│
│       │                            │           │        │    │
│       │                       ┌────▼───────────▼────────▼──┐ │
│       │                       │ crates/sim (순수 로직)      │ │
│       │                       └────────────────────────────┘ │
└───────┼──────────────────────────────────────────────────────┘
        │ WebRTC DataChannel (SCTP)
        ▼
   다른 피어들 (최대 8명)
```

시그널링(공개 Nostr 릴레이)은 **방에 들어가는 순간에만** 쓰인다. 연결이 맺어지면 릴레이는 경로에서 빠지고, 게임 트래픽은 피어 간 직접 흐른다.

---

## 2. 크레이트 분해

### `crates/sim` — 게임 로직

의존성 없음(no_std 가능). 네이티브에서 테스트된다.

```rust
pub struct World {
    pub tick: u32,
    pub players: SlotMap<PlayerId, Player>,
    pub map: MapGrid,
    pub phase: Phase,     // Lobby | Hiding | Seeking | RoundOver
    pub rules: RuleSet,   // 모드별 파라미터
}

impl World {
    pub fn step(&mut self, inputs: &InputFrame) -> Vec<SimEvent>;
    pub fn visible_to(&self, viewer: PlayerId) -> VisibilitySet;
}
```

핵심 성질:

- **결정적(deterministic)**: 같은 `World` + 같은 `InputFrame` → 같은 결과. 부동소수점 대신 Q16.16 고정소수점.
- **시간 개념이 없다**: `step()`은 1틱을 진행할 뿐. 실제 시계는 `app`이 쥔다. 고정 틱레이트 **30 Hz**.
- **네트워크를 모른다**: 호스트든 클라이언트든 같은 `sim`을 돌린다.

### `crates/net` — 프로토콜 + 권위

두 가지 역할(`Host` / `Client`)을 가진 상태기계.

```
Host                              Client
────                              ──────
inputs 수집 (자기 것 + 원격)        로컬 입력 → 즉시 예측 적용
   │                                  │
   ▼                                  ▼
sim.step()  ← 유일한 진실           sim.step() (예측)
   │                                  │
   ▼                                  │
피어별 가시성 필터링                   │
   │                                  ▼
   └──► 개인화된 스냅샷 ──────────► 수신 → 재조정(reconciliation)
```

호스트는 각 피어에게 **서로 다른 스냅샷**을 보낸다. 술래에게는 자기 시야 안의 숨은 사람만, 숨은 사람에게는 자기 근처 상황만. 이게 H4(월핵 차단)의 구현이다. 자세한 건 `docs/design-docs/authority-and-visibility.md`.

패킷:

| 방향 | 타입 | 주기 | 신뢰성 |
|---|---|---|---|
| C→H | `Input` | 30 Hz | unreliable, unordered |
| H→C | `Snapshot` (델타) | 15 Hz | unreliable, unordered |
| H→C | `Event` (잡힘, 라운드 종료) | 발생 시 | reliable, ordered |
| any | `Chat`, `Emote` | 발생 시 | reliable |
| any | `HostHandoff` | 호스트 이탈 시 | reliable |

DataChannel을 두 개 연다: `ordered: true`(이벤트용)와 `ordered: false, maxRetransmits: 0`(스냅샷/입력용). 지연 누적을 막기 위해서다.

### `crates/render` — WebGL2 렌더러

- 스프라이트 배칭: 텍스처 아틀라스 1장 + 인스턴스 버퍼 1개 → 드로우콜 1~3회/프레임
- 카메라는 로컬 플레이어 추적, 뷰포트 밖 컬링
- 시야(fog of war)는 프래그먼트 셰이더에서 처리. `sim`이 준 `VisibilitySet`을 텍스처로 올림
- `sim::World`를 **읽기만** 한다. 렌더러가 상태를 바꾸면 결정성이 깨진다

### `crates/input` — 입력 추상화

```rust
pub struct InputIntent {
    pub move_dir: Vec2Fixed,   // 정규화된 이동 방향
    pub action: ActionFlags,   // 상호작용 / 대시 / 감정표현
    pub look: Option<Vec2Fixed>,
}
```

키보드, 가상 조이스틱(터치), 게임패드가 전부 이 하나로 수렴한다. `sim`은 어느 쪽에서 왔는지 모른다(H6).

### `crates/app` — 조립

`wasm-bindgen`으로 노출되는 진입점. 프레임 루프, 고정 틱 누산기, JS 콜백 등록을 담당한다.

---

## 3. 왜 Rust/WASM인가, 왜 wasm-bindgen인가

**언어 선택의 근거는 [DD-001](docs/design-docs/language-choice.md)에 있다.** 요약: 결정적 시뮬레이션을 타입으로 강제할 수 있다는 것이 유일하게 대체 불가능한 이득이고, "성능 때문에"는 근거가 아니다. 탈출 조건도 거기 적혀 있다.

아래는 그 다음 질문 — Rust 안에서 왜 wasm-bindgen인가.

### macroquad가 아닌 이유

`macroquad`/`miniquad`는 `wasm-bindgen`을 쓰지 않고 자체 JS 심(shim)을 통해 브라우저와 통신한다. 편하지만 **임의의 JS 라이브러리를 부르려면 별도 `.js` 플러그인을 직접 작성**해야 한다.

우리는 Trystero(순수 JS, WebRTC 래퍼)를 반드시 호출해야 한다. 이게 프로젝트의 심장이다. 따라서:

- **wasm-bindgen 유지** — JS 상호운용이 1급 기능
- 렌더링은 `web-sys`의 WebGL2 바인딩으로 직접 구현 (엔진 미채택)

트레이드오프: 스프라이트 배칭·텍스처 로딩을 직접 짜야 한다(약 800~1200줄 예상). 대신 번들이 작아지고(H5), 엔진의 프레임 루프와 우리의 고정 틱 네트워크 루프가 싸우지 않는다.

기록: `docs/design-docs/rendering-pipeline.md` (엔진 선택), `docs/design-docs/language-choice.md` (언어 선택)

---

## 4. 호스트 선출과 이양

호스트는 **방을 처음 만든 피어**. 호스트가 나가면:

1. 남은 피어들이 `selfId` 사전순 최소값을 새 호스트로 뽑는다 (합의 불필요, 결정적)
2. 새 호스트는 자신이 마지막으로 받은 스냅샷 틱부터 시뮬레이션 재개
3. 최대 2틱(약 66ms) 되감기 발생 — 라운드는 이어진다

호스트가 곧 심판이므로 **호스트는 치트할 수 있다**. 이건 설계상 받아들인 리스크다. 근거와 완화책은 `docs/SECURITY.md` §3.

---

## 5. 시그널링 경로

```
1순위: Trystero 기본 전략 (Nostr 릴레이 다수) — 릴레이 리던던시 가장 높음
2순위: MQTT 전략으로 자동 폴백
3순위: 수동 코드 교환 (SDP를 Base64로 압축 → 카톡으로 붙여넣기)
```

3순위는 UI가 못생겨도 반드시 남긴다. 공개 릴레이는 언젠가 죽고, 그날에도 게임은 돌아가야 한다(RELIABILITY.md §2).

NAT 대칭 환경에서는 직접 연결이 실패할 수 있다. TURN이 필요한데 TURN은 서버다 → H1 위반. **우리는 TURN을 운영하지 않는다.** 대신 연결 실패를 감지해 "이 네트워크에서는 연결이 안 돼요. 다른 와이파이나 셀룰러로 시도해보세요"를 정직하게 띄운다. 사용자가 자기 TURN을 넣을 수 있는 설정 필드는 제공한다.

---

## 6. 빌드 · 배포

```
소스 → wasm-pack build --release --target web
     → wasm-opt -Oz
     → web/ 로 복사 + 에셋 해시 파일명
     → GitHub Actions → gh-pages 브랜치 push
```

- 에셋은 내용 해시 파일명(`atlas.a3f91c.png`)으로 영구 캐시
- `index.html`만 no-cache
- CI에서 번들 크기 측정 → H5(1.5MB) 초과 시 빌드 실패

---

## 7. 이 구조가 감당 못 하는 것 (알고 있는 한계)

| 한계 | 현재 대응 |
|---|---|
| 8명 초과 | 풀메시 연결 수가 n² — 8명(28연결)이 상한. 그 이상은 릴레이 토폴로지 필요, 미지원 |
| 호스트 치트 | 미해결. SECURITY.md §3 |
| 대칭 NAT | 미해결. 정직한 에러 + 사용자 TURN 입력 |
| 관전자 모드 | 미설계 |
| 랭킹/전적 | 서버가 없으므로 영구 저장 불가. localStorage 로컬 전적만 |
