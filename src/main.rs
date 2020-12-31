// Winit allows us to make windows
use futures::executor::block_on;
use winit::{
    // Import all event types
    event::*,
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

    let mut data = (0.1, 0.2, 0.3);

    // TODO Don't know what the fk clojures are RIP
    event_loop.run(move |event, _, control_flow| {
        // Listen to window close event to exit if window close is pressed?
        match event {
            Event::RedrawRequested(_) => {
                state.update();
                match state.render(data) {
                    Ok(_) => {}
                    // Recreate swap chain if lost
                    // TODO how does the `SwapChain` even get "Lost"?
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // If system ran out of memory, only option is to exit program
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // Otherwise print error
                    Err(e) => {
                        println!("SwapChainError: {:?}", e)
                    }
                }
            }
            Event::MainEventsCleared => {
                // We must keep requesting redraws, else `RedrawRequested` event will only trigger once
                window.request_redraw();
            }
            // In the case that a window event occurs, in which the window_id matches our current window id
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                // Have the `state.input()` take precendence over the main loop
                if !state.input(event) {
                    match event {
                        // In the case that the event is a close request,
                        // Set the loop behavior (control flow) to exit
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        // In the case that we have an input event
                        // `KeyboardInput` is a struct, so we have to use the struct matching syntax (remember `..` is syntax for autofill)
                        WindowEvent::CursorMoved { position, .. } => {
                            data.0 = position.x % 2.0;
                            data.1 = position.y % 2.0;
                        }
                        WindowEvent::KeyboardInput { input, .. } => {
                            // Match the attributes of the keypress

                            // Exit the program if the escape key is pressed
                            match input {
                        // Virtual keycode vs. scancode, use virtual when the semantic of the key is more important than the physical location of the key
                        KeyboardInput {
                            // `Element State` is pressed vs release
                            state: ElementState::Pressed,
                            // The virtual keycode of the keypress
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                                // Exit program
                        } => *control_flow = ControlFlow::Exit,
                        _ => (),
                    }
                        }
                        // On resize event, call our implemented state function and pass in the new size
                        WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                        // Scale factor changed could be changing display res, moving to new monitor, etc.
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => (),
                    }
                }
            }
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
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        // Then create a new swap chain based on the updated swap chain descriptor size
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }
    // Checks if an event is fully complete, returns bool, if true, main won't process it any longer
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }
    // Nothing to update / "tick" for now
    fn update(&mut self) {}
    // Basically wgpu
    fn render(&mut self, data: (f64, f64, f64)) -> Result<(), wgpu::SwapChainError> {
        // We need to get a frame to render to at first
        // Includes `wgpu::Texture` and `wgpu::Textureview` that holds the image that is being drawn
        // Remember the `?` operator here means return `Some(thing)` or return `Error`
        let frame = self.swap_chain.get_current_frame()?.output;
        // Recall that the `device` is responsible for creating commands to be sent to the `queue` of the GPU
        // `Encoder` builds a buffer that can be sent to GPU
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        // The outside `{}` means that the reference to the mutable borrow of `encoder` in `encoder.begin_render_pass()` is dropped after so we're allowed to call `encoder.finish()` after
        // You can also use `drop(render_pass)`
        {
            // Create a render pass using the encoder
            // `RenderPassDescriptor` only has two fields, `color_attachments` and `depth_stencil_attachment`
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                // Describe where the color is going to be drawn to
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    // Informs the texture to which the colors are going to be saved to
                    // Specifying the `frame.view` that we made earlier means we're drawing to the screen
                    attachment: &frame.view,
                    // The texture that will receive the resolved output, same as `attachment` unless multisampling (MSAA) is enabled
                    resolve_target: None,
                    // What to do with colors on the screen?
                    ops: wgpu::Operations {
                        // How to handle colors stored from the previous frame
                        // Currently we're just clearing the colors
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: data.0,
                            g: data.1,
                            b: data.2,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                // I think that depth has to do with the z-index of pixels, and stenciling can block pixels?
                // Maybe important for 3D but useless for 2D
                depth_stencil_attachment: None,
            });
        }
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
