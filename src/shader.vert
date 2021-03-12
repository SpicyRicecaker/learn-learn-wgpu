// `shader.vert`
// The vertex shader is part of the rendering pipeline and it manipulates a list of vertices, the "early state" of rendering

// Actually, just refer to https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Shader_modules

// The version of GLSL, latest can be found on https://en.wikipedia.org/wiki/OpenGL_Shading_Language#cite_note-13
// The vast majority of features are available from `version 130`, so it's the default version
// Older hardware might not support latest version
#version 460

// Many data types
// `float` can be represented by `vec2`, `vec3`, or `vec4`
// `int` can be represented by `ivec2`, `ivec3`, or `ivec4`
// There's also a matrix form for both, `mat2`, `mat3`, and `mat4`
// `sampler2D` and the like, are textures
// there's also the `bool` datatype

// Storing vertex data in the shader as positions is very ineffective, especially with big data sets, learn more abt `Buffer`s later

// Declaration is type, variable name = type, values
// const vec2 positions[3] = vec2[3] (
//   vec2(0.0, 0.5),
//   vec2(-0.5, -0.5),
//   vec2(0.5, -0.5)
// );

// Get position data from vertex buffer
// Position
layout(location=0) in vec3 a_position;
// Color
layout(location=1) in vec3 a_color;

// Literally just passthrough color to the shader.frag
layout(location=0) out vec3 v_color;


void main () {
  // `gl_Position` and `gl_VertexIndex` are "built-in" variables (existing beforehand) that say where the data is going to be
  // `gl_Position` means sending out the data
  // Takes in the 4 floats, including the posiition of where the vertex is being stored
  // type constructors are very flexible, you can have a 4D matrix built from a 2D + 1D + 1D, or only specify one value and have it take up the whole thing

  // Swizzling:
  // You can access arrays using coordinate dimension...
  // `array.x`, `array.y`, `array.z`, `array.w`
  // array style access...
  // `array[0]`, `array[1]`, etc.
  // color channel
  // .r .g .b .a
  // or texture dimension
  // .s .t .p .q
  // You can even call it like `array.rgb`

  // Return color for frag shader to deal with
  v_color = a_color;
  
  // The way that the data types are setup makes it easy to do matrix operations on them 
  // gl_Position = vec4(a_position, 1.0);
  // if(gl_VertexIndex == 2) {
  gl_Position = vec4(a_position, 1.0);
  // } else {
    // gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
  // }
  // if(a_position.r == 0.1) 
  // {
  // }
}

// If statements can be very inefficient, as gpu cores are synced so they have to wait for this to evaluate