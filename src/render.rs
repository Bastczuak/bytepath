mod gl {
  include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use crate::{color::ColorGl, render::gl::types::*, RGB_CLEAR_COLOR, SCREEN_HEIGHT, SCREEN_WIDTH};
use std::ffi::CString;
use crate::environment::{SCREEN_RENDER_HEIGHT, SCREEN_RENDER_WIDTH};

const FBO_VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec2 Position;
layout (location = 1) in vec2 TexCoords;
out VERTEX_SHADER_OUTPUT {
  vec2 TexCoords;
} OUT;
void main() {
  OUT.TexCoords = TexCoords;
  gl_Position = vec4(Position, 0.0, 1.0);
}
"#;

const FBO_FRAGMENT_SHADER: &str = r#"
#version 330 core
in VERTEX_SHADER_OUTPUT {
  vec2 TexCoords;
} IN;
out vec4 Color;
uniform sampler2D uTexture;
void main() {
  Color = texture(uTexture, IN.TexCoords);
}
"#;

#[rustfmt::skip]
const LOW_RES_QUAD_VERTICES: [f32; 24] = [
  -1.0, 1.0, 0.0,
  1.0, -1.0, -1.0,
  0.0, 0.0, 1.0,
  -1.0, 1.0, 0.0,
  -1.0, 1.0, 0.0,
  1.0, 1.0, -1.0,
  1.0, 0.0, 1.0,
  1.0, 1.0, 1.0,
];

pub struct Gl {
  inner: std::rc::Rc<gl::Gl>,
}

impl Gl {
  pub fn load_with<F>(load_fn: F) -> Self
    where
      F: FnMut(&'static str) -> *const GLvoid,
  {
    Self {
      inner: std::rc::Rc::new(gl::Gl::load_with(load_fn)),
    }
  }
}

impl std::ops::Deref for Gl {
  type Target = gl::Gl;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

pub struct LowResFrameBuffer {
  vao: GLuint,
  vbo: GLuint,
  fbo: GLuint,
  texture2d: GLuint,
  shader_program: GLuint,
}

pub struct OpenglCtx {
  clear_color: ColorGl,
  frame_buffer: LowResFrameBuffer,
  pub viewport: (GLsizei, GLsizei),
}

unsafe fn create_error_buffer(length: usize) -> CString {
  let mut buffer = Vec::with_capacity(length + 1);
  buffer.extend([b' '].iter().cycle().take(length));
  CString::from_vec_unchecked(buffer)
}

fn compile_shader(gl: &gl::Gl, src: &str, kind: GLenum) -> Result<GLuint, String> {
  unsafe {
    let shader = gl.CreateShader(kind);
    let c_str_src = CString::new(src.as_bytes()).unwrap();
    gl.ShaderSource(shader, 1, &c_str_src.as_ptr(), std::ptr::null());
    gl.CompileShader(shader);
    let mut success = gl::TRUE as GLint;
    gl.GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

    if success == (gl::FALSE as GLint) {
      let mut len = 0;
      gl.GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
      let error = create_error_buffer(len as usize);
      gl.GetShaderInfoLog(shader, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
      return Err(error.to_string_lossy().into_owned());
    }
    Ok(shader)
  }
}

fn link_program(gl: &gl::Gl, vertex_shader: GLuint, fragment_shader: GLuint) -> Result<GLuint, String> {
  unsafe {
    let program = gl.CreateProgram();
    gl.AttachShader(program, vertex_shader);
    gl.AttachShader(program, fragment_shader);
    gl.LinkProgram(program);
    let mut success = gl::TRUE as GLint;
    gl.GetProgramiv(program, gl::LINK_STATUS, &mut success);

    if success == (gl::FALSE as GLint) {
      let mut len = 0;
      gl.GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
      let error = create_error_buffer(len as usize);
      gl.GetProgramInfoLog(program, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
      return Err(error.to_string_lossy().into_owned());
    }

    gl.DeleteShader(vertex_shader);
    gl.DeleteShader(fragment_shader);

    Ok(program)
  }
}

pub fn create_shader_program(gl: &gl::Gl, vertex_src: &str, fragment_src: &str) -> Result<GLuint, String> {
  let vertex_shader = compile_shader(gl, vertex_src, gl::VERTEX_SHADER)?;
  let fragment_shader = compile_shader(gl, fragment_src, gl::FRAGMENT_SHADER)?;
  link_program(gl, vertex_shader, fragment_shader)
}

pub fn init(gl: &Gl) -> Result<OpenglCtx, String> {
  let low_res_prg = create_shader_program(gl, FBO_VERTEX_SHADER, FBO_FRAGMENT_SHADER)?;
  let (fbo_vao, fbo_vbo, fbo, fbo_texture) = unsafe {
    let (mut vao, mut vbo, mut fbo) = (0, 0, 0);
    gl.GenVertexArrays(1, &mut vao);
    gl.GenBuffers(1, &mut vbo);
    gl.BindVertexArray(vao);
    gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl.BufferData(
      gl::ARRAY_BUFFER,
      (LOW_RES_QUAD_VERTICES.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
      LOW_RES_QUAD_VERTICES.as_ptr() as *const GLvoid,
      gl::STATIC_DRAW,
    );

    let pos_attr = gl.GetAttribLocation(low_res_prg, CString::new("Position").unwrap().into_raw());
    gl.EnableVertexAttribArray(pos_attr as u32);
    gl.VertexAttribPointer(
      pos_attr as u32,
      2,
      gl::FLOAT,
      gl::FALSE,
      (4 * std::mem::size_of::<f32>()) as i32,
      std::ptr::null(),
    );

    let texture_coords_attr = gl.GetAttribLocation(low_res_prg, CString::new("TexCoords").unwrap().into_raw());
    gl.EnableVertexAttribArray(texture_coords_attr as u32);
    gl.VertexAttribPointer(
      texture_coords_attr as u32,
      2,
      gl::FLOAT,
      gl::FALSE,
      (4 * std::mem::size_of::<f32>()) as i32,
      (2 * std::mem::size_of::<f32>()) as *const GLvoid,
    );

    gl.UseProgram(low_res_prg);
    gl.Uniform1i(
      gl.GetUniformLocation(low_res_prg, CString::new("uTexture").unwrap().into_raw()),
      0,
    );

    gl.BindFramebuffer(gl::FRAMEBUFFER, fbo);

    let mut fbo_texture = 0;
    gl.GenTextures(1, &mut fbo_texture);
    gl.BindTexture(gl::TEXTURE_2D, fbo_texture);
    gl.TexImage2D(
      gl::TEXTURE_2D,
      0,
      gl::RGB as i32,
      SCREEN_RENDER_WIDTH,
      SCREEN_RENDER_HEIGHT,
      0,
      gl::RGB,
      gl::UNSIGNED_BYTE,
      std::ptr::null(),
    );
    gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    gl.FramebufferTexture2D(
      gl::FRAMEBUFFER,
      gl::COLOR_ATTACHMENT0,
      gl::TEXTURE_2D,
      fbo_texture,
      0,
    );

    let mut rbo = 0;
    gl.GenRenderbuffers(1, &mut rbo);
    gl.BindRenderbuffer(gl::RENDERBUFFER, rbo);
    gl.RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, SCREEN_RENDER_WIDTH, SCREEN_RENDER_HEIGHT);
    gl.FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo);
    if gl.CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
      println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
    }
    gl.BindFramebuffer(gl::FRAMEBUFFER, 0);

    (vao, vbo, fbo, fbo_texture)
  };

  Ok(OpenglCtx {
    clear_color: ColorGl::from(RGB_CLEAR_COLOR),
    frame_buffer: LowResFrameBuffer {
      vao: fbo_vao,
      vbo: fbo_vbo,
      fbo,
      texture2d: fbo_texture,
      shader_program: low_res_prg,
    },
    viewport: (SCREEN_WIDTH as GLsizei, SCREEN_HEIGHT as GLsizei),
  })
}

pub fn delete(gl: &Gl, opengl_ctx: &OpenglCtx) {
  unsafe{
    gl.DeleteVertexArrays(1, &opengl_ctx.frame_buffer.vao);
    gl.DeleteBuffers(1, &opengl_ctx.frame_buffer.vbo);
    gl.DeleteBuffers(1, &opengl_ctx.frame_buffer.texture2d);
    gl.DeleteProgram(opengl_ctx.frame_buffer.shader_program);
    gl.DeleteFramebuffers(1, &opengl_ctx.frame_buffer.fbo);
  }
}

pub fn render_gl(gl: &Gl, opengl_ctx: &OpenglCtx) -> Result<(), String> {
  let OpenglCtx { clear_color, frame_buffer, viewport: (w, h) } = opengl_ctx;
  unsafe {
    gl.BindFramebuffer(gl::FRAMEBUFFER, frame_buffer.fbo);
    gl.Viewport(0, 0, SCREEN_RENDER_WIDTH, SCREEN_RENDER_HEIGHT);
    gl.Enable(gl::DEPTH_TEST);
    gl.ClearColor(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
    gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

    // render scene

    gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
    gl.Viewport(0, 0, *w, *h);
    gl.Disable(gl::DEPTH_TEST);
    gl.UseProgram(frame_buffer.shader_program);
    gl.BindVertexArray(frame_buffer.vao);
    gl.ActiveTexture(gl::TEXTURE0);
    gl.BindTexture(gl::TEXTURE_2D, frame_buffer.texture2d);
    gl.DrawArrays(gl::TRIANGLES, 0, 6);
  }
  Ok(())
}
