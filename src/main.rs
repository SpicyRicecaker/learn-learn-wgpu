// Winit allows us to make windows
use futures::executor::block_on;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn main() {
    // An event loop creates everything that's needed to make a new window
    // For example, on linux it creates X11 or wayland connection, can be different for other OS
    let event_loop = EventLoop::new();
    // Creates a new window, taking in a reference to the event_loop
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Connect our wgpu state
    // `block_on()` is basically scuffed `await`, since main can't be `async`
    let mut state = block_on(State::new(&window));

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
    // Open connection to the GPU, responsible for creating rendering and compute processes in the form of commands, which are submitted to the queue
    device: wgpu::Device,
    // Executes the command buffer, provides methods for writing to buffers and textures
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
        let surface = unsafe { instance.create_surface(window) };
        // The adapter is a handle to the actual GPU
        // We request it with certain "hard" (mandatory) and "soft" (priority) query options
        let adapter = instance
            .request_adapter(
                // This is creating a type (struct is a category of type) from the wgpu library
                &wgpu::RequestAdapterOptions {
                    // Power preference default
                    power_preference: wgpu::PowerPreference::Default,
                    // Make sure that the GPU can actually display stuff on the surface that we made using the wgpu instance ealier
                    compatible_surface: Some(&surface),
                },
            )
            .await
            .unwrap();
        // The adapter that we made earlier is now being used to create the `device` and `queue`
        // `adapter.request_device()` opens an actual connection to the GPU, returning the `queue` and `device`
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    // `Features` field on `DeviceDescriptor` describes the features that we want
                    // The features themselves can be device-specific and thus not cross-platform, care
                    features: wgpu::Features::empty(),
                    // Limits the type of resources that can be created??
                    limits: wgpu::Limits::default(),
                    // Temporary field apparently
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap();

        // A swap chain descriptor describes the characteristics of a swap chain
        let sc_desc = wgpu::SwapChainDescriptor {
            // How will the swap chain be used? (Only option is OUTPUT_ATTACHMENT), which outputs texture to the screen
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            // How the textures are formatted in the swap chain
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            // Width and height of the swap chain, which must match the width and height of the surface, (in this case a `window`)
            width: size.width,
            height: size.height,
            // The mode that the swap chain will be presented in
            // Uses the enum `PresentMode`, which has options of `FIFO` (vsync), `Immediate` (vsync-off) and `Mailbox` (hybrid)
            present_mode: wgpu::PresentMode::Fifo,
        };
        // Represents the image or series of images that will be drawn onto a `Surface`
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        // We can return the struct that can be built using all of our variables
        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
        }
    }

    // To allow window resizing, we need to recreate the swap chain with the new size
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // Update size with new size
        self.size = new_size;
        // Then update size of window in the swap chain descriptor
        self.sc_desc.width = self.size.width;
        self.sc_desc.height = self.size.height;
        // Then create a new swap chain based on the updated swap chain descriptor size
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
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
