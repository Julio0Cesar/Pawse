use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId, WindowLevel},
};
use windows::{
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
};

struct App {
    window: Option<Window>,
    cursor_position: (f64, f64),
    is_dragging: bool,
    drag_start: Option<(i32, i32)>, // Posição inicial do cursor na tela (coordenadas absolutas)
    window_start_pos: Option<(i32, i32)>, // Posição inicial da janela na tela
}

impl App {
    fn get_hwnd(window: &Window) -> Option<HWND> {
        #[cfg(target_os = "windows")]
        {
            use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
            let handle = window.window_handle().ok()?;
            match handle.as_raw() {
                RawWindowHandle::Win32(handle) => Some(HWND(handle.hwnd.get() as _)),
                _ => None,
            }
        }
        #[cfg(not(target_os = "windows"))]
        None
    }
    
    fn is_point_in_drag_area(&self, y: f64) -> bool {
        // Verificar se está nos 20 pixels superiores
        y >= 0.0 && y < 20.0
    }
    
    fn is_point_in_close_button(&self, x: f64, y: f64) -> bool {
        // Área do botão de fechar: width 275-295, height 5-25
        x >= 275.0 && x < 295.0 && y >= 5.0 && y < 25.0
    }
    
    fn start_drag(&mut self) {
        if let Some(window) = &self.window {
            #[cfg(target_os = "windows")]
            {
                if let Some(hwnd) = Self::get_hwnd(window) {
                    unsafe {
                        // Obter posição atual do cursor na tela (coordenadas absolutas)
                        let mut cursor_pos = POINT { x: 0, y: 0 };
                        let _ = GetCursorPos(&mut cursor_pos);
                        
                        // Obter posição atual da janela na tela
                        let mut window_rect = RECT::default();
                        let _ = GetWindowRect(hwnd, &mut window_rect);
                        
                        // Armazenar posições iniciais
                        self.drag_start = Some((cursor_pos.x, cursor_pos.y));
                        self.window_start_pos = Some((window_rect.left, window_rect.top));
                    }
                }
            }
        }
    }
    
    fn update_drag(&mut self) {
        if let (Some(window), Some(cursor_start), Some(window_start)) = 
            (&self.window, &self.drag_start, &self.window_start_pos) {
            #[cfg(target_os = "windows")]
            {
                if let Some(hwnd) = Self::get_hwnd(window) {
                    unsafe {
                        // Obter posição atual do cursor na tela (coordenadas absolutas)
                        let mut cursor_pos = POINT { x: 0, y: 0 };
                        let _ = GetCursorPos(&mut cursor_pos);
                        
                        // Calcular diferença de movimento em coordenadas da tela
                        let dx = cursor_pos.x - cursor_start.0;
                        let dy = cursor_pos.y - cursor_start.1;
                        
                        // Calcular nova posição da janela
                        let new_left = window_start.0 + dx;
                        let new_top = window_start.1 + dy;
                        
                        // Mover janela
                        let _ = SetWindowPos(
                            hwnd,
                            HWND_TOP,
                            new_left,
                            new_top,
                            0,
                            0,
                            SWP_NOSIZE | SWP_NOZORDER,
                        );
                    }
                }
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Criar janela sem decorações
        let window_attributes = Window::default_attributes()
            .with_inner_size(winit::dpi::LogicalSize::new(304, 372))
            .with_resizable(false)
            .with_decorations(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_position(winit::dpi::LogicalPosition::new(0, 0));
        
        let window = event_loop.create_window(window_attributes).unwrap();
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) {
                    event_loop.exit();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    match state {
                        winit::event::ElementState::Pressed => {
                            // Verificar se clicou no botão de fechar primeiro
                            if self.is_point_in_close_button(self.cursor_position.0, self.cursor_position.1) {
                                event_loop.exit();
                            }
                            // Verificar se clicou na área de arrastar (20px superiores)
                            else if self.is_point_in_drag_area(self.cursor_position.1) {
                                self.is_dragging = true;
                                self.start_drag();
                            }
                        }
                        winit::event::ElementState::Released => {
                            self.is_dragging = false;
                            self.drag_start = None;
                            self.window_start_pos = None;
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = (position.x, position.y);
                
                // Se estiver arrastando, atualizar posição da janela
                if self.is_dragging {
                    self.update_drag();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        window: None,
        cursor_position: (0.0, 0.0),
        is_dragging: false,
        drag_start: None,
        window_start_pos: None,
    };
    
    event_loop.run_app(&mut app).unwrap();
}
