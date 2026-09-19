# 네트워크 프로토콜 (생성 문서)

> **이 파일은 `tools/gen_protocol_doc.rs`가 `crates/net/src/wire.rs`에서 생성한다. 손으로 고치지 말 것.**
> 현재는 구현 전이라 설계 초안이 들어있다. 구현이 끝나면 생성기가 이 내용을 대체한다.

생성 시각: (미생성 — 초안)
소스: `crates/net/src/wire.rs` (미작성)

---

## 공통 헤더

```
offset  size  field
0       1     msg_type
1       ...   payload
```

바이트 순서: **리틀 엔디언**.
고정소수점: `Q8.8` (i16, 값 = raw / 256.0). 맵 64×40 타일(DD-009) → 좌표 범위 충분.

---

## 0x01 · Snapshot (H→C, unreliable)

호스트가 피어별로 개별 생성. 내용이 피어마다 다르다 (DD-002).

```
1   u8    msg_type = 0x01
4   u32   tick
4   u32   last_processed_input      // 이 피어의 마지막 처리 입력 시퀀스
1   u8    flags                     // bit0: 전체 스냅샷(델타 아님)
1   u8    entity_count
    [entity_count 개]
      1   u8    entity_id
      1   u8    field_mask          // 어떤 필드가 들어있는지
      2   i16   x_q8      (mask bit0)
      2   i16   y_q8      (mask bit1)
      1   u8    angle_q8  (mask bit2)   // 0..255 = 0..2π
      1   u8    state     (mask bit3)   // Idle|Walk|Sneak|Dash|Possessed|Ghost
      1   u8    skin      (mask bit4)
1   u8    despawn_count
    [despawn_count 개]
      1   u8    entity_id
```

델타: `field_mask`가 0인 엔티티는 생략된다. 정지한 플레이어는 0바이트.

예상 크기 (8명, 일반 상황): **60~90 바이트**

---

## 0x02 · Input (C→H, unreliable)

```
1   u8    msg_type = 0x02
4   u32   sequence
1   u8    count                     // 재전송 포함 최근 입력 개수 (1..4)
    [count 개]
      2   i16   move_x_q8
      2   i16   move_y_q8
      1   u8    action_flags        // INTERACT|DASH|EMOTE1..4
      1   u8    look_flags          // bit0: look 존재
      2   i16   look_x_q8  (조건부)
      2   i16   look_y_q8  (조건부)
```

**unreliable 채널이므로 최근 입력 3~4개를 중복 전송한다.** 하나 잃어도 다음 패킷에 들어있다. 10바이트 × 4 = 40바이트, 30Hz에서 1.2 KB/s. 충분히 싸다.

---

## 0x03 · Event (H→C, reliable ordered)

```
1   u8    msg_type = 0x03
4   u32   tick
1   u8    event_kind
    ... (kind별 payload)
```

| kind | 이름 | payload |
|---|---|---|
| 0x01 | PhaseChange | u8 phase, u16 remaining_secs |
| 0x02 | PlayerCaught | u8 victim, u8 seeker |
| 0x03 | PlayerEscaped | u8 player |
| 0x04 | RoundEnd | u8 winner_side, u8 next_seeker |
| 0x05 | NoiseHint | u8 direction_q8, u8 strength |
| 0x06 | RingUpdate | i16 cx_q8, i16 cy_q8, u16 radius_q8, u16 secs |
| 0x07 | Kick | u8 reason |

---

## 0x04 · Chat (any→any, reliable)

```
1   u8    msg_type = 0x04
1   u8    len            // ≤ 200
len bytes  UTF-8
```

수신 측은 **반드시** UTF-8 검증 후 `textContent`로만 렌더한다 (SECURITY.md §6).

---

## 0x05 · HostHandoff (any→any, reliable)

```
1   u8    msg_type = 0x05
N   bytes new_host_peer_id (길이 접두)
4   u32   resume_tick
```

---

## 0x06 · HostShadow (H→후보호스트, reliable)

호스트 이양 대비 저해상도 월드 상태 (RELIABILITY.md §3).

```
1   u8    msg_type = 0x06
4   u32   tick
1   u8    entity_count
    [entity_count 개]
      1   u8    entity_id
      1   u8    x_coarse        // 4타일 단위 양자화 (0..31)
      1   u8    y_coarse
      1   u8    state
```

2초 주기. **정밀도를 일부러 낮췄다** — 후보 호스트에게도 정확한 좌표를 주지 않는다.

---

## 0x07 · Hello / 0x08 · Welcome (핸드셰이크, reliable)

```
Hello   (C→H): u8 type, u8 proto_ver, u8 name_len, name bytes
Welcome (H→C): u8 type, u8 proto_ver, u8 your_entity_id, u8 is_host,
               u8 player_count, [u8 id, u8 name_len, name bytes] × count,
               RuleSet (16 bytes)
```

`proto_ver` 불일치 시 즉시 연결 종료 + "버전이 달라요. 새로고침해주세요" 표시. GitHub Pages 캐시 때문에 실제로 생길 수 있는 상황이다.

---

## 대역폭 추정 (미측정)

8명, 호스트 기준:

| 방향 | 계산 | 값 |
|---|---|---|
| 업로드 (스냅샷) | 75 B × 15 Hz × 7 peer | ≈ 7.9 KB/s |
| 업로드 (이벤트) | 간헐 | ≈ 0.3 KB/s |
| 다운로드 (입력) | 40 B × 30 Hz × 7 peer | ≈ 8.4 KB/s |

클라이언트는 각각 1 KB/s 내외.

**이 숫자는 계산값이지 측정값이 아니다.** exec-plan 003에서 실측.
