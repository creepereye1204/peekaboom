# 에셋 매니페스트

> **이 파일은 부분적으로 수동 관리된다.** 후보 조사 결과는 손으로 기록하고, 실제 빌드에 들어간 파일 목록(§3)은 `tools/pack_atlas.rs`가 생성한다. §3은 손으로 고치지 않는다.

마지막 조사: 2026-09-19

---

## 1. 라이선스 정책 (H7)

- **CC0 1.0 Universal** 또는 그에 준하는 것만 사용
- CC-BY는 **쓰지 않는다** — 귀속 표기 관리 비용 대비 이득이 없고, CC0로 충분히 대체 가능
- 폰트는 OFL 허용 (재배포 가능, 임베딩 가능)
- 에셋 추가 시 이 파일에 **출처 URL + 라이선스 + 사용 파일**을 반드시 기록

---

## 2. 채택 후보 (조사 완료, 다운로드 검증 대기)

### 2-1. 캐릭터 · 탑다운 스프라이트

| 항목 | 내용 |
|---|---|
| 팩 | Kenney — **Top-down Shooter** |
| URL | https://kenney.nl/assets/top-down-shooter |
| 미러 | https://opengameart.org/content/topdown-shooter |
| 라이선스 | CC0 1.0 Universal |
| 규모 | 580 에셋 |
| 포함 | 캐릭터, 타일, 오브젝트. 개별 PNG + 타일시트 + 스프라이트시트 + 벡터 원본 |
| 용도 | 플레이어 캐릭터, 기본 탑다운 시점 |
| ⚠ 주의 | **미리보기에서 흐리게 표시된 캐릭터 일부는 "Kenney Game Assets 2" 전용이며 무료 팩에 없다.** zip을 받아 실제 파일 존재를 확인한 뒤 매니페스트에 넣을 것 |

### 2-2. 실내 타일 · 오브젝트

| 항목 | 내용 |
|---|---|
| 팩 | Kenney — **Scribble Dungeons** |
| URL | https://kenney-assets.itch.io/scribble-dungeons |
| 라이선스 | CC0 1.0 Universal |
| 규모 | 256+ 스프라이트, zip 1.4 MB |
| 포함 | 탑다운 실내/던전 타일, 캐릭터, 무기, 아이템, 벡터 원본 |
| 용도 | 맵 타일, 빙의 가능 오브젝트 |

### 2-3. 대안 타일셋 (1비트 스타일)

| 항목 | 내용 |
|---|---|
| 팩 | Kenney — **1-Bit Pack** |
| URL | https://kenney-assets.itch.io/1-bit-pack |
| 라이선스 | CC0 1.0 Universal |
| 규모 | 1,000+ 타일 |
| 용도 | **대안 아트 디렉션.** 1비트는 어둠/시야 표현에 극도로 잘 맞고 아틀라스가 아주 작아진다(H5에 유리). DESIGN.md의 파스텔 방향과 배타적이므로 **둘 중 하나를 골라야 한다** |
| 결정 | 미정 — exec-plan 002의 아트 스파이크에서 둘 다 프로토타입 후 결정 |

### 2-4. 모바일 온스크린 컨트롤

| 항목 | 내용 |
|---|---|
| 팩 | Kenney — **Onscreen Controls** |
| URL | https://kenney.nl/assets/onscreen-controls |
| 라이선스 | CC0 1.0 Universal |
| 규모 | 400 에셋, zip 761 KB |
| 포함 | 8가지 스타일의 조이스틱/버튼/D-pad. 섞어 쓸 수 있음 |
| 용도 | 가상 조이스틱, 액션 버튼 (DD-005, mobile-controls.md) |
| 선택 스타일 | `flat_dark` (어두운 게임 화면 위 대비) — 다운로드 후 실제 폴더명 확인 필요 |

### 2-5. UI (메뉴 · 패널 · 버튼)

| 항목 | 내용 |
|---|---|
| 팩 | Kenney — **UI Pack** |
| URL | https://opengameart.org/content/ui-pack |
| 라이선스 | CC0 1.0 Universal |
| 포함 | 버튼, 슬라이더, 패널, 체크/크로스 아이콘 |
| 용도 | 로비, 설정, 결과 화면 |

### 2-6. 효과음

| 항목 | 내용 |
|---|---|
| 팩 | Kenney — **Interface Sounds** |
| URL | https://kenney.nl/assets/interface-sounds · https://opengameart.org/content/interface-sounds |
| 라이선스 | CC0 1.0 Universal |
| 규모 | 100개 OGG, zip 834 KB, 볼륨 정규화됨 |
| 용도 | UI 클릭, 확인, 토글 |
| ⚠ 부족 | **발소리·대시·잡힘 같은 게임플레이 사운드는 이 팩에 없다.** 별도 조사 필요 (§4 미해결) |

### 2-7. 폰트

| 항목 | 내용 |
|---|---|
| 1순위 | Pretendard (OFL 1.1) — https://github.com/orioncactus/pretendard |
| 2순위 | Noto Sans KR (OFL 1.1) |
| 처리 | `pyftsubset`으로 한글 상용 2350자 + 영숫자 + 기호 서브셋 → WOFF2 |
| 예산 | ≤ 180 KB |

---

## 3. 빌드에 실제로 포함된 파일

```
(아직 없음 — tools/pack_atlas.rs 미구현)
```

이 섹션은 아틀라스 패커가 자동 생성한다. 빌드 시 다음 형식으로 채워진다:

```
atlas.<hash>.png   2048×2048   NEAREST
  ├ player/idle_0      src: kenney_top-down-shooter/PNG/Man Blue/manBlue_stand.png
  ├ tile/floor_wood_0  src: kenney_scribbledungeons/PNG/tile_0042.png
  └ ...
```

---

## 4. 미해결

| # | 항목 | 상태 |
|---|---|---|
| A1 | 게임플레이 SFX (발소리, 대시, 잡힘, 유령 쿵) | **미확보.** Kenney Interface Sounds에 없음. freesound.org의 CC0 필터 또는 Kenney *Impact Sounds* / *RPG Audio* 조사 필요 |
| A2 | BGM (로비용 1곡) | 미확보. Kenney *Music Jingles* 후보 |
| A3 | 아트 디렉션 확정 (파스텔 vs 1비트) | exec-plan 002 스파이크에서 결정 |
| A4 | 실제 zip 다운로드 + 파일 존재 검증 | **전 항목 미수행.** 위 정보는 공개 페이지 기준이며, 실제 파일 목록은 받아봐야 안다 |
| A5 | 한글 폰트 서브셋 실측 크기 | 미측정 |

> A4가 가장 중요하다. 이 문서의 팩 이름·URL·라이선스는 실제 배포 페이지에서 확인했지만, **개별 파일 경로는 아직 하나도 검증되지 않았다.** 코드에서 파일을 참조하기 전에 반드시 zip을 받아 확인할 것 (AGENTS.md §6).
