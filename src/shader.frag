// `shader.frag`
// A fragment shader is also part of the rendering pipeline and it manipulates pixels, can do things like interpolation

#version 460

// `in`, `out`, and `inout` substitute c pointers and refs
// `in` passed into a function means that the value of the param is being read
// `out` passed into a function means that the value of the param is being set
// `inout` passed into a function means that the value of the param is being read and set
// `layout` specifies where `f_color` will be saved to 
// `location=0` is current texture of the swapchain, which is the screen
layout(location=0) in vec3 v_color;
layout(location=0) out vec4 f_color;

void main() {
  f_color = vec4(v_color, 1.0);
  // f_color = vec4(0.1, 0.2, 0.3, 1.0);
}