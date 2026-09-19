# DD-003 · 네트코드 모델 (예측 · 재조정 · 보간)

- 상태: **채택됨**
- 관련: CB-2, CB-6, DD-002

---

## 문제

P2P RTT는 40~120ms. 입력을 보내고 결과를 기다리면 조작이 고무줄처럼 느껴진다. 반대로 각자 마음대로 움직이면 상태가 갈라진다.

## 결정

Valve/Overwatch 계열의 고전 3종 세트를 그대로 쓴다. 새로운 걸 발명하지 않는다.

```
1. 클라이언트 예측 (Client-side Prediction)  — 내 캐릭터
2. 서버 재조정 (Server Reconciliation)       — 어긋났을 때
3. 엔티티 보간 (Entity Interpolation)         — 남의 캐릭터
```

### 1. 클라이언트 예측

로컬 입력은 **즉시** 로컬 `sim`에 적용한다. 동시에 시퀀스 번호를 붙여 호스트로 보내고, 미확인 입력을 링버퍼에 보관한다.

```rust
struct PendingInputs {
    buf: [InputIntent; 64],   // 64틱 ≈ 2.1초. 그보다 늦으면 포기
    head: u32,                // 다음 시퀀스
    acked: u32,               // 호스트가 처리했다고 알린 마지막 시퀀스
}
```

### 2. 재조정

호스트 스냅샷에는 `last_processed_input[peer]`가 들어온다. 클라이언트는:

```
1. 로컬 상태를 스냅샷의 권위 상태로 되돌린다
2. acked 이후의 미확인 입력들을 순서대로 다시 step()
3. 결과가 되돌리기 전과 (거의) 같으면 아무 일 없음
```

`sim`이 결정적이라(CB-6) 이 재실행이 정확히 맞아떨어진다. 이게 고정소수점을 쓰는 가장 큰 이유다.

**보정 스무딩**: 재조정 후 위치 차이가 0.25타일 미만이면 즉시 스냅하지 않고 3프레임에 걸쳐 보간한다. 미세한 튐을 없앤다. 0.25타일 이상이면 즉시 스냅한다 — 큰 차이를 부드럽게 만들면 오히려 더 이상하다.

### 3. 엔티티 보간

원격 플레이어는 **100ms 과거**를 렌더링한다. 스냅샷 2개 사이를 선형 보간한다.

```
render_time = now - INTERP_DELAY(100ms)
두 스냅샷 s0.t <= render_time <= s1.t 를 찾아 lerp
```

스냅샷 15Hz(66ms 간격)이므로 100ms 버퍼면 한 패킷을 잃어도 보간이 끊기지 않는다.

패킷이 계속 안 오면(200ms 초과) 보간 대신 **마지막 속도로 최대 300ms까지 외삽**하고, 그 뒤엔 멈춘 채로 둔다. 무한 외삽은 캐릭터가 벽을 뚫고 날아가게 만든다.

## 틱레이트

| 항목 | 값 | 근거 |
|---|---|---|
| 시뮬레이션 틱 | 30 Hz | 모바일 배터리 + 숨바꼭질은 프레임 정밀 조작이 필요없다 |
| 입력 전송 | 30 Hz (틱당 1회) | |
| 스냅샷 전송 | 15 Hz | 대역폭 절반. 보간이 메꾼다 |
| 렌더 | requestAnimationFrame (보통 60) | 시뮬과 분리. 보간으로 부드럽게 |

`app`의 누산기 루프:

```rust
accumulator += dt;
while accumulator >= TICK_DT {
    sim.step(input);
    accumulator -= TICK_DT;
}
render(alpha = accumulator / TICK_DT);
```

탭이 백그라운드로 갔다 오면 `dt`가 거대해진다. **최대 5틱까지만 따라잡고 나머지는 버린다** (death spiral 방지). 5틱 이상 밀렸으면 호스트 스냅샷으로 강제 동기화한다.

## 패킷 포맷

바이너리. JSON 쓰지 않는다.

```
Snapshot (unreliable):
  u8  type = 0x01
  u32 tick
  u32 last_processed_input
  u8  entity_count
  [ entity: u8 id | u8 flags | i16 x_q8 | i16 y_q8 | u8 angle_q8 | (조건부) u8 state ]
```

좌표는 Q8.8 고정소수점 16비트. 맵이 64×40 타일(DD-009)이므로 충분하다. 8명 기준 스냅샷 1개 ≈ 60~90바이트. 15Hz × 7피어 ≈ **9 KB/s 업로드** (호스트 기준). 모바일 셀룰러에서도 부담 없다.

델타 압축: 직전 ack된 스냅샷 대비 변한 필드만 비트마스크로. 정지한 플레이어는 0바이트.

## DataChannel 설정

```js
// 스냅샷 / 입력 — 늦은 패킷은 쓸모없다
pc.createDataChannel('fast', { ordered: false, maxRetransmits: 0 })

// 이벤트 / 채팅 / 호스트 이양 — 반드시 도착해야 한다
pc.createDataChannel('sure', { ordered: true })
```

Trystero의 `makeAction`은 내부적으로 신뢰성 채널을 쓴다. 따라서 **스냅샷은 Trystero 액션을 쓰지 않고** `room.getPeers()`로 `RTCPeerConnection`을 꺼내 우리가 직접 `fast` 채널을 만든다. Trystero는 매치메이킹과 이벤트에만 쓴다.

> 이건 Trystero를 "용도에 안 맞게" 쓰는 게 아니라, 매치메이킹 레이어와 전송 레이어를 분리하는 정상적인 사용이다. `getPeers()`가 공개 API로 `RTCPeerConnection`을 돌려주는 이유가 이것이다.

## 미결 사항

- [ ] 실측 RTT 분포 (한국 통신 3사 셀룰러 ↔ 가정용 와이파이). exec-plan 003
- [ ] 지터 버퍼를 고정 100ms로 둘지, RTT에 적응시킬지. 일단 고정으로 시작
- [ ] 8명 풀메시에서 호스트 업로드 실측. 이론값 9KB/s가 맞는지
