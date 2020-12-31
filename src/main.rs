// Winit allows us to make windows
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    // An event loop creates everything that's needed to make a new window
    // For example, on linux it creates X11 or wayland connection, can be different for other OS
    let event_loop = EventLoop::new();
    // Creates a new window, taking in a reference to the event_loop
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // TODO Don't know what the fk clojures are RIP
    event_loop.run(move |event, _, control_flow| {
        // Can't tell but maybe it's game loop stuff
        *control_flow = ControlFlow::Wait;

        // Listen to window close event to exit if window close is pressed?
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}

struct State {
    // Platform specific "surface" in which rendered images can be put
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        
        // The first thing to create for wgpu is an instance
        // BackendBit::Primary => the backend with primary tier of support, like vulkan, dx12, etc.
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        // Primary use of an instance is to create `Surfaces` and `Adapters`
        // This creates a surface from a "raw window handle", provided by winit
        let surface = unsafe {instance.create_surface(window)};
        // The adapter is a handle to the actual GPU
        // We request it with certain "hard" (mandatory) and "soft" (priority) query options
        let adapter = instance.request_adapter(
            // This is creating a type (struct is a category of type) from the wgpu library
            &wgpu::RequestAdapterOptions {
                // Power preference default
                power_preference: wgpu::PowerPreference::Default,
                // Make sure that the GPU can actually display stuff on the surface that we made using the wgpu instance ealier
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();
        
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None,
        ).await.unwrap();
        todo!();
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        todo!()
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        todo!()
    }
    fn update(&mut self) {
        todo!()
    }
    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        todo!()
    }
}
