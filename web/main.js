// 조립 + 프레임 루프. 001(연결 수직 슬라이스) 범위:
// 호스트 권위 없음, 예측/보간 없음, 카메라 없음 (exec-plans/active/001-connection-slice.md).
import init, { Game } from "../pkg/peekaboom_app.js";
import { selfId } from "trystero";
import { connect } from "./net.js";
import { getOrCreateRoom, shareRoom } from "./room.js";

const TICK_MS = 1000 / 30; // netcode-model.md: 시뮬레이션 틱 30Hz
const MAX_CATCHUP_TICKS = 5; // death spiral 방지: 5틱 이상 밀리면 나머지는 버린다
const LOCAL_SLOT = 0;

async function main() {
  await init();

  const canvas = document.getElementById("game-canvas");
  const debugEl = document.getElementById("debug-panel");
  const shareBtn = document.getElementById("share-btn");

  const game = new Game(canvas, LOCAL_SLOT);

  function resize() {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    game.resize(canvas, window.innerWidth, window.innerHeight, dpr);
  }
  window.addEventListener("resize", resize);
  window.addEventListener("orientationchange", resize);
  resize();

  // --- 입력: 키보드 (DD-005) ---
  window.addEventListener("keydown", (e) => {
    if (e.repeat) return;
    game.on_key_down(e.code);
  });
  window.addEventListener("keyup", (e) => game.on_key_up(e.code));

  // --- 입력: 터치 조이스틱, 화면 왼쪽 절반, floating stick (DD-005) ---
  let joystickPointerId = null;
  canvas.addEventListener("pointerdown", (e) => {
    if (joystickPointerId !== null) return;
    if (e.clientX > window.innerWidth / 2) return;
    joystickPointerId = e.pointerId;
    canvas.setPointerCapture(e.pointerId);
    game.joystick_start(e.clientX, e.clientY);
  });
  canvas.addEventListener("pointermove", (e) => {
    if (e.pointerId !== joystickPointerId) return;
    game.joystick_move(e.clientX, e.clientY);
  });
  const endJoystick = (e) => {
    if (e.pointerId !== joystickPointerId) return;
    joystickPointerId = null;
    game.joystick_end();
  };
  canvas.addEventListener("pointerup", endJoystick);
  canvas.addEventListener("pointercancel", endJoystick);

  // --- 방 / 네트워크 ---
  const { code, key } = getOrCreateRoom();

  // 슬롯 배정은 각 브라우저가 로컬로만 정한다 (호스트 권위가 아직 없다 — M1
  // 검증 대상은 "연결이 되는가"이지 "다같이 일관된 상태를 보는가"가 아니다).
  const remoteSlots = new Map();
  let nextSlot = 1;
  const slotFor = (peerId) => {
    if (!remoteSlots.has(peerId)) remoteSlots.set(peerId, nextSlot++);
    return remoteSlots.get(peerId);
  };

  let lastJoinError = null;
  const net = connect(code, key, {
    onPeerJoin(peerId) {
      game.add_peer(slotFor(peerId));
    },
    onPeerLeave(peerId) {
      if (remoteSlots.has(peerId)) game.remove_peer(remoteSlots.get(peerId));
    },
    onPosition(peerId, x, y) {
      game.on_remote_pos(slotFor(peerId), x, y);
    },
    onJoinError(details) {
      lastJoinError = details;
      console.error("[peekaboom] join error", details);
    },
  });

  shareBtn.addEventListener("click", async () => {
    const result = await shareRoom();
    const label = { shared: "공유됨", copied: "링크 복사됨", cancelled: "공유", failed: "복사 실패" };
    shareBtn.textContent = label[result] ?? "공유";
    setTimeout(() => (shareBtn.textContent = "공유"), 1500);
  });

  // --- 고정 틱 누산기 + 렌더 (DD-003) ---
  let acc = 0;
  let last = performance.now();
  function frame(now) {
    acc += now - last;
    last = now;
    let ticks = 0;
    while (acc >= TICK_MS && ticks < MAX_CATCHUP_TICKS) {
      game.tick();
      net.sendPos(game.local_x(), game.local_y());
      acc -= TICK_MS;
      ticks++;
    }
    if (ticks === MAX_CATCHUP_TICKS) acc = 0;
    game.render();
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);

  // --- 디버그 패널: RTT + 릴레이 상태 + 빌드 해시 (001 완료 조건 4) ---
  const buildHash = document.querySelector('meta[name="build"]')?.content ?? "dev";
  setInterval(async () => {
    const peers = Object.keys(net.room.getPeers());
    const rtts = await Promise.all(peers.map((p) => net.ping(p)));
    const relayLine = Object.entries(net.relayStatus())
      .map(([url, st]) => {
        try {
          return `${new URL(url).host}:${st}`;
        } catch {
          return `${url}:${st}`;
        }
      })
      .join(" ");
    const rttLine = peers.map((p, i) => `${p.slice(0, 4)}:${rtts[i] ?? "?"}ms`).join(" ");

    debugEl.textContent =
      `build ${buildHash} | room ${code} | self ${selfId.slice(0, 6)}\n` +
      `peers ${peers.length} (fast ${net.fastChannelCount()}) | RTT ${rttLine}\n` +
      `relay: ${relayLine}` +
      (lastJoinError ? `\n⚠ join error: ${JSON.stringify(lastJoinError)}` : "");
  }, 2000);
}

main().catch((err) => {
  console.error("[peekaboom] 초기화 실패", err);
  document.body.textContent = `초기화 실패: ${err}`;
});
