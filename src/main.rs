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

    let mut batch = Batch::new();

    // TODO Don't know what the fk clojures are RIP
    event_loop.run(move |event, _, control_flow| {
        // Listen to window close event to exit if window close is pressed?
        match event {
            Event::RedrawRequested(_) => {
                state.update();
                match state.render(&batch) {
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
                            // Update batch
                            batch.cursor_position.0 = position.x;
                            batch.cursor_position.1 = position.y;
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
                                    virtual_keycode,
                                    ..
                                } => {
                                    match virtual_keycode {
                                        // Exit program if escape is pressed
                                        Some(VirtualKeyCode::Escape) => {
                                            *control_flow = ControlFlow::Exit
                                        }
                                        // If spacebar is pressed update batch
                                        Some(VirtualKeyCode::Space) => {
                                            batch.space_pressed = !batch.space_pressed
                                        }
                                        _ => (),
                                    }
                                }
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

struct Batch {
    // 0 or 1
    space_pressed: bool,
    // Cursor position
    cursor_position: (f64, f64),
}

impl Batch {
    fn new() -> Self {
        Batch {
            space_pressed: false,
            cursor_position: (0.0, 0.0),
        }
    }
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
    render_pipeline: wgpu::RenderPipeline,
    render_pipeline2: wgpu::RenderPipeline,
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

        // `wgpu::include_spirv!` differs from `wgpu::util::make_spirv` in that it takes in file name vs. `&str`
        // So we can directly include our `.spv` files
        let vs_module = device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
        let vs2_module = device.create_shader_module(wgpu::include_spirv!("shader2.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));
        let fs2_module = device.create_shader_module(wgpu::include_spirv!("shader2.frag.spv"));

        // Pipeline layout describes a pipeline
        let render_pipeline_layout =
            // `PipelineLayoutDescriptor` can be used to create a pipeline layout
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        // Render pipline descriptor describes a render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            // Label shows up in debuggin
            label: Some("Render Pipeline"),
            // TODO describes **bindings** for layout???
            layout: Some(&render_pipeline_layout),
            // `ProgrammableStageDescriptor` describes the stage of a rendering pipeline
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                // Shader module is a compiled shader module on the gpu that defines the rendering stage
                // In this case we're inputting the vertex shader
                module: &vs_module,
                entry_point: "main",
            },
            // Fragment shader technically optional, so surrounded with `Some`
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                // In this case we're inputting the fragment shader
                module: &fs_module,
                entry_point: "main",
            }),
            // Rasterization process for the pipeline
            // Describes how to process primitives before they are sent to the fragment shader
            rasterization_state: Some(
                // `RasterizationStateDescriptor` describes the state of rasterizer in render pipeline
                // A rasterizer (aster for star, starshaped) basically turns a vector into pixels
                wgpu::RasterizationStateDescriptor {
                    // Counter clockwise or clockwise, depending on the coordinate system?
                    // Vertices with counterclockwise order are considered the front face, used for right handed coordinate systems
                    front_face: wgpu::FrontFace::Ccw,
                    // Primitives that don't meet the criteria are culled, which is good because it speeds up rendering process for images that arent't seen anyway
                    cull_mode: wgpu::CullMode::Back,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                    clamp_depth: false,
                },
            ),
            // Describes how colors are stored and processed throughout the render pipeline
            color_states: &[wgpu::ColorStateDescriptor {
                // We put it to `swap_chain` format so it's easy to copy to it
                format: sc_desc.format,
                // Just replace previous pixels
                color_blend: wgpu::BlendDescriptor::REPLACE,
                // Apparently very complicated
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                // Enable writes to all color channels, rgba
                write_mask: wgpu::ColorWrite::ALL,
            }],
            // Tell `wgpu` that we want to use a list of triangles for drawing
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,

            // *** Apparently this entire section is basically buffers so...
            depth_stencil_state: None,
            //
            vertex_state: wgpu::VertexStateDescriptor {
                // Format of the index buffer to `u16`
                index_format: wgpu::IndexFormat::Uint16,
                //
                vertex_buffers: &[],
            },
            // Anti aliasing stuff
            // Samples calculated per pixel, this is MSAA, 1 is no multisampling
            sample_count: 1,
            // Use all samples in `sample_count` above
            sample_mask: !0,
            // Anti-aliasing
            alpha_to_coverage_enabled: false,
        });

        // Render pipline descriptor describes a render pipeline
        let render_pipeline2 = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            // Label shows up in debuggin
            label: Some("Render Pipeline"),
            // TODO describes **bindings** for layout???
            layout: Some(&render_pipeline_layout),
            // `ProgrammableStageDescriptor` describes the stage of a rendering pipeline
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                // Shader module is a compiled shader module on the gpu that defines the rendering stage
                // In this case we're inputting the vertex shader
                module: &vs2_module,
                entry_point: "main",
            },
            // Fragment shader technically optional, so surrounded with `Some`
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                // In this case we're inputting the fragment shader
                module: &fs2_module,
                entry_point: "main",
            }),
            // Rasterization process for the pipeline
            // Describes how to process primitives before they are sent to the fragment shader
            rasterization_state: Some(
                // `RasterizationStateDescriptor` describes the state of rasterizer in render pipeline
                // A rasterizer (aster for star, starshaped) basically turns a vector into pixels
                wgpu::RasterizationStateDescriptor {
                    // Counter clockwise or clockwise, depending on the coordinate system?
                    // Vertices with counterclockwise order are considered the front face, used for right handed coordinate systems
                    front_face: wgpu::FrontFace::Ccw,
                    // Primitives that don't meet the criteria are culled, which is good because it speeds up rendering process for images that arent't seen anyway
                    cull_mode: wgpu::CullMode::Back,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                    clamp_depth: false,
                },
            ),
            // Describes how colors are stored and processed throughout the render pipeline
            color_states: &[wgpu::ColorStateDescriptor {
                // We put it to `swap_chain` format so it's easy to copy to it
                format: sc_desc.format,
                // Just replace previous pixels
                color_blend: wgpu::BlendDescriptor::REPLACE,
                // Apparently very complicated
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                // Enable writes to all color channels, rgba
                write_mask: wgpu::ColorWrite::ALL,
            }],
            // Tell `wgpu` that we want to use a list of triangles for drawing
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,

            // *** Apparently this entire section is basically buffers so...
            depth_stencil_state: None,
            //
            vertex_state: wgpu::VertexStateDescriptor {
                // Format of the index buffer to `u16`
                index_format: wgpu::IndexFormat::Uint16,
                //
                vertex_buffers: &[],
            },
            // Anti aliasing stuff
            // Samples calculated per pixel, this is MSAA, 1 is no multisampling
            sample_count: 1,
            // Use all samples in `sample_count` above
            sample_mask: !0,
            // Anti-aliasing
            alpha_to_coverage_enabled: false,
        });


        // We can return the struct that can be built using all of our variables
        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            // TROLL
            render_pipeline2
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
    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }
    // Nothing to update / "tick" for now
    fn update(&mut self) {}
    // Basically wgpu
    fn render(&mut self, batch: &Batch) -> Result<(), wgpu::SwapChainError> {
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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                            // Rgb based off of batch TROLL
                            r: batch.cursor_position.0 % 4.0,
                            g: batch.cursor_position.1 % 4.0,
                            b: (batch.cursor_position.0 + batch.cursor_position.1) % 4.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                // I think that depth has to do with the z-index of pixels, and stenciling can block pixels?
                // Maybe important for 3D but useless for 2D
                depth_stencil_attachment: None,
            });

            // Set render pipeline to the pipeline that we defined in `state`
            if batch.space_pressed {
                render_pass.set_pipeline(&self.render_pipeline2);
            } else {
                render_pass.set_pipeline(&self.render_pipeline);
            }
            // Draw triangle????
            render_pass.draw(0..4, 0..2);
        }
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
