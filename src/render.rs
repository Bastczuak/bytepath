#[allow(clippy::all)]
pub mod gl {
  include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use crate::{
  color::ColorGl,
  environment::{SCREEN_HEIGHT, SCREEN_RENDER_HEIGHT, SCREEN_RENDER_WIDTH, SCREEN_WIDTH},
  render::gl::types::*,
  resources::{Character, DrawBuffers, LineGeometry, QuadGeometry, TextBuffers},
  Camera, CircleGeometry, RGB_CLEAR_COLOR,
};
use bevy_ecs::system::{Res, ResMut};
use freetype as ft;
use lyon::{
  lyon_tessellation::{FillOptions, FillTessellator, FillVertex, FillVertexConstructor},
  math::{point, Point},
  path::Path,
  tessellation::{
    geometry_builder::simple_builder, StrokeOptions, StrokeTessellator, StrokeVertex, StrokeVertexConstructor,
    VertexBuffers,
  },
};
use std::ffi::CString;

macro_rules! get_offset {
  ($type:ty, $field:tt) => {{
    let dummy = core::mem::MaybeUninit::<$type>::uninit();
    let dummy_ptr = dummy.as_ptr();
    let field_ptr = core::ptr::addr_of!((*dummy_ptr).$field);
    field_ptr as usize - dummy_ptr as usize
  }};
}
macro_rules! cstr {
  ($literal:expr) => {
    (std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($literal, "\0").as_bytes()))
  };
}

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

const SCENE_VERTEX_SHADER: &str = r#"
#version 330 core

layout (location = 0) in mat4 Transform;
layout (location = 4) in vec4 Color;
layout (location = 5) in vec2 Position;

uniform mat4 uMVP;

out VERTEX_SHADER_OUTPUT {
  vec4 Color;
} OUT;

void main() {
  gl_Position = uMVP * Transform * vec4(Position, 0.0, 1.0);
  OUT.Color = Color;
}
"#;

const SCENE_FRAGMENT_SHADER: &str = r#"
#version 330 core

in VERTEX_SHADER_OUTPUT {
  vec4 Color;
} IN;

out vec4 Color;

void main() {
  Color = IN.Color;
}
"#;

const TEXT_VERTEX_SHADER: &str = r#"
#version 330 core

layout (location = 0) in vec4 PosTex;
layout (location = 1) in vec4 Color;

uniform mat4 uProjection;

out VERTEX_SHADER_OUTPUT {
  vec2 TexCoords;
  vec4 Color;
} OUT;

void main() {
  gl_Position = uProjection * vec4(PosTex.xy, 0.0, 1.0);
  OUT.TexCoords = PosTex.zw;
  OUT.Color = Color;
}
"#;

const TEXT_FRAGMENT_SHADER: &str = r#"
#version 330 core

in VERTEX_SHADER_OUTPUT {
  vec2 TexCoords;
  vec4 Color;
} IN;

out vec4 Color;

uniform sampler2D uTexture;

void main() {
  vec4 sampled = vec4(1.0, 1.0, 1.0, texture(uTexture, IN.TexCoords).r);
  Color = IN.Color * sampled;
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
  scene_program: GLuint,
  text_program: GLuint,
  pub viewport: (GLsizei, GLsizei),
}

#[repr(C)]
#[derive(Debug)]
pub struct MyVertex {
  transform_mat4_1: [f32; 4],
  transform_mat4_2: [f32; 4],
  transform_mat4_3: [f32; 4],
  transform_mat4_4: [f32; 4],
  color_rgba: [f32; 4],
  position: [f32; 2],
}

#[repr(C)]
#[derive(Debug)]
pub struct MyTextVertex {
  pub pos_tex: [f32; 4],
  pub color_rgba: [f32; 4],
}

pub struct WithTransformColor {
  pub transform: glam::Mat4,
  pub color_rgba: ColorGl,
}

impl StrokeVertexConstructor<MyVertex> for WithTransformColor {
  fn new_vertex(&mut self, vertex: StrokeVertex) -> MyVertex {
    let t = self.transform.to_cols_array_2d();
    MyVertex {
      transform_mat4_1: t[0],
      transform_mat4_2: t[1],
      transform_mat4_3: t[2],
      transform_mat4_4: t[3],
      color_rgba: self.color_rgba.to_array(),
      position: vertex.position().to_array(),
    }
  }
}

impl FillVertexConstructor<MyVertex> for WithTransformColor {
  fn new_vertex(&mut self, vertex: FillVertex) -> MyVertex {
    let t = self.transform.to_cols_array_2d();
    MyVertex {
      transform_mat4_1: t[0],
      transform_mat4_2: t[1],
      transform_mat4_3: t[2],
      transform_mat4_4: t[3],
      color_rgba: self.color_rgba.to_array(),
      position: vertex.position().to_array(),
    }
  }
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

pub fn calculate_size_for_lines() -> VertexBuffers<Point, u16> {
  let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
  let mut vertex_builder = simple_builder(&mut geometry);
  let mut tessellator = StrokeTessellator::new();
  let mut builder = Path::builder();
  builder.begin(point(0.0, 0.0));
  builder.line_to(point(0.0, 1.0));
  builder.close();

  tessellator
    .tessellate_path(&builder.build(), &StrokeOptions::default(), &mut vertex_builder)
    .unwrap();

  geometry
}

pub fn calculate_size_for_circles() -> VertexBuffers<Point, u16> {
  let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
  let mut vertex_builder = simple_builder(&mut geometry);
  let mut tessellator = StrokeTessellator::new();
  tessellator
    .tessellate_circle(
      Point::new(0.0, 0.0),
      16.0,
      &StrokeOptions::default(),
      &mut vertex_builder,
    )
    .unwrap();

  geometry
}

pub fn calculate_size_for_quads() -> VertexBuffers<Point, u16> {
  let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
  let mut vertex_builder = simple_builder(&mut geometry);
  let mut tessellator = FillTessellator::new();
  tessellator
    .tessellate_circle(Point::new(0.0, 0.0), 16.0, &FillOptions::default(), &mut vertex_builder)
    .unwrap();

  geometry
}

pub fn create_draw_buffer<T>(
  gl: &Gl,
  opengl_ctx: &OpenglCtx,
  get_vertex_buffer: fn() -> VertexBuffers<Point, u16>,
) -> DrawBuffers<T> {
  unsafe {
    let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
    let vertex_buffer = get_vertex_buffer();

    gl.GenVertexArrays(1, &mut vao);
    gl.GenBuffers(1, &mut vbo);
    gl.GenBuffers(1, &mut ebo);
    gl.BindVertexArray(vao);
    gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl.BufferData(
      gl::ARRAY_BUFFER,
      (std::mem::size_of::<MyVertex>() * vertex_buffer.vertices.len() * 10000) as GLsizeiptr,
      std::ptr::null(),
      gl::DYNAMIC_DRAW,
    );
    gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
    gl.BufferData(
      gl::ELEMENT_ARRAY_BUFFER,
      (std::mem::size_of::<u16>() * vertex_buffer.indices.len() * 10000) as GLsizeiptr,
      std::ptr::null(),
      gl::DYNAMIC_DRAW,
    );

    let transform_attr = gl.GetAttribLocation(opengl_ctx.scene_program, cstr!("Transform").as_ptr()) as GLuint;
    gl.EnableVertexAttribArray(transform_attr);
    gl.VertexAttribPointer(
      transform_attr,
      4,
      gl::FLOAT,
      gl::FALSE,
      (std::mem::size_of::<MyVertex>()) as i32,
      get_offset!(MyVertex, transform_mat4_1) as *const GLvoid,
    );
    gl.EnableVertexAttribArray(transform_attr + 1);
    gl.VertexAttribPointer(
      transform_attr + 1,
      4,
      gl::FLOAT,
      gl::FALSE,
      (std::mem::size_of::<MyVertex>()) as i32,
      get_offset!(MyVertex, transform_mat4_2) as *const GLvoid,
    );
    gl.EnableVertexAttribArray(transform_attr + 2);
    gl.VertexAttribPointer(
      transform_attr + 2,
      4,
      gl::FLOAT,
      gl::FALSE,
      (std::mem::size_of::<MyVertex>()) as i32,
      get_offset!(MyVertex, transform_mat4_3) as *const GLvoid,
    );
    gl.EnableVertexAttribArray(transform_attr + 3);
    gl.VertexAttribPointer(
      transform_attr + 3,
      4,
      gl::FLOAT,
      gl::FALSE,
      (std::mem::size_of::<MyVertex>()) as i32,
      get_offset!(MyVertex, transform_mat4_4) as *const GLvoid,
    );
    let color_attr = gl.GetAttribLocation(opengl_ctx.scene_program, cstr!("Color").as_ptr());
    gl.EnableVertexAttribArray(color_attr as u32);
    gl.VertexAttribPointer(
      color_attr as u32,
      4,
      gl::FLOAT,
      gl::FALSE,
      (std::mem::size_of::<MyVertex>()) as i32,
      get_offset!(MyVertex, color_rgba) as *const GLvoid,
    );

    let pos_attr = gl.GetAttribLocation(opengl_ctx.scene_program, cstr!("Position").as_ptr());
    gl.EnableVertexAttribArray(pos_attr as u32);
    gl.VertexAttribPointer(
      pos_attr as u32,
      2,
      gl::FLOAT,
      gl::FALSE,
      (std::mem::size_of::<MyVertex>()) as i32,
      get_offset!(MyVertex, position) as *const GLvoid,
    );

    DrawBuffers::<T>::new(vao, vbo, ebo)
  }
}

pub fn create_text_buffer(gl: &Gl, opengl_ctx: &OpenglCtx) -> TextBuffers {
  let path = std::path::Path::new("m5x7.ttf");
  let library = ft::Library::init().unwrap();
  let face = library.new_face(path, 0).unwrap();
  face.set_pixel_sizes(0, 48).unwrap();

  let (atlas_texture, characters) = unsafe {
    let (mut w, mut h) = (0, 0);
    for c in 32..127 {
      if face.load_char(c, ft::face::LoadFlag::RENDER).is_ok() {
        w += face.glyph().bitmap().width();
        h = h.max(face.glyph().bitmap().rows());
      } else {
        eprintln!("could not load character {}", c as u8 as char);
      }
    }

    let mut texture = 0;
    gl.GenTextures(1, &mut texture);
    gl.BindTexture(gl::TEXTURE_2D, texture);
    gl.TexImage2D(
      gl::TEXTURE_2D,
      0,
      gl::RED as i32,
      w,
      h,
      0,
      gl::RED,
      gl::UNSIGNED_BYTE,
      std::ptr::null(),
    );
    gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
    gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

    let mut x = 0;
    let mut characters = std::collections::HashMap::<char, Character>::new();
    gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);

    for c in 32..127 {
      if face.load_char(c, ft::face::LoadFlag::RENDER).is_ok() {
        gl.TexSubImage2D(
          gl::TEXTURE_2D,
          0,
          x,
          0,
          face.glyph().bitmap().width(),
          face.glyph().bitmap().rows(),
          gl::RED,
          gl::UNSIGNED_BYTE,
          face.glyph().bitmap().buffer().as_ptr() as *const GLvoid,
        );

        let character = Character {
          tx: x as f32 / w as f32,
          tx_1: (x as f32 + face.glyph().bitmap().width() as f32) / w as f32,
          ty: face.glyph().bitmap().rows() as f32 / h as f32,
          width: face.glyph().bitmap().width() as f32,
          height: face.glyph().bitmap().rows() as f32,
          bearing: glam::vec2(face.glyph().bitmap_left() as f32, face.glyph().bitmap_top() as f32),
          advance: (face.glyph().advance().x >> 6) as f32,
        };
        characters.insert(c as u8 as char, character);

        x += face.glyph().bitmap().width();
      } else {
        eprintln!("could not load character {}", c as u8 as char);
      }
    }

    gl.BindTexture(gl::TEXTURE_2D, 0);

    (texture, characters)
  };

  let (vao, vbo) = unsafe {
    let (mut vao, mut vbo) = (0, 0);
    gl.GenVertexArrays(1, &mut vao);
    gl.GenBuffers(1, &mut vbo);
    gl.BindVertexArray(vao);
    gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl.BufferData(
      gl::ARRAY_BUFFER,
      (6 * std::mem::size_of::<MyVertex>()) as GLsizeiptr,
      std::ptr::null(),
      gl::DYNAMIC_DRAW,
    );

    let pos_tex_attr = gl.GetAttribLocation(opengl_ctx.text_program, cstr!("PosTex").as_ptr());
    gl.EnableVertexAttribArray(pos_tex_attr as u32);
    gl.VertexAttribPointer(
      pos_tex_attr as u32,
      4,
      gl::FLOAT,
      gl::FALSE,
      (std::mem::size_of::<MyTextVertex>()) as i32,
      get_offset!(MyTextVertex, pos_tex) as *const GLvoid,
    );

    let color_attr = gl.GetAttribLocation(opengl_ctx.text_program, cstr!("Color").as_ptr());
    gl.EnableVertexAttribArray(color_attr as u32);
    gl.VertexAttribPointer(
      color_attr as u32,
      4,
      gl::FLOAT,
      gl::FALSE,
      (std::mem::size_of::<MyTextVertex>()) as i32,
      get_offset!(MyTextVertex, color_rgba) as *const GLvoid,
    );

    gl.BindBuffer(gl::ARRAY_BUFFER, 0);
    gl.BindVertexArray(0);

    (vao, vbo)
  };

  TextBuffers {
    vao,
    vbo,
    atlas_texture,
    characters,
    vertex_buffer: Vec::new(),
  }
}

pub fn init(gl: &Gl) -> Result<OpenglCtx, String> {
  let low_res_prg = create_shader_program(gl, FBO_VERTEX_SHADER, FBO_FRAGMENT_SHADER)?;
  let scene_prg = create_shader_program(gl, SCENE_VERTEX_SHADER, SCENE_FRAGMENT_SHADER)?;
  let text_prg = create_shader_program(gl, TEXT_VERTEX_SHADER, TEXT_FRAGMENT_SHADER)?;
  let (fbo_vao, fbo_vbo, fbo, fbo_texture) = unsafe {
    let (mut vao, mut vbo) = (0, 0);
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

    let pos_attr = gl.GetAttribLocation(low_res_prg, cstr!("Position").as_ptr());
    gl.EnableVertexAttribArray(pos_attr as u32);
    gl.VertexAttribPointer(
      pos_attr as u32,
      2,
      gl::FLOAT,
      gl::FALSE,
      (4 * std::mem::size_of::<f32>()) as i32,
      std::ptr::null(),
    );

    let texture_coords_attr = gl.GetAttribLocation(low_res_prg, cstr!("TexCoords").as_ptr());
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
    gl.Uniform1i(gl.GetUniformLocation(low_res_prg, cstr!("uTexture").as_ptr()), 0);

    let mut fbo = 0;
    gl.GenFramebuffers(1, &mut fbo);
    gl.BindFramebuffer(gl::FRAMEBUFFER, fbo);

    let mut fbo_texture = 0;
    gl.GenTextures(1, &mut fbo_texture);
    gl.BindTexture(gl::TEXTURE_2D, fbo_texture);
    gl.TexImage2D(
      gl::TEXTURE_2D,
      0,
      gl::RGB as i32,
      SCREEN_WIDTH,
      SCREEN_HEIGHT,
      0,
      gl::RGB,
      gl::UNSIGNED_BYTE,
      std::ptr::null(),
    );
    gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, fbo_texture, 0);

    let mut rbo = 0;
    gl.GenRenderbuffers(1, &mut rbo);
    gl.BindRenderbuffer(gl::RENDERBUFFER, rbo);
    gl.RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, SCREEN_WIDTH, SCREEN_HEIGHT);
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
    scene_program: scene_prg,
    text_program: text_prg,
    viewport: (SCREEN_RENDER_WIDTH as GLsizei, SCREEN_RENDER_HEIGHT as GLsizei),
  })
}

pub type RenderSystemState<'w, 's> = (
  Res<'w, Camera>,
  ResMut<'w, CircleGeometry>,
  ResMut<'w, QuadGeometry>,
  ResMut<'w, LineGeometry>,
  ResMut<'w, TextBuffers>,
);

pub fn render_gl(gl: &Gl, opengl_ctx: &OpenglCtx, render_state: RenderSystemState) -> Result<(), String> {
  let (camera, mut circles, mut quads, mut lines, mut texts) = render_state;
  let OpenglCtx {
    clear_color,
    frame_buffer,
    scene_program,
    text_program,
    viewport: (w, h),
  } = opengl_ctx;

  unsafe fn draw<T>(gl: &Gl, buffers: &mut DrawBuffers<T>) {
    gl.BindVertexArray(buffers.vao);
    gl.BindBuffer(gl::ARRAY_BUFFER, buffers.vbo);
    gl.BufferSubData(
      gl::ARRAY_BUFFER,
      0,
      (buffers.vertex_buffer.vertices.len() * std::mem::size_of::<MyVertex>()) as GLsizeiptr,
      buffers.vertex_buffer.vertices.as_ptr() as *const GLvoid,
    );
    gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffers.ebo);
    gl.BufferSubData(
      gl::ELEMENT_ARRAY_BUFFER,
      0,
      (buffers.vertex_buffer.indices.len() * std::mem::size_of::<u16>()) as GLsizeiptr,
      buffers.vertex_buffer.indices.as_ptr() as *const GLvoid,
    );
    gl.DrawElements(
      gl::TRIANGLES,
      buffers.vertex_buffer.indices.len() as i32,
      gl::UNSIGNED_SHORT,
      std::ptr::null(),
    );
    buffers.vertex_buffer.vertices.clear();
    buffers.vertex_buffer.indices.clear();
  }

  unsafe {
    gl.BindFramebuffer(gl::FRAMEBUFFER, frame_buffer.fbo);
    gl.Viewport(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
    gl.Enable(gl::DEPTH_TEST);
    gl.ClearColor(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
    gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

    //----------------------SCENE----------------------//
    let Camera {
      camera_pos,
      camera_front,
      camera_up,
      camera_zoom,
      ..
    } = *camera;
    let view = glam::Mat4::look_at_rh(camera_pos, camera_pos + camera_front, camera_up);
    let projection = glam::Mat4::orthographic_rh_gl(0.0, SCREEN_WIDTH as f32, 0.0, SCREEN_HEIGHT as f32, -100.0, 100.0)
      * glam::Mat4::from_scale(camera_zoom);

    gl.UseProgram(*scene_program);
    let mvp_mat = {
      let model = glam::Mat4::from_rotation_z(0.0f32.to_radians());
      projection * view * model
    };
    gl.UniformMatrix4fv(
      gl.GetUniformLocation(*scene_program, cstr!("uMVP").as_ptr()),
      1,
      gl::FALSE,
      mvp_mat.to_cols_array().as_ptr(),
    );

    draw(gl, &mut circles);
    draw(gl, &mut quads);
    draw(gl, &mut lines);

    //----------------------SCENE----------------------//

    gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
    gl.Viewport(0, 0, *w, *h);
    gl.Disable(gl::DEPTH_TEST);
    gl.UseProgram(frame_buffer.shader_program);
    gl.BindVertexArray(frame_buffer.vao);
    gl.ActiveTexture(gl::TEXTURE0);
    gl.BindTexture(gl::TEXTURE_2D, frame_buffer.texture2d);
    gl.DrawArrays(gl::TRIANGLES, 0, 6);

    //----------------------TEXT----------------------//
    gl.Enable(gl::BLEND);
    gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    gl.UseProgram(*text_program);
    gl.ActiveTexture(gl::TEXTURE0);
    gl.BindTexture(gl::TEXTURE_2D, texts.atlas_texture);

    let projection = glam::Mat4::orthographic_rh_gl(0.0, *w as f32, 0.0, *h as f32, -10.0, 10.0);
    gl.UniformMatrix4fv(
      gl.GetUniformLocation(*text_program, cstr!("uProjection").as_ptr()),
      1,
      gl::FALSE,
      projection.to_cols_array().as_ptr(),
    );

    gl.BindVertexArray(texts.vao);
    gl.BindBuffer(gl::ARRAY_BUFFER, texts.vbo);
    gl.BufferData(
      gl::ARRAY_BUFFER,
      (texts.vertex_buffer.len() * std::mem::size_of::<MyTextVertex>()) as GLsizeiptr,
      texts.vertex_buffer.as_ptr() as *const GLvoid,
      gl::DYNAMIC_DRAW,
    );
    gl.BindBuffer(gl::ARRAY_BUFFER, 0);
    gl.DrawArrays(gl::TRIANGLES, 0, texts.vertex_buffer.len() as i32);

    gl.BindVertexArray(0);
    gl.BindTexture(gl::TEXTURE_2D, 0);
    gl.Disable(gl::BLEND);
    texts.vertex_buffer.clear();
    //----------------------TEXT----------------------//
  }
  Ok(())
}

pub fn delete(gl: &Gl, opengl_ctx: &OpenglCtx, render_state: RenderSystemState) {
  let (_, circles, quads, lines, texts) = render_state;
  unsafe {
    gl.DeleteVertexArrays(1, &opengl_ctx.frame_buffer.vao);
    gl.DeleteVertexArrays(1, &circles.vao);
    gl.DeleteVertexArrays(1, &quads.vao);
    gl.DeleteVertexArrays(1, &lines.vao);
    gl.DeleteVertexArrays(1, &texts.vao);
    gl.DeleteBuffers(1, &opengl_ctx.frame_buffer.vbo);
    gl.DeleteBuffers(1, &opengl_ctx.frame_buffer.texture2d);
    gl.DeleteBuffers(1, &circles.vbo);
    gl.DeleteBuffers(1, &quads.vbo);
    gl.DeleteBuffers(1, &lines.vbo);
    gl.DeleteBuffers(1, &texts.vbo);
    gl.DeleteBuffers(1, &circles.ebo);
    gl.DeleteBuffers(1, &quads.ebo);
    gl.DeleteBuffers(1, &lines.ebo);
    gl.DeleteBuffers(1, &texts.atlas_texture);
    gl.DeleteProgram(opengl_ctx.frame_buffer.shader_program);
    gl.DeleteProgram(opengl_ctx.scene_program);
    gl.DeleteProgram(opengl_ctx.text_program);
    gl.DeleteFramebuffers(1, &opengl_ctx.frame_buffer.fbo);
  }
}
