precision mediump float;
varying vec2 v_texcoord;
uniform sampler2D tex;
uniform vec2 tex_size;
uniform float offset;

void main() {
    vec2 half_pixel = 1.0 / tex_size;
    vec4 color = texture2D(tex, v_texcoord + vec2(-offset, -offset) * half_pixel);
    color += texture2D(tex, v_texcoord + vec2(offset, -offset) * half_pixel);
    color += texture2D(tex, v_texcoord + vec2(-offset, offset) * half_pixel);
    color += texture2D(tex, v_texcoord + vec2(offset, offset) * half_pixel);
    gl_FragColor = color * 0.25;
}
