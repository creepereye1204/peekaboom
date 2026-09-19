# FRONTEND.md

브라우저에서 돌아가는 모든 것: DOM, 캔버스, JS ↔ WASM 경계, 에셋 로딩.

---

## 1. 레이어 구조

```
┌─────────────────────────────────────┐
│ DOM UI  (로비, 메뉴, 토스트)         │  ← 일반 HTML/CSS
├─────────────────────────────────────┤
│ Canvas  (게임 화면, WebGL2)          │  ← Rust가 소유
├─────────────────────────────────────┤
│ DOM HUD (타이머, 버튼, 조이스틱)      │  ← 캔버스 위에 오버레이
└─────────────────────────────────────┘
```

**게임 HUD를 캔버스에 안 그리는 이유**: 텍스트 렌더링과 접근성을 공짜로 얻는다. 터치 타겟도 브라우저가 알아서 처리한다. 대신 DOM 갱신은 프레임당이 아니라 **값이 바뀔 때만** 한다.

조이스틱만 예외로 캔버스 위 별도 `<canvas>` 또는 CSS transform으로 그린다 — 매 프레임 움직이므로 DOM 리플로우를 피한다.

---

## 2. JS ↔ WASM 경계

JS가 하는 일은 **딱 세 가지**. 그 이상 늘어나면 리뷰에서 막는다 (AGENTS.md §4).

### 2-1. Trystero 래핑

```js
// web/net.js
import { joinRoom, selfId } from 'trystero'

export function connect(roomCode, password, wasmHandle) {
  const room = joinRoom(
    { appId: 'peekaboom', password, relayConfig: { redundancy: 4 } },
    roomCode
  )

  room.onPeerJoin = id => wasmHandle.on_peer_join(id)
  room.onPeerLeave = id => wasmHandle.on_peer_leave(id)

  // 신뢰성 채널: 이벤트/채팅/호스트이양
  const sure = room.makeAction('sure')
  sure.onMessage = (data, { peerId }) => wasmHandle.on_reliable(peerId, data)

  // 비신뢰 채널: 스냅샷/입력은 RTCPeerConnection에서 직접 연다
  //  → Trystero의 액션은 신뢰성 채널을 쓰므로 지연이 누적된다 (DD-003)
  attachFastChannels(room, wasmHandle)

  return { room, sure, selfId }
}
```

`attachFastChannels`는 `room.getPeers()`로 `RTCPeerConnection`을 꺼내 `{ordered:false, maxRetransmits:0}` 채널을 별도로 만든다.

### 2-2. 캔버스 생성 · 리사이즈

`ResizeObserver` + `devicePixelRatio`(상한 2.0) → Rust의 `on_resize(w, h, dpr)` 호출.

### 2-3. 오디오 언락

첫 `pointerdown`에서 `AudioContext.resume()`. 한 번 성공하면 리스너 제거.

### 경계 설계 규칙

- **데이터는 `Uint8Array`로 넘긴다.** JSON 직렬화 금지. 프로토콜은 바이너리다 (DD-003)
- Rust → JS 호출은 최소화. 프레임당 수십 번씩 넘나들면 경계 비용이 쌓인다
- JS에 게임 상태를 두지 않는다. **단일 진실은 WASM 안의 `World`뿐**

---

## 3. 에셋 로딩

```
1. index.html 파싱 → WASM 페치 시작 (streaming instantiate)
2. 동시에 아틀라스 PNG 페치 (preload 링크)
3. WASM 준비 → 로비 UI 표시 (게임 에셋 없어도 로비는 뜬다)
4. 아틀라스 도착 → WebGL 텍스처 업로드
5. 오디오는 지연 로딩 (첫 라운드 시작 전까지만 도착하면 됨)
```

**로비를 에셋보다 먼저 띄운다.** 사람들이 기다리는 동안 이름을 입력하고 친구를 기다리게 만들면 체감 로딩이 사라진다.

| 에셋 | 형식 | 예산 |
|---|---|---|
| 아틀라스 | PNG (팔레트 최적화) | ≤ 400 KB |
| 폰트 | WOFF2 서브셋 (한글 상용 2350자 + 영숫자) | ≤ 180 KB |
| 효과음 | OGG Vorbis 64kbps 모노 | ≤ 120 KB 합계 |
| WASM | gzip | ≤ 1.5 MB (H5) |

한글 폰트가 예상외로 크다. 서브셋을 반드시 만든다 (`pyftsubset`).

---

## 4. 렌더 루프

```rust
// crates/app/src/loop.rs 개요
fn frame(&mut self, now_ms: f64) {
    let dt = (now_ms - self.last).min(MAX_FRAME_DT);  // 250ms 상한
    self.last = now_ms;

    self.net.pump();                       // 수신 패킷 처리
    self.acc += dt;

    let mut ticks = 0;
    while self.acc >= TICK_DT && ticks < MAX_CATCHUP {   // 5틱 상한
        self.world.step(self.input.intent());
        self.acc -= TICK_DT;
        ticks += 1;
    }
    if ticks == MAX_CATCHUP { self.request_resync(); }

    self.net.flush();                      // 입력/스냅샷 송신
    self.render.draw(&self.world, self.acc / TICK_DT);
}
```

렌더는 `requestAnimationFrame`, 시뮬은 고정 30Hz. 둘이 분리돼 있다 (DD-003).

---

## 5. UI 상태 기계

```
Boot → NameEntry → Connecting → Lobby ⇄ InGame → RoundResult → Lobby
                       ↓ 실패
                  ConnectionHelp (수동 연결 / TURN 안내)
```

각 상태는 DOM에서 `<section>` 하나. `hidden` 속성으로 토글. 프레임워크 없음.

**React/Vue를 안 쓰는 이유**: 화면이 6개고 상태가 단순하다. 번들 예산(H5)에서 프레임워크 몫을 쓰느니 WASM에 쓴다.

---

## 6. 스타일

- CSS 변수로 테마. 다크 기본, `prefers-color-scheme: light` 대응
- `env(safe-area-inset-*)` 필수 (노치/홈바)
- 애니메이션은 `transform`/`opacity`만 — 레이아웃을 건드리면 모바일에서 끊긴다
- `prefers-reduced-motion` 존중: 화면 흔들림/펄스 끔

---

## 7. 안 하는 것

- 서비스워커 / PWA — v1 범위 밖. 캐싱은 HTTP 헤더로 충분
- SSR — 정적 사이트다
- 번들러 플러그인 지옥 — `vite` 최소 설정만. 설정 파일이 100줄 넘으면 뭔가 잘못된 것
- 프레임워크
- CDN에서 라이브러리 로드 (SECURITY.md §6)
