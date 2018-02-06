use std::{ffi, os, ptr, rc};

// Import OpenGL bindings.
include!(concat!(env!("OUT_DIR"), "/gl.rs"));

#[derive(Clone)]
pub struct Backend {
    gl: rc::Rc<Gl>,
}

impl Backend {
    /// Constructor.
    pub fn load<F>(mut func: F) -> Self
        where F: FnMut(&str) -> *const ()
    {
        let gl = rc::Rc::new(Gl::load_with(|sym| func(sym) as *const _));
        Backend { gl }
    }

    // Error checking

    /// Corresponds to `glGetError` plus an error check.
    pub fn check_error(&self) {
        let error = unsafe { self.gl.GetError() };
        if error != 0 {
            error!(target: "gl", "0x{:x}", error);
        }
    }

    // Pipeline state operations

    /// Corresponds to `glClear`.
    pub fn clear(&self, ops: u32) {
        trace!(target: "gl", "glClear{:?}", (ops,));
        unsafe {
            self.gl.Clear(ops);
        }
        self.check_error();
    }

    /// Corresponds to `glClearColor`.
    pub fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        trace!(target: "gl", "glClearColor{:?}", (r, g, b, a));
        unsafe {
            self.gl.ClearColor(r, g, b, a);
        }
        self.check_error();
    }

    /// Corresponds to `glClearDepth`.
    pub fn clear_depth(&self, z: f64) {
        trace!(target: "gl", "glClearDepth{:?}", (z,));
        unsafe {
            self.gl.ClearDepth(z);
        }
        self.check_error();
    }

    /// Corresponds to `glEnable`.
    pub fn enable(&self, state: u32) {
        trace!(target: "gl", "glEnable{:?}", (state,));
        unsafe {
            self.gl.Enable(state);
        }
        self.check_error();
    }

    /// Corresponds to `glDisable`.
    pub fn disable(&self, state: u32) {
        trace!(target: "gl", "glDisable{:?}", (state,));
        unsafe {
            self.gl.Disable(state);
        }
        self.check_error();
    }

    /// Corresponds to `glCullFace`.
    pub fn cull_face(&self, opt: u32) {
        trace!(target: "gl", "glCullFace{:?}", (opt,));
        unsafe {
            self.gl.CullFace(opt);
        }
        self.check_error();
    }

    /// Corresponds to `glFrontFace`.
    pub fn front_face(&self, opt: u32) {
        trace!(target: "gl", "glFrontFace{:?}", (opt,));
        unsafe {
            self.gl.FrontFace(opt);
        }
        self.check_error();
    }

    /// Corresponds to `glDepthFunc`.
    pub fn depth_func(&self, opt: u32) {
        trace!(target: "gl", "glDepthFunc{:?}", (opt,));
        unsafe {
            self.gl.DepthFunc(opt);
        }
        self.check_error();
    }

    /// Corresponds to `glPolygonMode`.
    pub fn polygon_mode(&self, face: u32, mode: u32) {
        trace!(target: "gl", "glPolygonMode{:?}", (face, mode));
        unsafe {
            self.gl.PolygonMode(face, mode);
        }
        self.check_error();
    }

    /// Corresponds to `glLineWidth`.
    pub fn line_width(&self, width: f32) {
        trace!(target: "gl", "glLineWidth{:?}", (width,));
        unsafe {
            self.gl.LineWidth(width);
        }
        self.check_error();
    }

    /// Corresponds to `glPointSize`.
    pub fn point_size(&self, size: f32) {
        trace!(target: "gl", "glPointSize{:?}", (size,));
        unsafe {
            self.gl.PointSize(size);
        }
        self.check_error();
    }

    /// Corresponds to `glViewport`.
    pub fn viewport(&self, x: u32, y: u32, w: u32, h: u32) {
        trace!(target: "gl", "glViewport{:?}", (x, y, w, h));
        unsafe {
            self.gl.Viewport(x as _, y as _, w as _, h as _);
        }
        self.check_error();
    }

    // Buffer operations

    /// Corresponds to `glGenBuffer`.
    pub fn gen_buffer(&self) -> u32 {
        let mut id: u32 = 0;
        unsafe {
            trace!(target: "gl", "glGenBuffers(1) ");
            self.gl.GenBuffers(1, &mut id as *mut _)
        };
        trace!(target: "gl", " => {}", id);
        self.check_error();
        id
    }

    /// Corresponds to `glBindBuffer`.
    pub fn bind_buffer(&self, id: u32, ty: u32) {
        unsafe {
            trace!(target: "gl", "glBindBuffer{:?}", (ty, id));
            self.gl.BindBuffer(ty, id);
        }
        self.check_error();
    }

    /// Corresponds to `glBufferData`.
    pub fn buffer_data<T>(&self, id: u32, len: usize, ptr: *const T, usage: u32) {
        unsafe {
            trace!(target: "gl", "glBufferData{:?}", (id, len, ptr, usage));
            self.gl.BufferData(id, len as _, ptr as *const _, usage);
        }
        self.check_error();
    }

    /// Corresponds to `glBufferSubData`.
    pub fn buffer_sub_data<T>(&self, ty: u32, off: usize, len: usize, ptr: *const T) {
        unsafe {
            trace!(target: "gl", "glBufferSubData{:?}", (ty, off, len, ptr));
            self.gl.BufferSubData(ty, off as _, len as _, ptr as *const _);
        }
        self.check_error();
    }

    // Vertex array operations

    /// Corresponds to `glGenVertexArrays`.
    pub fn gen_vertex_array(&self) -> u32 {
        let mut id: u32 = 0;
        unsafe {
            trace!(target: "gl", "glGenVertexArrays(1) ");
            self.gl.GenVertexArrays(1, &mut id as *mut _);
            trace!(target: "gl", "=> {}", id);
        }
        self.check_error();
        id
    }

    /// Corresponds to `glBindVertexArray`.
    pub fn bind_vertex_array(&self, id: u32) {
        unsafe {
            trace!(target: "gl", "glBindVertexArray{:?}", (id,));
            self.gl.BindVertexArray(id);
        }
        self.check_error();
    }

    /// Corresponds to `glVertexAttribPointer`.
    pub fn vertex_attrib_pointer(&self, id: u8, sz: i32, ty: u32, norm: bool, stride: i32, off: usize) {
        unsafe {
            trace!(target: "gl", "glVertexAttribPointer{:?}", (id, sz, ty, norm, stride, off));
            self.gl.VertexAttribPointer(id as _, sz as _, ty, if norm == true { 1 } else { 0 }, stride as _, off as *const _);
        }
        self.check_error();
    }

    /// Corresponds to `glEnableVertexAttribArray`.
    pub fn enable_vertex_attrib_array(&self, idx: u8) {
        unsafe {
            trace!(target: "gl", "glEnableVertexAttribArray{:?}", (idx,));
            self.gl.EnableVertexAttribArray(idx as _);
        }
        self.check_error();
    }

    // Framebuffer operations.

    /// Corresponds to `glGenFramebuffers(1)`.
    pub fn gen_framebuffer(&self) -> u32 {
        trace!(target: "gl", "glGenFramebuffers(1)");
        let mut id = 0;
        unsafe {
            self.gl.GenFramebuffers(1, &mut id as *mut _);
        }
        self.check_error();
        id
    }

    /// Corresponds to `glGenRenderbuffers(1)`.
    pub fn gen_renderbuffer(&self) -> u32 {
        trace!(target: "gl", "glGenRenderbuffers(1)");
        let mut id = 0;
        unsafe {
            self.gl.GenRenderbuffers(1, &mut id as *mut _);
        }
        self.check_error();
        id
    }

    /// Corresponds to `glBindRenderbuffer`.
    pub fn bind_renderbuffer(&self, id: u32) {
        trace!(target: "gl", "glBindRenderbuffer{:?} ", (RENDERBUFFER, id));
        unsafe {
            self.gl.BindRenderbuffer(RENDERBUFFER, id);
        }
        self.check_error();
    }

    /// Corresponds to `glRenderbufferStorage`.
    pub fn renderbuffer_storage(&self, format: u32, width: i32, height: i32) {
        trace!(
            target: "gl",
            "glBindRenderbufferStorage{:?} ",
            (RENDERBUFFER, format, width, height),
        );
        unsafe {
            self.gl.RenderbufferStorage(RENDERBUFFER, format, width, height);
        }
        self.check_error();
    }

    /// Corresponds to `glRenderbufferStorageMultisample`.
    pub fn renderbuffer_storage_multisample(
        &self,
        samples: i32,
        format: u32,
        width: i32,
        height: i32,
    ) {
        trace!(
            target: "gl",
            "glBindRenderbufferStorageMultisample{:?} ",
            (RENDERBUFFER, samples, format, width, height),
        );
        unsafe {
            self.gl.RenderbufferStorageMultisample(
                RENDERBUFFER,
                samples,
                format,
                width,
                height,
            );
        }
        self.check_error();
    }

    /// Corresponds to `glBindFramebuffer`.
    pub fn bind_framebuffer(&self, id: u32) {
        trace!(target: "gl", "glBindFramebuffer{:?} ", (FRAMEBUFFER, id));
        unsafe {
            self.gl.BindFramebuffer(FRAMEBUFFER, id);
        }
        self.check_error();
    }

    /// Corresponds to `glFramebufferTexture2D`.
    pub fn framebuffer_texture2d(&self, attachment: u32, texture: u32) {
        trace!(
            target: "gl",
            "glFramebufferTexture2D{:?}",
            (
                FRAMEBUFFER,
                COLOR_ATTACHMENT0 + attachment,
                TEXTURE_2D,
                texture,
                0,
            ),
        );
        unsafe {
            self.gl.FramebufferTexture2D(
                FRAMEBUFFER,
                COLOR_ATTACHMENT0 + attachment,
                TEXTURE_2D,
                texture,
                0,
            );
        }
        self.check_error();
    }

    /// Corresponds to `glFramebufferRenderbuffer`.
    pub fn framebuffer_renderbuffer(&self, attachment: u32, renderbuffer: u32) {
        trace!(
            target: "gl",
            "glFramebufferRenderbuffer{:?}",
            (
                FRAMEBUFFER,
                COLOR_ATTACHMENT0 + attachment,
                RENDERBUFFER,
                renderbuffer,
            ),
        );
        unsafe {
            self.gl.FramebufferRenderbuffer(
                FRAMEBUFFER,
                COLOR_ATTACHMENT0 + attachment,
                RENDERBUFFER,
                renderbuffer,
            );
        }
        self.check_error();
    }

    /// Corresponds to `glDrawBuffers`.
    pub fn draw_buffers(&self, buffers: &[u32]) {
        trace!(target: "gl", "glDrawBuffers{:?}", (buffers.len(), buffers));
        unsafe {
            self.gl.DrawBuffers(buffers.len() as _, buffers.as_ptr() as _);
        }
        self.check_error();
    }
    
    // Program operations

    /// Corresponds to `glCreateShader`.
    pub fn create_shader(&self, ty: u32) -> u32 {
        let id = unsafe {
            trace!(target: "gl", "glCreateShader{:?} ", (ty,));
            self.gl.CreateShader(ty)
        };
        trace!(target: "gl", "=> {}", id);
        self.check_error();
        id
    }

    /// Corresponds to `glShaderSource`.
    pub fn shader_source(&self, id: u32, source: &ffi::CStr) {
        unsafe {
            trace!(target: "gl", "glShaderSource{:?}", (id, source));
            let ptr = source.as_ptr() as *const i8;
            self.gl.ShaderSource(id, 1, &ptr as *const _, ptr::null());
        }
        self.check_error();
    }

    /// Corresponds to `glCompileShader`.
    pub fn compile_shader(&self, id: u32) {
        let mut status = 0i32;
        unsafe {
            trace!(target: "gl", "glCompileShader{:?}", (id,));
            self.gl.CompileShader(id);
            self.check_error();
            self.gl.GetShaderiv(id, COMPILE_STATUS, &mut status as *mut _);
            self.check_error();
            if status == 0 {
                panic!("Shader compilation failed");
            }
        }
    }

    /// Corresponds to `glCreateProgram`.
    pub fn create_program(&self) -> u32 {
        let id = unsafe {
            trace!(target: "gl", "glCreateProgram() ");
            self.gl.CreateProgram()
        };
        trace!(target: "gl", "=> {}", id);
        self.check_error();
        id
    }

    /// Corresponds to `glAttachShader`.
    pub fn attach_shader(&self, program: u32, shader: u32) {
        unsafe {
            trace!(target: "gl", "glAttachShader{:?}", (program, shader));
            self.gl.AttachShader(program, shader);
        }
        self.check_error();
    }

    /// Corresponds to `glLinkProgram`.
    pub fn link_program(&self, id: u32) {
        let mut status = 0i32;
        unsafe {
            trace!(target: "gl", "glLinkProgram{:?}", (id,));
            self.gl.LinkProgram(id);
            self.check_error();
            trace!(target: "gl", "glGetProgramiv{:?} ", (id, LINK_STATUS));
            self.gl.GetProgramiv(id, LINK_STATUS, &mut status as *mut _);
            trace!(target: "gl", "=> {}", status);
            self.check_error();
            if status == 0i32 {
                panic!("Program linking failed");
            }
        }
    }

    /// Corresponds to `glGetUniformBlockIndex`.
    pub fn get_uniform_block_index(
        &self,
        id: u32,
        name: &ffi::CStr,
    ) -> u32 {
        let index;
        unsafe {
            trace!(target: "gl", "glGetUniformBlockIndex{:?} ", (id, name));
            index = self.gl.GetUniformBlockIndex(id, name.as_ptr() as _);
            trace!(target: "gl", "=> {}", index);
        }
        self.check_error();
        index
    }

    /// Corresponds to `glUniformBlockBinding`.
    pub fn uniform_block_binding(
        &self,
        program: u32,
        index: u32,
        binding: u32,
    ) {
        trace!(target: "gl", "glUniformBlockBinding{:?} ", (program, index, binding));
        unsafe {
            self.gl.UniformBlockBinding(program, index, binding);
        }
        self.check_error();
    }

    /// Corresponds to `glGetUniformLocation`.
    pub fn get_uniform_location(
        &self,
        id: u32,
        name: &ffi::CStr,
    ) -> i32 {
        let index;
        unsafe {
            trace!(target: "gl", "glGetUniformLocation{:?} ", (id, name));
            index = self.gl.GetUniformLocation(id, name.as_ptr() as _);
            trace!(target: "gl", "=> {}", index);
        }
        self.check_error();
        index
    }

    // Texture operations

    /// Corresponds to `glGenTextures`.
    pub fn gen_texture(&self) -> u32 {
        let mut id = INVALID_INDEX;
        unsafe {
            trace!(target: "gl", "glGenTextures(1) ");
            self.gl.GenTextures(1, &mut id as *mut _);
            trace!(target: "gl", "=> {}", id);
        }
        self.check_error();
        id
    }

    /// Corresponds to `glBindTexture`.
    pub fn bind_texture(&self, ty: u32, id: u32) {
        unsafe {
            trace!(target: "gl", "glBindTexture{:?}", (ty, id));
            self.gl.BindTexture(ty, id);
        }
        self.check_error();
    }

    /// Corresponds to `glTexParameteri`.
    pub fn tex_parameteri(&self, ty: u32, param: u32, value: u32) {
        unsafe {
            trace!(target: "gl", "glTexParameteri{:?}", (ty, param, value));
            self.gl.TexParameteri(ty, param, value as i32);
        }
        self.check_error();
    }

    /// Corresponds to `glTexImage2D`.
    pub fn tex_image_2d(
        &self,
        target: u32,
        internal_format: u32,
        width: u32,
        height: u32,
        format: u32,
        ty: u32,
        data: *const os::raw::c_void,
    ) {
        unsafe {
            trace!(target: "gl", 
                "glTexImage2D{:?}",
                (
                    target,
                    0,
                    internal_format,
                    width,
                    height,
                    0,
                    format,
                    ty,
                    data,
                ),
            );
            self.gl.TexImage2D(
                target,
                0,
                internal_format as _,
                width as _,
                height as _,
                0,
                format,
                ty,
                data,
            );
        }
        self.check_error();
    }

    /// Corresponds to `glGetTexImage`.
    pub fn get_tex_image(
        &self,
        target: u32,
        format: u32,
        ty: u32,
        ptr: *mut os::raw::c_void,
    ) {
        trace!(
            target: "gl",
            "glGetTexImage{:?}",
            (
                target,
                0,
                format,
                ty,
                ptr,
            ),
        );
        unsafe {
            self.gl.GetTexImage(
                target,
                0,
                format,
                ty,
                ptr,
            );
        }
        self.check_error();
    }

    /// Corresponds to `glGenerateMipmap`.
    pub fn generate_mipmap(&self, target: u32) {
        unsafe {
            trace!(target: "gl", "glGenerateMipmap{:?}", (target,));
            self.gl.GenerateMipmap(target);
        }
        self.check_error();
    }
    
    // Draw call operations

    /// Corresponds to `glDrawArrays`.
    pub fn draw_arrays(&self, mode: u32, offset: usize, count: usize) {
        unsafe {
            trace!(target: "gl", "glDrawArrays{:?}", (mode, offset, count));
            self.gl.DrawArrays(mode, offset as _, count as _);
        }
        self.check_error();
    }

    /// Corresponds to `glDrawElements`.
    pub fn draw_elements(&self, mode: u32, offset: usize, count: usize, ty: u32) {
        unsafe {
            trace!(target: "gl", "glDrawElements{:?}", (mode, count, ty, offset));
            self.gl.DrawElements(mode, count as _, ty, offset as *const _);
        }
        self.check_error();
    }

    /// Corresponds to `glUseProgram`.
    pub fn use_program(&self, id: u32) {
        unsafe {
            trace!(target: "gl", "glUseProgram{:?}", (id,));
            self.gl.UseProgram(id);
        }
        self.check_error();
    }

    /// Corresponds to `glBindBufferBase`.
    pub fn bind_buffer_base(&self, target: u32, binding: u32, id: u32) {
        unsafe {
            trace!(target: "gl", "glBindBufferBase{:?}", (target, binding, id));
            self.gl.BindBufferBase(target, binding, id);
        }
        self.check_error();
    }

    /// Corresponds to `glActiveTexture(GL_TEXTURE0 + index)`.
    pub fn active_texture(&self, index: u32) {
        unsafe {
            trace!(target: "gl", "glActiveTexture{:?}", (index,));
            self.gl.ActiveTexture(TEXTURE0 + index);
        }
        self.check_error();
    }
}
