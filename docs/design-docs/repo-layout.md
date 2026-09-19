# DD-006 · 저장소 레이아웃과 빌드

- 상태: **채택됨**
- 관련: H2, H5, H8, ARCHITECTURE §2, §6

---

## 문제

백지에서 시작하는 세션이 "어디에 뭘 만들어야 하는지"를 알아야 한다. ARCHITECTURE.md는 크레이트 *경계*를 말하지만 파일 경로·빌드 명령·CI 단계를 말하지 않는다. 그 간극에서 세션마다 다른 구조가 나오면 하네스가 무의미해진다.

이 문서는 **그대로 만들면 되는 정답**이다. 해석의 여지를 남기지 않는 게 목적이다.

---

## 디렉터리 트리

```
peekaboom/
├── AGENTS.md
├── ARCHITECTURE.md
├── README.md
├── Cargo.toml                  # 워크스페이스 루트. [package]도 겸한다(아래 참조)
├── src/lib.rs                  # 배포 안 함 — 루트 tests/의 그릇일 뿐
├── rust-toolchain.toml
├── .gitignore
├── package.json                # vite + trystero (+ devDep: @playwright/test)
├── vite.config.js
├── playwright.config.js        # e2e/ 실행 설정
│
├── crates/
│   ├── sim/                    # 의존성 0
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # World, Player, step()
│   │       ├── fixed.rs        # Fx (Q16.16), V2
│   │       ├── map.rs          # MapGrid, Tile — M2
│   │       ├── intent.rs       # InputIntent, ActionFlags
│   │       └── vis.rs          # visible_to() — M2
│   ├── net/                    # 001은 의존성 0. wire.rs부터 sim 의존 시작
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # PeerTable, 호스트 선출
│   │       ├── wire.rs         # 바이너리 직렬화 — M2
│   │       ├── host.rs         # 호스트 상태기계 — M2
│   │       └── client.rs       # 예측/재조정 — M2
│   ├── input/                  # sim에만 의존
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # combine()
│   │       ├── keyboard.rs
│   │       └── joystick.rs
│   ├── render/                 # sim + web-sys(wasm32 전용, target-gated)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # 모듈 조립, wasm32 cfg 게이트
│   │       ├── batch.rs        # 인스턴스 배칭 데이터 (네이티브에서도 컴파일)
│   │       ├── gl.rs           # Renderer, WebGL2 실제 구현 — wasm32 전용
│   │       └── atlas.rs        # 텍스처 아틀라스 — M2
│   └── app/                    # 전부 + wasm-bindgen (wasm32 전용, target-gated)
│       ├── Cargo.toml
│       └── src/lib.rs          # #[wasm_bindgen] Game
│
├── web/
│   ├── index.html
│   ├── style.css
│   ├── main.js                 # 조립 + 프레임 루프 호출
│   ├── net.js                  # Trystero 래핑 + fast 채널
│   └── room.js                 # 방 코드 / URL / localStorage
│
├── assets/                     # 원본 (git에 커밋) — M2부터 채워짐
│   └── src/                    # 받아온 CC0 zip을 푼 것
│
├── tools/
│   ├── pack_atlas.rs           # 아틀라스 생성 + 매니페스트 §3 갱신 — M2
│   ├── gen_protocol_doc.rs     # wire.rs → generated/net-protocol.md — M2
│   ├── check_size.sh           # 번들 예산 (H5)
│   └── check_assets.sh         # 코드 참조 ↔ 매니페스트 대조 (H7)
│
├── tests/                      # 워크스페이스 통합 테스트 (루트 [package]의 tests/)
│   ├── determinism.rs
│   ├── no_leak.rs              # H4. CI 필수 — M2 (visible_to 이후)
│   └── netsim.rs               # 지연/손실 주입 — M2
│
├── e2e/                        # Playwright. 브라우저 단 통합 테스트
│   └── connection.spec.js      # 2피어 fast 채널 개설 스모크 테스트
│
├── .github/workflows/
│   ├── ci.yml
│   └── deploy.yml
│
└── docs/                       # 이 하네스
```

### 워크스페이스 루트가 패키지이기도 한 이유

Cargo의 통합 테스트(`tests/*.rs`)는 반드시 어떤 패키지에 속해야 하는데, 여러
크레이트에 걸친 테스트(결정성, H4 유출 방지, 네트워크 시뮬레이션)를 특정
크레이트(`sim`이나 `net`) 소유로 두면 어색하다. 그래서 루트 `Cargo.toml`이
`[workspace]`와 `[package]`를 동시에 선언한다 — Cargo가 지원하는 표준 패턴이다.
이 루트 패키지(`peekaboom-workspace-tests`)는 배포되지 않고, `dev-dependencies`로
필요한 크레이트를 끌어와 `tests/`에서만 쓴다.

### 크레이트 이름

Cargo 패키지명은 `peekaboom-<크레이트>`, 라이브러리명은 `peekaboom_<크레이트>`.
`app`만 `crate-type = ["cdylib", "rlib"]`이다. 나머지는 기본(rlib).

---

## 의존 방향 (강제됨)

```
app ──┬──> render ──┐
      ├──> input  ──┼──> sim
      └──> net    ──┘
```

`sim`은 아무것도 모른다. 역참조가 생기면 설계가 틀린 것이다.

**검증 방법**: `crates/sim/Cargo.toml`의 `[dependencies]`가 비어 있어야 한다. CI에서 확인한다.

### wasm32 격리

| 크레이트 | wasm32 의존 | 네이티브 테스트 |
|---|---|---|
| `sim` | 없음 | ✅ 전부 |
| `net` | 없음 | ✅ 전부 |
| `input` | 없음 | ✅ 전부 |
| `render` | `web-sys` | 타입체크만 |
| `app` | `wasm-bindgen` | 타입체크만 |

**로직의 대부분이 앞 세 개에 있어야 한다.** `render`/`app`에 판단 로직이 쌓이면 테스트 불가능한 코드가 늘어난다는 신호다.

`input`이 브라우저 의존 없이 성립하는 이유: DOM 이벤트에서 뽑은 **값**(키 코드 문자열, 포인터 좌표 f32)만 받는다. 리스너 등록은 `app`의 얇은 껍데기가 한다 (DD-005).

---

## 툴체인 고정

`rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.95"
targets = ["wasm32-unknown-unknown"]
components = ["rustfmt", "clippy"]
```

버전을 고정하는 이유: 결정적 시뮬레이션(CB-6)이 컴파일러 최적화 차이에 영향받지 않게 하려는 것. 고정소수점만 쓰면 이론상 안전하지만, 고정해두면 "왜 이 머신에서만 해시가 다르지"를 조사할 일이 없다.

### 릴리스 프로파일

```toml
[profile.release]
opt-level = "z"      # 크기 우선. H5(1.5MB)가 속도보다 중요하다
lto = true
codegen-units = 1
panic = "abort"      # 패닉 언와인딩 코드가 번들에 안 들어간다
strip = true
```

`opt-level = "z"`가 게임에 이상해 보일 수 있다. 근거: 우리 부하는 **GPU 드로우콜과 네트워크**이지 CPU 연산이 아니다. 8명 × 30Hz 시뮬레이션은 어떤 최적화 수준에서도 여유롭다. 반면 모바일 로딩 3초는 이탈률에 직결된다 (CB-4).

이 판단이 틀렸다면(프로파일링에서 CPU 병목 확인 시) `opt-level = 2`로 바꾸고 크기 예산을 재협상한다.

---

## 빌드

```bash
# 개발 (네이티브 테스트 — 대부분의 작업은 여기서 끝난다)
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

# wasm 빌드
cargo build --release --target wasm32-unknown-unknown -p peekaboom-app
wasm-bindgen target/wasm32-unknown-unknown/release/peekaboom_app.wasm \
    --out-dir pkg --target web
wasm-opt -Oz pkg/peekaboom_app_bg.wasm -o pkg/peekaboom_app_bg.wasm

# 웹 번들
npm ci
npm run build          # vite build → dist/
```

### wasm-pack을 안 쓰는 이유

`wasm-pack`은 npm 배포를 전제한 워크플로를 끼워넣고, 내부적으로 `wasm-bindgen` 버전을 자기가 관리한다. 우리는 npm에 배포하지 않으므로 `cargo build` + `wasm-bindgen` CLI를 직접 부르는 게 단계가 적고 버전 충돌이 없다.

> **알려진 함정**: `wasm-bindgen` CLI 버전과 `Cargo.toml`의 `wasm-bindgen` 크레이트 버전이 **정확히 일치**해야 한다. 어긋나면 런타임에 이상한 에러가 난다. CI에서 `cargo install wasm-bindgen-cli --version $(cargo metadata로 뽑은 버전)`으로 맞춘다.

### vite 설정

```js
// vite.config.js
export default {
  root: 'web',
  base: './',              // GitHub Pages 하위 경로 대응. 절대경로면 404
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    target: 'es2022',
  },
}
```

`base: './'`가 중요하다. `https://user.github.io/peekaboom/`처럼 하위 경로에 올라가므로 절대경로 에셋은 전부 깨진다.

`pkg/`는 `web/`의 형제 디렉터리이므로 `main.js`에서 `../pkg/...`로 참조한다. vite가 번들에 포함시킨다.

---

## CI 단계

`.github/workflows/ci.yml` — PR마다. 두 잡(`rust`, `wasm-and-web`)으로 나뉜다 —
네이티브 검증이 wasm 빌드보다 훨씬 빠르므로 먼저 실패하는 게 낫다.

```
[rust job]
1. checkout
2. rust 툴체인 (rust-toolchain.toml 자동 인식 — `rustup show`)
3. cargo fmt --check
4. cargo clippy --workspace --all-targets -- -D warnings
5. cargo test --workspace            ← sim/net/input/tests/determinism.rs 전부 포함
6. sim 의존성 비어있는지 확인          ← `cargo tree -p peekaboom-sim`으로 검증

[wasm-and-web job, rust job 통과 후]
7. wasm-bindgen-cli 설치 (버전 고정 — Cargo.toml의 wasm-bindgen과 정확히 일치)
8. wasm 빌드
9. tools/check_size.sh               ← H5
10. tools/check_assets.sh            ← H7
11. npm ci && npm run build
12. Playwright E2E (2피어)
```

`tests/no_leak.rs`(H4)·`tests/netsim.rs`는 `visible_to()`/호스트 상태기계가
생기는 M2부터 CI에 추가된다 — 001에는 가릴 시야도, 흉내 낼 호스트도 없다
(tech-debt-tracker TD-012).

`.github/workflows/deploy.yml` — main 푸시 시 `dist/`를 `gh-pages`로
(`peaceiris/actions-gh-pages`).

### 캐시

`~/.cargo`와 `target/`을 캐시한다. `Cargo.lock` 해시를 키로.
`Cargo.lock`은 **커밋한다** (애플리케이션이지 라이브러리가 아니다).

---

## 빌드 해시 노출

`index.html`에 빌드 식별자를 박는다.

```html
<meta name="build" content="__BUILD_SHA__">
```

이유: GitHub Pages는 캐시가 제법 오래 남는다. "안 된다"는 제보의 상당수가 **구버전을 보고 있는 것**이다. 화면 어딘가(디버그 패널)에 빌드 해시가 보이면 5초 만에 판별된다.

프로토콜 버전 불일치 시 표시할 문구도 여기 근거를 둔다 (generated/net-protocol.md의 `proto_ver`).

---

## .gitignore

```
/target
/pkg
/dist
/node_modules
assets/src/*.zip
```

`assets/src/`에 푼 **파일들은 커밋한다** (원본 zip만 제외). 이유: CC0라 재배포 가능하고, 외부 URL이 죽어도 빌드가 된다 (CB-5와 같은 논리).

---

## 새 세션이 백지에서 시작할 때

```
1. AGENTS.md 읽기 (제약)
2. 이 문서 읽기 (구조)
3. docs/exec-plans/active/ 에서 가장 낮은 번호의 미완료 계획
4. 그 계획이 참조하는 product-spec / design-doc 읽기
5. 스캐폴딩 → 구현
```

**디렉터리를 새로 발명하지 않는다.** 위 트리에 자리가 없는 파일이 필요하면, 그건 이 문서를 고칠 신호다.
