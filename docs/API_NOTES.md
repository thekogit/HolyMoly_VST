# API notes (generated in S02)
NIH-plug source: C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101
## create_egui_editor

> C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:37:pub fn create_egui_editor<T, B, U>(
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:38:    egui_state: Arc<EguiState>,
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:39:    user_state: T,
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:40:    build: B,
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:41:    update: U,
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:42:) -> Option<Box<dyn Editor>>
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:43:where
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:44:    T: 'static + Send + Sync,
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:45:    B: Fn(&Context, &mut T) + 'static + Send + Sync,
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:46:    U: Fn(&Context, &ParamSetter, &mut T) + 'static + Send + Sync,
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:47:{
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:48:    Some(Box::new(editor::EguiEditor {
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:49:        egui_state,
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:50:        user_state: Arc::new(RwLock::new(user_state)),
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:51:        build: Arc::new(build),
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:52:        update: Arc::new(update),
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:53:
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:54:        // TODO: We can't get the size of the window when baseview does its own scaling, so if the
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:55:        //       host does not set a scale factor on Windows or Linux we should just use a factor of
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:56:        //       1. That may make the GUI tiny but it also prevents it from getting cut off.
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:57:        #[cfg(target_os = "macos")]



## EguiState::from_size

> C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:96:    pub fn from_size(width: u32, height: u32) -> Arc<EguiState> {
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:97:        Arc::new(EguiState {
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:98:            size: AtomicCell::new((width, height)),
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:99:            requested_size: Default::default(),
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\nih_plug_egui\src\lib.rs:100:            open: AtomicBool::new(false),



## ParamSetter (file: C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs)

> C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:140:    pub fn begin_set_parameter<P: Param>(&self, param: &P) {
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:141:        unsafe { self.raw_context.raw_begin_set_parameter(param.as_ptr()) };
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:142:    }
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:143:
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:144:    /// Set a parameter to the specified parameter value. You will need to call
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:145:    /// [`begin_set_parameter()`][Self::begin_set_parameter()] before and
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:146:    /// [`end_set_parameter()`][Self::end_set_parameter()] after calling this so the host can
> C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:153:    pub fn set_parameter<P: Param>(&self, param: &P, value: P::Plain) {
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:154:        let ptr = param.as_ptr();
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:155:        let normalized = param.preview_normalized(value);
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:156:        unsafe {
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:157:            self.raw_context
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:158:                .raw_set_parameter_normalized(ptr, normalized)
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:159:        };
> C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:169:    pub fn set_parameter_normalized<P: Param>(&self, param: &P, normalized: f32) {
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:170:        let ptr = param.as_ptr();
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:171:        unsafe {
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:172:            self.raw_context
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:173:                .raw_set_parameter_normalized(ptr, normalized)
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:174:        };
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:175:    }
> C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:180:    pub fn end_set_parameter<P: Param>(&self, param: &P) {
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:181:        unsafe { self.raw_context.raw_end_set_parameter(param.as_ptr()) };
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:182:    }
  C:\Users\szpon\.cargo\git\checkouts\nih-plug-d4210f82e920295f\de42101\src\context\gui.rs:183:}



