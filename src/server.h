#ifndef HEYDE_SERVER_H
#define HEYDE_SERVER_H

#include <wayland-server-core.h>
#include <wlr/backend.h>
#include <wlr/render/allocator.h>
#include <wlr/render/wlr_renderer.h>
#include <wlr/types/wlr_scene.h>
#include <wlr/types/wlr_output_layout.h>
#include <wlr/types/wlr_output.h>
#include <wlr/types/wlr_xdg_shell.h>
#include <wlr/types/wlr_layer_shell_v1.h>
#include <wlr/types/wlr_cursor.h>
#include <wlr/types/wlr_xcursor_manager.h>
#include <wlr/types/wlr_seat.h>
#include <wlr/types/wlr_keyboard.h>
#include <wlr/util/box.h>
#include <GLES2/gl2.h>
#include "animation.h"

#define HEYDE_NUM_WORKSPACES 9

enum heyde_cursor_mode {
    HEYDE_CURSOR_PASSTHROUGH,
    HEYDE_CURSOR_MOVE,
    HEYDE_CURSOR_RESIZE,
};

struct blur_fb {
    GLuint tex;
    GLuint fb;
    int width, height;
};

#define MAX_BLUR_PASSES 10

struct heyde_render_state {
    GLuint corner_program;
    GLint corner_proj;
    GLint corner_tex;
    GLint corner_size;
    GLint corner_radius;

    GLuint shadow_program;
    GLint shadow_proj;
    GLint shadow_size;
    GLint shadow_radius;
    GLint shadow_blur;
    GLint shadow_color;

    GLuint blur_program;
    GLint blur_proj;
    GLint blur_tex;
    GLint blur_tex_size;
    GLint blur_offset;

    struct blur_fb downsample_fbs[MAX_BLUR_PASSES];
    struct blur_fb upsample_fbs[MAX_BLUR_PASSES];
    int blur_passes;
};

struct heyde_workspace {
    struct heyde_server *server;
    int index;
    struct wlr_scene_tree *scene_tree;
};

struct heyde_server {
    struct wl_display *wl_display;
    struct wlr_backend *backend;
    struct wlr_renderer *renderer;
    struct wlr_allocator *allocator;
    struct wlr_scene *scene;
    struct wlr_scene_tree *layers[5]; // background, bottom, workspaces, top, overlay
    struct wlr_output_layout *output_layout;

    struct wl_list outputs;
    struct wl_listener new_output;

    struct wlr_xdg_shell *xdg_shell;
    struct wl_list toplevels;
    struct wl_listener new_xdg_surface;

    struct wlr_layer_shell_v1 *layer_shell;
    struct wl_list layer_surfaces;
    struct wl_listener new_layer_shell_surface;

    struct heyde_colors *monet_colors;
    int monet_palette_idx;
    struct wlr_scene_rect *background_rect;
    struct heyde_render_state render;
    struct heyde_animation_manager animation_mgr;

    struct heyde_workspace workspaces[HEYDE_NUM_WORKSPACES];
    int current_workspace;
    struct wlr_scene_tree *workspace_tree;
    double swipe_delta;

    struct wlr_cursor *cursor;
    struct wlr_xcursor_manager *cursor_mgr;
    struct wlr_seat *seat;
    struct wl_list keyboards;
    struct wl_listener new_input;
    struct wl_listener cursor_motion;
    struct wl_listener cursor_motion_absolute;
    struct wl_listener cursor_button;
    struct wl_listener cursor_axis;
    struct wl_listener cursor_frame;
    struct wl_listener request_set_cursor;
    struct wl_listener swipe_begin;
    struct wl_listener swipe_update;
    struct wl_listener swipe_end;

    enum heyde_cursor_mode cursor_mode;
    struct heyde_toplevel *grabbed_toplevel;
    double grab_x, grab_y;
    struct wlr_box grab_geobox;
    uint32_t resize_edges;
};

struct heyde_output {
    struct wl_list link;
    struct heyde_server *server;
    struct wlr_output *wlr_output;
    struct wl_listener frame;
    struct wl_listener request_state;
    struct wl_listener destroy;

    struct timespec last_frame;
};

struct heyde_keyboard {
    struct wl_list link;
    struct heyde_server *server;
    struct wlr_keyboard *wlr_keyboard;

    struct wl_listener modifiers;
    struct wl_listener key;
    struct wl_listener destroy;
};

struct heyde_toplevel {
    struct wl_list link;
    struct heyde_server *server;
    struct wlr_xdg_toplevel *xdg_toplevel;
    struct wlr_scene_tree *scene_tree;
    struct wl_listener map;
    struct wl_listener unmap;
    struct wl_listener destroy;
    struct wl_listener request_move;
    struct wl_listener request_resize;
    struct wl_listener request_maximize;
    struct wl_listener request_fullscreen;
};

struct heyde_layer_surface {
    struct wl_list link;
    struct heyde_server *server;
    struct wlr_layer_surface_v1 *wlr_layer_surface;
    struct wlr_scene_layer_surface_v1 *scene_layer_surface;

    struct wl_listener map;
    struct wl_listener unmap;
    struct wl_listener destroy;
    struct wl_listener surface_commit;
};

#endif
