attribute vec2 position;
attribute vec2 texcoord;
varying vec2 v_texcoord;
uniform mat4 projection;

void main() {
    v_texcoord = texcoord;
    gl_Position = projection * vec4(position, 0.0, 1.0);
}
