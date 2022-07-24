#[allow(clippy::all)]
mod gl {
  include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use crate::{
  color::ColorGl,
  environment::{SCREEN_HEIGHT, SCREEN_RENDER_HEIGHT, SCREEN_RENDER_WIDTH, SCREEN_WIDTH},
  render::gl::types::*,
  Camera, RGB_CLEAR_COLOR,
};
use lyon::{
  geom::{euclid::Box2D, Size},
  math::Point,
  tessellation::{
    geometry_builder::simple_builder, BuffersBuilder, StrokeOptions, StrokeTessellator, StrokeVertex,
    StrokeVertexConstructor, VertexBuffers,
  },
};
use std::ffi::CString;

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
  inner: std::sync::Arc<gl::Gl>,
}

impl Gl {
  pub fn load_with<F>(load_fn: F) -> Self
    where
      F: FnMut(&'static str) -> *const GLvoid,
  {
    Self {
      inner: std::sync::Arc::new(gl::Gl::load_with(load_fn)),
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

pub struct Scene {
  vao: GLuint,
  vbo: GLuint,
  ebo: GLuint,
  shader_program: GLuint,
}

pub struct OpenglCtx {
  clear_color: ColorGl,
  frame_buffer: LowResFrameBuffer,
  scene: Scene,
  pub viewport: (GLsizei, GLsizei),
}

#[repr(C)]
struct MyVertex {
  transform_mat4_1: [f32; 4],
  transform_mat4_2: [f32; 4],
  transform_mat4_3: [f32; 4],
  transform_mat4_4: [f32; 4],
  color_rgba: [f32; 4],
  position: [f32; 2],
}

struct MyVertexConfig {
  transform: glam::Mat4,
  color_rgba: glam::Vec4,
}

impl StrokeVertexConstructor<MyVertex> for MyVertexConfig {
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

macro_rules! get_offset {
  ($type:ty, $field:tt) => {{
    let dummy = core::mem::MaybeUninit::<$type>::uninit();
    let dummy_ptr = dummy.as_ptr();
    let field_ptr = core::ptr::addr_of!((*dummy_ptr).$field);
    field_ptr as usize - dummy_ptr as usize
  }};
}

pub fn init(gl: &Gl) -> Result<OpenglCtx, String> {
  let low_res_prg = create_shader_program(gl, FBO_VERTEX_SHADER, FBO_FRAGMENT_SHADER)?;
  let scene_prg = create_shader_program(gl, SCENE_VERTEX_SHADER, SCENE_FRAGMENT_SHADER)?;
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
  let (scene_vao, scene_vbo, scene_ebo) = unsafe {
    let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
    let mut vertex_builder = simple_builder(&mut geometry);
    let mut tessellator = StrokeTessellator::new();
    let mut options = StrokeOptions::default();
    options.line_width = 0.1;
    tessellator
      .tessellate_circle(Point::new(0.0, 0.0), 16.0, &options, &mut vertex_builder)
      .unwrap();
    let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
    gl.GenVertexArrays(1, &mut vao);

    gl.GenBuffers(1, &mut vbo);
    gl.GenBuffers(1, &mut ebo);
    gl.BindVertexArray(vao);
    gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl.BufferData(
      gl::ARRAY_BUFFER,
      (std::mem::size_of::<MyVertex>() * geometry.vertices.len() * 10000) as GLsizeiptr,
      std::ptr::null(),
      gl::DYNAMIC_DRAW,
    );
    gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
    gl.BufferData(
      gl::ELEMENT_ARRAY_BUFFER,
      (std::mem::size_of::<u16>() * geometry.indices.len() * 10000) as GLsizeiptr,
      std::ptr::null(),
      gl::DYNAMIC_DRAW,
    );

    let transform_attr = gl.GetAttribLocation(scene_prg, CString::new("Transform").unwrap().into_raw()) as GLuint;
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
    let color_attr = gl.GetAttribLocation(scene_prg, CString::new("Color").unwrap().into_raw());
    gl.EnableVertexAttribArray(color_attr as u32);
    gl.VertexAttribPointer(
      color_attr as u32,
      4,
      gl::FLOAT,
      gl::FALSE,
      (std::mem::size_of::<MyVertex>()) as i32,
      get_offset!(MyVertex, color_rgba) as *const GLvoid,
    );

    let pos_attr = gl.GetAttribLocation(scene_prg, CString::new("Position").unwrap().into_raw());
    gl.EnableVertexAttribArray(pos_attr as u32);
    gl.VertexAttribPointer(
      pos_attr as u32,
      2,
      gl::FLOAT,
      gl::FALSE,
      (std::mem::size_of::<MyVertex>()) as i32,
      get_offset!(MyVertex, position) as *const GLvoid,
    );

    (vao, vbo, ebo)
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
    scene: Scene {
      vao: scene_vao,
      vbo: scene_vbo,
      ebo: scene_ebo,
      shader_program: scene_prg,
    },
    viewport: (SCREEN_RENDER_WIDTH as GLsizei, SCREEN_RENDER_HEIGHT as GLsizei),
  })
}

pub fn delete(gl: &Gl, opengl_ctx: &OpenglCtx) {
  unsafe {
    gl.DeleteVertexArrays(1, &opengl_ctx.frame_buffer.vao);
    gl.DeleteVertexArrays(1, &opengl_ctx.scene.vao);
    gl.DeleteBuffers(1, &opengl_ctx.frame_buffer.vbo);
    gl.DeleteBuffers(1, &opengl_ctx.frame_buffer.texture2d);
    gl.DeleteBuffers(1, &opengl_ctx.scene.vbo);
    gl.DeleteBuffers(1, &opengl_ctx.scene.ebo);
    gl.DeleteProgram(opengl_ctx.frame_buffer.shader_program);
    gl.DeleteProgram(opengl_ctx.scene.shader_program);
    gl.DeleteFramebuffers(1, &opengl_ctx.frame_buffer.fbo);
  }
}

pub fn render_gl(gl: &Gl, opengl_ctx: &OpenglCtx, camera: &Camera) -> Result<(), String> {
  let OpenglCtx {
    clear_color,
    frame_buffer,
    viewport: (w, h),
    scene,
  } = opengl_ctx;
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
    let projection = glam::Mat4::orthographic_rh_gl(
      -SCREEN_WIDTH as f32 * 0.5,
      SCREEN_WIDTH as f32 * 0.5,
      -SCREEN_HEIGHT as f32 * 0.5,
      SCREEN_HEIGHT as f32 * 0.5,
      -100.0,
      100.0,
    ) * glam::Mat4::from_scale(camera_zoom);

    let mut geometry: VertexBuffers<MyVertex, u16> = VertexBuffers::new();
    {
      let mut tessellator = StrokeTessellator::new();
      let mut options = StrokeOptions::default();
      options.line_width = 1.0;
      let radius = 16.0;
      let transform = glam::Mat4::from_rotation_translation(
        glam::Quat::from_axis_angle(glam::Vec3::new(0.0, 0.0, 1.0), 45.0f32.to_radians()),
        glam::Vec3::new(0.0, 0.0, -1.0),
      );

      tessellator
        .tessellate_circle(
          Point::new(0.0, 0.0),
          radius,
          &options,
          &mut BuffersBuilder::new(
            &mut geometry,
            MyVertexConfig {
              transform,
              color_rgba: glam::Vec4::new(0.0, 1.0, 0.0, 1.0),
            },
          ),
        )
        .unwrap();

      let (w, h) = (16.0, 16.0);
      let transform = glam::Mat4::from_rotation_translation(
        glam::Quat::from_axis_angle(glam::Vec3::new(0.0, 0.0, 1.0), 20.0f32.to_radians()),
        glam::Vec3::new(0.0, 0.0, -40.0),
      ) * glam::Mat4::from_translation(glam::Vec3::new(w / -2.0, h / -2.0, 0.0));
      tessellator
        .tessellate_rectangle(
          &Box2D::from_origin_and_size(Point::new(0.0, 0.0), Size::new(w, h)),
          &options,
          &mut BuffersBuilder::new(
            &mut geometry,
            MyVertexConfig {
              color_rgba: glam::Vec4::new(1.0, 0.0, 0.0, 1.0),
              transform,
            },
          ),
        )
        .unwrap();
    }

    gl.UseProgram(scene.shader_program);
    gl.BindVertexArray(scene.vao);
    gl.BindBuffer(gl::ARRAY_BUFFER, scene.vbo);
    gl.BufferSubData(
      gl::ARRAY_BUFFER,
      0,
      (geometry.vertices.len() * std::mem::size_of::<MyVertex>()) as GLsizeiptr,
      geometry.vertices.as_ptr() as *const GLvoid,
    );
    gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, scene.ebo);
    gl.BufferSubData(
      gl::ELEMENT_ARRAY_BUFFER,
      0,
      (geometry.indices.len() * std::mem::size_of::<u16>()) as GLsizeiptr,
      geometry.indices.as_ptr() as *const GLvoid,
    );

    let mvp_mat = {
      let model = glam::Mat4::from_rotation_z(0.0f32.to_radians());
      projection * view * model
    };

    gl.UniformMatrix4fv(
      gl.GetUniformLocation(scene.shader_program, CString::new("uMVP").unwrap().into_raw()),
      1,
      gl::FALSE,
      mvp_mat.to_cols_array().as_ptr(),
    );

    gl.DrawElements(
      gl::TRIANGLES,
      geometry.indices.len() as i32,
      gl::UNSIGNED_SHORT,
      std::ptr::null(),
    );
    //----------------------SCENE----------------------//

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
