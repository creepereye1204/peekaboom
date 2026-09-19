//! `peekaboom-app` — wasm-bindgen 진입점. 위 크레이트들을 조립한다.
//! 프레임 루프의 실제 시계는 `web/main.js`가 쥔다 (sim은 시간 개념이 없다).
//!
//! 001(연결 수직 슬라이스) 범위: 호스트 권위 없음. `tick()`은 로컬 플레이어만
//! 진행시키고, 원격 플레이어 좌표는 `on_remote_pos`로 수신한 값을 그대로 반영한다.
#![cfg(target_arch = "wasm32")]

use std::collections::HashSet;

use peekaboom_render::batch::Instance;
use peekaboom_render::Renderer;
use peekaboom_sim::{InputIntent, PlayerId, World, V2};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

const PLAYER_SIZE_CSS_PX: f32 = 32.0;
const LOCAL_COLOR: [f32; 4] = [0.30, 0.65, 1.00, 1.0];
const REMOTE_COLOR: [f32; 4] = [1.00, 0.55, 0.20, 1.0];
const JOYSTICK_BASE_CSS_PX: f32 = 128.0; // DD-005: 최대 반경 64px 지름의 두 배
const JOYSTICK_KNOB_CSS_PX: f32 = 56.0;
const JOYSTICK_BASE_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 0.15];
const JOYSTICK_KNOB_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 0.35];

#[wasm_bindgen]
pub struct Game {
    world: World,
    renderer: Renderer,
    local_id: PlayerId,
    pressed_keys: HashSet<String>,
    joystick_origin: Option<(f32, f32)>,
    joystick_current: (f32, f32),
    dpr: f64,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement, local_id: u8) -> Result<Game, JsValue> {
        let renderer = Renderer::new(&canvas)?;
        let mut world = World::new();
        // 001: 카메라가 없으니 화면 왼쪽 위 근처에 스폰한다. 정확한 위치는
        // 중요하지 않다 — 검증 대상은 "두 사각형이 움직이는 게 보이는가"뿐.
        world.spawn(local_id, V2::from_f32(120.0, 120.0));

        Ok(Game {
            world,
            renderer,
            local_id,
            pressed_keys: HashSet::new(),
            joystick_origin: None,
            joystick_current: (0.0, 0.0),
            dpr: 1.0,
        })
    }

    pub fn resize(&mut self, canvas: HtmlCanvasElement, css_w: f64, css_h: f64, dpr: f64) {
        self.dpr = dpr.max(1.0).min(peekaboom_render::MAX_DEVICE_PIXEL_RATIO);
        self.renderer.resize(&canvas, css_w, css_h, dpr);
    }

    pub fn on_key_down(&mut self, code: String) {
        self.pressed_keys.insert(code);
    }

    pub fn on_key_up(&mut self, code: String) {
        self.pressed_keys.remove(&code);
    }

    /// 화면 왼쪽 절반 터치 시작 (floating stick 중심 고정). 좌표는 CSS px.
    pub fn joystick_start(&mut self, x: f32, y: f32) {
        self.joystick_origin = Some((x, y));
        self.joystick_current = (x, y);
    }

    pub fn joystick_move(&mut self, x: f32, y: f32) {
        if self.joystick_origin.is_some() {
            self.joystick_current = (x, y);
        }
    }

    pub fn joystick_end(&mut self) {
        self.joystick_origin = None;
    }

    pub fn add_peer(&mut self, id: u8) {
        self.world.spawn(id, V2::from_f32(320.0, 120.0));
    }

    pub fn remove_peer(&mut self, id: u8) {
        self.world.despawn(id);
    }

    /// 원격 피어에게서 수신한 좌표를 그대로 반영한다 (보간 없음 — 001 범위).
    pub fn on_remote_pos(&mut self, id: u8, x: f32, y: f32) {
        if self.world.player(id).is_none() {
            self.world.spawn(id, V2::from_f32(x, y));
        } else {
            self.world.set_remote_pos(id, V2::from_f32(x, y));
        }
    }

    /// 로컬 입력을 반영해 한 틱 진행한다. 30Hz로 `main.js`의 고정 틱
    /// 누산기가 호출한다.
    pub fn tick(&mut self) {
        let pressed: Vec<&str> = self.pressed_keys.iter().map(|s| s.as_str()).collect();
        let kb_move = peekaboom_input::keyboard::move_dir(&pressed);
        let kb_action = peekaboom_input::keyboard::action_flags_from_keys(&pressed);
        let js_dir = self
            .joystick_origin
            .map(|origin| peekaboom_input::joystick::direction(origin, self.joystick_current));

        let intent: InputIntent = peekaboom_input::combine(kb_move, kb_action, js_dir);
        self.world.step(self.local_id, &intent);
        self.world.advance_tick();
    }

    pub fn render(&self) {
        let dpr = self.dpr as f32;
        let mut instances: Vec<Instance> = self
            .world
            .iter_players()
            .map(|(id, p)| Instance {
                x: p.pos.x.to_f32() * dpr,
                y: p.pos.y.to_f32() * dpr,
                size: PLAYER_SIZE_CSS_PX * dpr,
                color: if id == self.local_id {
                    LOCAL_COLOR
                } else {
                    REMOTE_COLOR
                },
            })
            .collect();

        // DD-005 "원 두 개로" — 진짜 원은 아니지만(사각형 배칭만 있음), 001은
        // 의도적으로 못생기게 만든다. 손잡이 위치를 눈으로 볼 수 있으면 충분하다.
        if let Some(origin) = self.joystick_origin {
            let (dx, dy) = peekaboom_input::joystick::knob_offset(origin, self.joystick_current);
            instances.push(Instance {
                x: origin.0 * dpr,
                y: origin.1 * dpr,
                size: JOYSTICK_BASE_CSS_PX * dpr,
                color: JOYSTICK_BASE_COLOR,
            });
            instances.push(Instance {
                x: (origin.0 + dx) * dpr,
                y: (origin.1 + dy) * dpr,
                size: JOYSTICK_KNOB_CSS_PX * dpr,
                color: JOYSTICK_KNOB_COLOR,
            });
        }

        self.renderer.render(&instances);
    }

    /// 로컬 위치를 네트워크로 보내기 위해 JS에 넘긴다 (CSS px 좌표계).
    pub fn local_x(&self) -> f32 {
        self.world
            .player(self.local_id)
            .map(|p| p.pos.x.to_f32())
            .unwrap_or(0.0)
    }

    pub fn local_y(&self) -> f32 {
        self.world
            .player(self.local_id)
            .map(|p| p.pos.y.to_f32())
            .unwrap_or(0.0)
    }
}
