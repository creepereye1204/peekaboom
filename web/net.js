// Trystero 래핑 + `fast` 채널(unreliable) 직접 개설.
//
// **주의**: docs/references/trystero-llms.txt(캐시)는 설치된 버전(0.21.8)과
// 어긋나는 부분이 있었다 — 실제 소스(`node_modules/trystero/src/room.js`,
// `strategy.js`, `peer.js`)를 직접 읽고 확인했고, 캐시를 갱신했다
// (tech-debt-tracker.md "해결된 부채" 참조). 실제 API:
//   - room.onPeerJoin(fn) / room.onPeerLeave(fn) — 대입식이 아니라 함수 호출이다
//   - joinRoom(config, roomId, onJoinError) — 3번째 인자는 옵션 객체가 아니라
//     onJoinError 콜백 그 자체다. onPeerHandshake는 이 버전에 없다
//   - 누가 DataChannel을 만들지 알려주는 isInitiator 같은 건 공개 API에 없다.
//     대신 Trystero 자신도 오퍼 충돌을 `selfId > peerId` 비교로 해소한다
//     (strategy.js). 우리도 같은 규칙(사전순 최소가 이긴다 — net::PeerTable과
//     동일한 관례)으로 어느 쪽이 `createDataChannel`을 부를지 정한다:
//     자기 selfId가 상대보다 사전순으로 작으면 내가 만들고, 아니면 듣는다.
//   - peer.js가 자체 'data' 채널을 위해 `pc.ondatachannel =`(프로퍼티 대입)을
//     쓰므로, 우리는 절대 그 프로퍼티를 덮어쓰지 않는다 — `addEventListener`만
//     쓴다. peer.js는 `onnegotiationneeded`를 제네릭하게(초기 연결 이후에도)
//     계속 걸어두므로, 연결 후에 우리가 채널을 새로 만들어도 그 오퍼는 여전히
//     Trystero의 시그널링 릴레이를 통해 상대에게 전달된다.
import { joinRoom, getRelaySockets, selfId } from "trystero";

const APP_ID = "peekaboom-v1";
const BUFFERED_AMOUNT_LIMIT = 64 * 1024; // netcode-model.md: 넘으면 이번 틱 스킵

/**
 * @param {string} code 방 코드 (roomId)
 * @param {string} key 방 비밀번호 (SDP 암호화 — SECURITY.md §5)
 * @param {object} handlers
 * @param {(peerId: string) => void} [handlers.onPeerJoin]
 * @param {(peerId: string) => void} [handlers.onPeerLeave]
 * @param {(peerId: string, x: number, y: number) => void} [handlers.onPosition]
 * @param {(details: object) => void} [handlers.onJoinError]
 */
export function connect(code, key, handlers = {}) {
  const fastChannels = new Map(); // peerId -> RTCDataChannel

  function wireChannel(peerId, ch) {
    ch.binaryType = "arraybuffer";
    ch.onopen = () => fastChannels.set(peerId, ch);
    ch.onclose = () => fastChannels.delete(peerId);
    ch.onerror = () => fastChannels.delete(peerId);
    ch.onmessage = (ev) => {
      // 크기 검증 (SECURITY.md §4): 우리 페이로드는 항상 8바이트(x,y f32)다.
      if (!(ev.data instanceof ArrayBuffer) || ev.data.byteLength !== 8) return;
      const view = new DataView(ev.data);
      handlers.onPosition?.(peerId, view.getFloat32(0, true), view.getFloat32(4, true));
    };
  }

  function setupFastChannel(peerId) {
    const pc = room.getPeers()[peerId];
    if (!pc) return;

    if (selfId < peerId) {
      wireChannel(peerId, pc.createDataChannel("fast", { ordered: false, maxRetransmits: 0 }));
    } else {
      pc.addEventListener("datachannel", (ev) => {
        if (ev.channel.label === "fast") wireChannel(peerId, ev.channel);
      });
    }
  }

  const room = joinRoom({ appId: APP_ID, password: key }, code, (details) => {
    handlers.onJoinError?.(details);
  });

  room.onPeerJoin((peerId) => {
    setupFastChannel(peerId);
    handlers.onPeerJoin?.(peerId);
  });
  room.onPeerLeave((peerId) => {
    fastChannels.delete(peerId);
    handlers.onPeerLeave?.(peerId);
  });

  return {
    room,

    /** 자기 위치를 연결된 모든 fast 채널로 브로드캐스트한다 (unreliable, 001 한정). */
    sendPos(x, y) {
      const buf = new ArrayBuffer(8);
      const view = new DataView(buf);
      view.setFloat32(0, x, true);
      view.setFloat32(4, y, true);
      for (const ch of fastChannels.values()) {
        if (ch.readyState !== "open") continue;
        if (ch.bufferedAmount > BUFFERED_AMOUNT_LIMIT) continue;
        ch.send(buf);
      }
    },

    /** RTT(ms). 실패하면 null. */
    async ping(peerId) {
      try {
        return await room.ping(peerId);
      } catch {
        return null;
      }
    },

    /** 디버그 패널용: {relayUrl: 'connecting'|'open'|'closing'|'closed'} */
    relayStatus() {
      const READY_STATE = ["connecting", "open", "closing", "closed"];
      const status = {};
      for (const [url, ws] of Object.entries(getRelaySockets())) {
        status[url] = READY_STATE[ws.readyState] ?? "?";
      }
      return status;
    },

    fastChannelCount() {
      return fastChannels.size;
    },

    leave() {
      room.leave();
    },
  };
}
