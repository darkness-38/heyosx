#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <GLES2/gl2.h>
#include <wlr/util/log.h>
#include <wlr/render/gles2.h>
#include <wlr/types/wlr_matrix.h>
#include "render.h"
#include "server.h"
#include "monet.h"

static char *load_file(const char *path) {
    FILE *f = fopen(path, "rb");
    if (!f) return NULL;
    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    fseek(f, 0, SEEK_SET);
    char *buf = malloc(len + 1);
    if (!buf) {
        fclose(f);
        return NULL;
    }
    size_t n = fread(buf, 1, len, f);
    buf[n] = '\0';
    fclose(f);
    return buf;
}

static GLuint compile_shader(GLenum type, const char *source) {
    GLuint shader = glCreateShader(type);
    glShaderSource(shader, 1, &source, NULL);
    glCompileShader(shader);

    GLint status;
    glGetShaderiv(shader, GL_COMPILE_STATUS, &status);
    if (!status) {
        char log[512];
        glGetShaderInfoLog(shader, 512, NULL, log);
        wlr_log(WLR_ERROR, "Shader compilation failed: %s", log);
        glDeleteShader(shader);
        return 0;
    }
    return shader;
}

static GLuint create_program(const char *vert_src, const char *frag_src) {
    GLuint vert = compile_shader(GL_VERTEX_SHADER, vert_src);
    GLuint frag = compile_shader(GL_FRAGMENT_SHADER, frag_src);
    if (!vert || !frag) {
        if (vert) glDeleteShader(vert);
        if (frag) glDeleteShader(frag);
        return 0;
    }

    GLuint prog = glCreateProgram();
    glAttachShader(prog, vert);
    glAttachShader(prog, frag);
    glLinkProgram(prog);

    GLint status;
    glGetProgramiv(prog, GL_LINK_STATUS, &status);
    if (!status) {
        char log[512];
        glGetProgramInfoLog(prog, 512, NULL, log);
        wlr_log(WLR_ERROR, "Program linking failed: %s", log);
        glDeleteProgram(prog);
        glDeleteShader(vert);
        glDeleteShader(frag);
        return 0;
    }

    glDeleteShader(vert);
    glDeleteShader(frag);
    return prog;
}

static void fb_init(struct blur_fb *fb, int width, int height) {
    if (fb->fb != 0 && fb->width == width && fb->height == height) return;
    if (fb->fb != 0) {
        glDeleteFramebuffers(1, &fb->fb);
        glDeleteTextures(1, &fb->tex);
    }
    fb->width = width;
    fb->height = height;
    glGenTextures(1, &fb->tex);
    glBindTexture(GL_TEXTURE_2D, fb->tex);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, width, height, 0, GL_RGBA, GL_UNSIGNED_BYTE, NULL);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);

    glGenFramebuffers(1, &fb->fb);
    glBindFramebuffer(GL_FRAMEBUFFER, fb->fb);
    glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, fb->tex, 0);
    glBindFramebuffer(GL_FRAMEBUFFER, 0);
}

void heyde_render_init(struct heyde_server *server) {
    if (!wlr_renderer_is_gles2(server->renderer)) {
        wlr_log(WLR_INFO, "Skipping GLES2 shader initialization (not using GLES2 renderer)");
        return;
    }

    char *vert_src = load_file("src/shaders/basic.vert");
    char *corner_frag_src = load_file("src/shaders/corner.frag");
    char *shadow_frag_src = load_file("src/shaders/shadow.frag");
    char *blur_frag_src = load_file("src/shaders/blur.frag");

    if (!vert_src || !corner_frag_src || !shadow_frag_src || !blur_frag_src) {
        wlr_log(WLR_ERROR, "Failed to load shaders from files");
        free(vert_src);
        free(corner_frag_src);
        free(shadow_frag_src);
        free(blur_frag_src);
        return;
    }

    server->render.corner_program = create_program(vert_src, corner_frag_src);
    server->render.corner_proj = glGetUniformLocation(server->render.corner_program, "projection");
    server->render.corner_tex = glGetUniformLocation(server->render.corner_program, "tex");
    server->render.corner_size = glGetUniformLocation(server->render.corner_program, "size");
    server->render.corner_radius = glGetUniformLocation(server->render.corner_program, "radius");

    server->render.shadow_program = create_program(vert_src, shadow_frag_src);
    server->render.shadow_proj = glGetUniformLocation(server->render.shadow_program, "projection");
    server->render.shadow_size = glGetUniformLocation(server->render.shadow_program, "size");
    server->render.shadow_radius = glGetUniformLocation(server->render.shadow_program, "radius");
    server->render.shadow_blur = glGetUniformLocation(server->render.shadow_program, "blur");
    server->render.shadow_color = glGetUniformLocation(server->render.shadow_program, "color");

    server->render.blur_program = create_program(vert_src, blur_frag_src);
    server->render.blur_proj = glGetUniformLocation(server->render.blur_program, "projection");
    server->render.blur_tex = glGetUniformLocation(server->render.blur_program, "tex");
    server->render.blur_tex_size = glGetUniformLocation(server->render.blur_program, "tex_size");
    server->render.blur_offset = glGetUniformLocation(server->render.blur_program, "offset");

    server->render.blur_passes = 5; // Default

    (void)fb_init;

    free(vert_src);
    free(corner_frag_src);
    free(shadow_frag_src);
    free(blur_frag_src);
}

struct heyde_render_data {
    struct heyde_server *server;
    struct wlr_output *wlr_output;
    struct wlr_render_pass *render_pass;
};

static void render_scene_buffer(struct wlr_scene_buffer *scene_buffer, int sx, int sy, void *data) {
    (void)sx; (void)sy;
    struct heyde_render_data *render_data = data;
    struct heyde_server *server = render_data->server;
    (void)server;

    struct wlr_scene_node *node = &scene_buffer->node;
    
    // Find the toplevel associated with this node if any
    struct wlr_scene_tree *tree = node->parent;
    while (tree) {
        if (tree->node.data) {
            // Found a toplevel
            break;
        }
        tree = tree->node.parent;
    }
}

void heyde_render_output(struct heyde_server *server, struct wlr_scene_output *scene_output, struct wlr_scene_output_state_options *options) {
    (void)server;
    (void)render_scene_buffer;
    wlr_scene_output_commit(scene_output, options);
}
