//! WebGL2 컨텍스트 관리 + 인스턴스 드로우콜. wasm32 전용.

use crate::batch::{self, Instance};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext as Gl, WebGlBuffer, WebGlProgram, WebGlShader,
};

const VERT_SRC: &str = r#"#version 300 es
layout(location = 0) in vec2 a_local_pos;   // 유닛 사각형 로컬 정점 (-0.5..0.5)
layout(location = 1) in vec2 a_inst_pos;    // 인스턴스 화면 픽셀 좌표
layout(location = 2) in float a_inst_size;  // 인스턴스 픽셀 크기
layout(location = 3) in vec4 a_inst_color;  // 인스턴스 색상

uniform vec2 u_resolution; // 드로잉 버퍼 크기 (device px)

out vec4 v_color;

void main() {
    vec2 pixel_pos = a_inst_pos + a_local_pos * a_inst_size;
    vec2 clip = (pixel_pos / u_resolution) * 2.0 - 1.0;
    // 화면 좌표는 y-down, clip space는 y-up
    gl_Position = vec4(clip.x, -clip.y, 0.0, 1.0);
    v_color = a_inst_color;
}
"#;

const FRAG_SRC: &str = r#"#version 300 es
precision mediump float;
in vec4 v_color;
out vec4 out_color;
void main() {
    out_color = v_color;
}
"#;

/// 최초 인터랙션까지 3초 예산(H5) 안에서 감당 가능한 상한.
pub const MAX_DEVICE_PIXEL_RATIO: f64 = 2.0;

pub struct Renderer {
    gl: Gl,
    program: WebGlProgram,
    instance_vbo: WebGlBuffer,
    u_resolution: web_sys::WebGlUniformLocation,
    device_width: f32,
    device_height: f32,
}

impl Renderer {
    pub fn new(canvas: &HtmlCanvasElement) -> Result<Renderer, JsValue> {
        let gl = canvas
            .get_context("webgl2")?
            .ok_or_else(|| JsValue::from_str("WebGL2 컨텍스트를 만들 수 없습니다"))?
            .dyn_into::<Gl>()?;

        let vert = compile_shader(&gl, Gl::VERTEX_SHADER, VERT_SRC)?;
        let frag = compile_shader(&gl, Gl::FRAGMENT_SHADER, FRAG_SRC)?;
        let program = link_program(&gl, &vert, &frag)?;

        // 유닛 사각형 (두 삼각형, 로컬 좌표 -0.5..0.5)
        #[rustfmt::skip]
        let quad: [f32; 12] = [
            -0.5, -0.5,   0.5, -0.5,   0.5, 0.5,
            -0.5, -0.5,   0.5, 0.5,   -0.5, 0.5,
        ];

        let vao = gl
            .create_vertex_array()
            .ok_or_else(|| JsValue::from_str("VAO 생성 실패"))?;
        gl.bind_vertex_array(Some(&vao));

        let quad_vbo = gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("quad VBO 생성 실패"))?;
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&quad_vbo));
        unsafe {
            let view = js_sys::Float32Array::view(&quad);
            gl.buffer_data_with_array_buffer_view(Gl::ARRAY_BUFFER, &view, Gl::STATIC_DRAW);
        }
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_with_i32(0, 2, Gl::FLOAT, false, 0, 0);

        let instance_vbo = gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("instance VBO 생성 실패"))?;
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&instance_vbo));
        let stride = (batch::FLOATS_PER_INSTANCE * 4) as i32;

        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_with_i32(1, 2, Gl::FLOAT, false, stride, 0);
        gl.vertex_attrib_divisor(1, 1);

        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_with_i32(2, 1, Gl::FLOAT, false, stride, 2 * 4);
        gl.vertex_attrib_divisor(2, 1);

        gl.enable_vertex_attrib_array(3);
        gl.vertex_attrib_pointer_with_i32(3, 4, Gl::FLOAT, false, stride, 3 * 4);
        gl.vertex_attrib_divisor(3, 1);

        gl.bind_vertex_array(Some(&vao));

        let u_resolution = gl
            .get_uniform_location(&program, "u_resolution")
            .ok_or_else(|| JsValue::from_str("u_resolution 유니폼을 찾을 수 없습니다"))?;

        gl.clear_color(0.08, 0.09, 0.11, 1.0);

        Ok(Renderer {
            gl,
            program,
            instance_vbo,
            u_resolution,
            device_width: canvas.width() as f32,
            device_height: canvas.height() as f32,
        })
    }

    /// `css_w`/`css_h`: CSS 픽셀. `dpr`: devicePixelRatio (H5 상한 2.0으로 클램프).
    pub fn resize(&mut self, canvas: &HtmlCanvasElement, css_w: f64, css_h: f64, dpr: f64) {
        let dpr = dpr.min(MAX_DEVICE_PIXEL_RATIO).max(1.0);
        let w = (css_w * dpr).round() as u32;
        let h = (css_h * dpr).round() as u32;
        canvas.set_width(w.max(1));
        canvas.set_height(h.max(1));
        self.gl.viewport(0, 0, w as i32, h as i32);
        self.device_width = w as f32;
        self.device_height = h as f32;
    }

    pub fn render(&self, instances: &[Instance]) {
        let gl = &self.gl;
        gl.clear(Gl::COLOR_BUFFER_BIT);
        if instances.is_empty() {
            return;
        }

        let packed = batch::pack(instances);
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&self.instance_vbo));
        unsafe {
            let view = js_sys::Float32Array::view(&packed);
            gl.buffer_data_with_array_buffer_view(Gl::ARRAY_BUFFER, &view, Gl::DYNAMIC_DRAW);
        }

        gl.use_program(Some(&self.program));
        gl.uniform2f(
            Some(&self.u_resolution),
            self.device_width,
            self.device_height,
        );
        gl.draw_arrays_instanced(Gl::TRIANGLES, 0, 6, instances.len() as i32);
    }
}

fn compile_shader(gl: &Gl, kind: u32, src: &str) -> Result<WebGlShader, JsValue> {
    let shader = gl
        .create_shader(kind)
        .ok_or_else(|| JsValue::from_str("셰이더 생성 실패"))?;
    gl.shader_source(&shader, src);
    gl.compile_shader(&shader);
    if gl
        .get_shader_parameter(&shader, Gl::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        let log = gl.get_shader_info_log(&shader).unwrap_or_default();
        Err(JsValue::from_str(&format!("셰이더 컴파일 실패: {log}")))
    }
}

fn link_program(gl: &Gl, vert: &WebGlShader, frag: &WebGlShader) -> Result<WebGlProgram, JsValue> {
    let program = gl
        .create_program()
        .ok_or_else(|| JsValue::from_str("프로그램 생성 실패"))?;
    gl.attach_shader(&program, vert);
    gl.attach_shader(&program, frag);
    gl.link_program(&program);
    if gl
        .get_program_parameter(&program, Gl::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        let log = gl.get_program_info_log(&program).unwrap_or_default();
        Err(JsValue::from_str(&format!("프로그램 링크 실패: {log}")))
    }
}
