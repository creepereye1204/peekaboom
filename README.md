# PEEKABOOM

**브라우저에서 하는 서버 없는 멀티플레이 숨바꼭질.**

링크 하나 보내면 6명이 30초 안에 들어와서 같이 한다. 계정도, 설치도, 서버도 없다.

```
https://<user>.github.io/peekaboom/#r=A7K3QX&k=...
```

---

## 뭔가

| | |
|---|---|
| **모드** | 숨바꼭질 (술래가 찾는다) / 럼블 (링이 좁아진다) |
| **인원** | 3~8명 |
| **플랫폼** | 데스크톱 + 모바일 브라우저 (터치 조작 1급 지원) |
| **네트워크** | WebRTC P2P. 시그널링은 공개 릴레이, 게임 데이터는 피어 직통 |
| **호스팅** | GitHub Pages 정적 파일. **런타임 서버 비용 0원** |
| **구현** | Rust → WASM, WebGL2 직접 제어 |

---

## 상태

**개발 중. 아직 플레이할 수 없다.**

현재 단계는 `docs/exec-plans/index.md`에서 확인.

---

## 알아둘 것

### 방을 만든 사람이 심판입니다

서버가 없기 때문에, 방을 만든 사람의 브라우저가 게임 판정을 맡습니다. 그 사람이 프로그램을 고치면 다른 사람의 위치를 볼 수 있습니다.

**모르는 사람이 만든 방에서는 공정성이 보장되지 않습니다.** 아는 사람끼리 하세요.

(랭킹이나 보상이 없어서 치트해서 얻을 게 없긴 합니다. 자세한 내용: [docs/SECURITY.md](docs/SECURITY.md) §3)

### 서로의 IP를 알게 됩니다

WebRTC P2P의 본질적 성질입니다. 우리가 수집하는 게 아니라, 브라우저끼리 직접 연결하려면 주소를 알아야 하기 때문입니다.

### 연결이 안 되는 네트워크가 있습니다

일부 기업망·학교망·통신사 환경(대칭 NAT)에서는 브라우저 간 직접 연결이 안 됩니다. 이때는 중계 서버(TURN)가 필요한데, **우리는 서버를 운영하지 않습니다.**

- 와이파이 대신 셀룰러로(또는 반대로) 바꾸면 되는 경우가 많습니다
- 설정에서 직접 TURN 서버를 넣을 수 있습니다 (무료 서비스 있음)

### 아무것도 저장하지 않습니다

- 계정 없음, 쿠키 없음, 분석 도구 없음, 서버 로그 없음 (서버가 없음)
- 닉네임·설정만 브라우저 `localStorage`에. 어디로도 전송되지 않습니다
- 그래서 전적이나 랭킹도 없습니다. 저장할 곳이 없습니다

---

## 개발

이 저장소는 **문서 주도(하네스 엔지니어링)**로 굴러갑니다. 문서가 사양이고 코드는 그 구현입니다.

### 읽는 순서

```
1. AGENTS.md                      ← 제약. 가장 먼저
2. ARCHITECTURE.md                ← 시스템이 어떤 조각으로 나뉘나
3. docs/design-docs/repo-layout.md ← 어디에 뭘 만드나
4. docs/exec-plans/index.md       ← 지금 어느 단계인가
```

### 문서 지도

| 경로 | 내용 |
|---|---|
| `AGENTS.md` | 에이전트가 지킬 제약. 하드 제약 8개 |
| `ARCHITECTURE.md` | 크레이트 분해, 호스트 권위, 시그널링 경로 |
| `docs/design-docs/` | **왜** 그렇게 정했나 (ADR). 폐기된 결정도 남긴다 |
| `docs/product-specs/` | **무엇을** 만드나. 사용자가 보고 겪는 것 |
| `docs/exec-plans/` | **지금** 뭘 하나. 진행 상태와 완료 조건 |
| `docs/references/` | 외부 라이브러리 요약 캐시 |
| `docs/generated/` | 스크립트가 만드는 문서. 손으로 안 고친다 |
| `docs/PLANS.md` | 전체 로드맵 (M0~M5) |
| `docs/PRODUCT_SENSE.md` | 제품 판단. 기술 판단보다 위 |
| `docs/SECURITY.md` | 위협 모델. 서버가 없으면 위협도 다르다 |
| `docs/RELIABILITY.md` | 남의 인프라가 죽었을 때도 돌아가기 |
| `docs/QUALITY_SCORE.md` | 머지 기준 |

### 핵심 설계 두 가지

**1. 숨은 사람의 좌표는 아예 보내지 않는다** ([DD-002](docs/design-docs/authority-and-visibility.md))

P2P 브로드캐스트는 모두에게 모든 걸 보낸다. F12만 열면 숨은 사람 위치가 보인다. 그래서 호스트가 **피어별로 다른 스냅샷**을 만든다. 술래에겐 자기 시야 안의 것만.

이건 최적화가 아니라 **게임 규칙**이다. CI 필수 테스트로 강제한다.

**2. 결정적 시뮬레이션** ([CB-6](docs/design-docs/core-beliefs.md))

부동소수점 대신 Q16.16 고정소수점. 이 위에 호스트 이양·클라이언트 재조정·리플레이가 올라간다.

### 빌드

```bash
cargo test --workspace                                   # 로직 (네이티브, 빠름)
cargo clippy --workspace --all-targets -- -D warnings
npm run build                                            # → dist/
```

자세한 건 [docs/design-docs/repo-layout.md](docs/design-docs/repo-layout.md).

---

## 라이선스

코드: MIT

에셋: CC0 1.0 (출처는 [docs/generated/asset-manifest.md](docs/generated/asset-manifest.md))
폰트: OFL 1.1
