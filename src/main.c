#include <assert.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <wayland-server-core.h>
#include <wlr/backend.h>
#include <wlr/render/allocator.h>
#include <wlr/render/wlr_renderer.h>
#include <wlr/types/wlr_scene.h>
#include <wlr/types/wlr_output_layout.h>
#include <wlr/types/wlr_output.h>
#include <wlr/types/wlr_xdg_shell.h>
#include <wlr/util/log.h>
#include <xkbcommon/xkbcommon.h>
#include <unistd.h>
#include "server.h"
#include "monet.h"
#include "render.h"
#include "workspace.h"

static int handle_signal(int signal, void *data) {
    struct heyde_server *server = data;
    wlr_log(WLR_INFO, "Caught signal %d, terminating compositor...", signal);
    wl_display_terminate(server->wl_display);
    return 0;
}

static bool handle_keybinding(struct heyde_server *server, uint32_t modifiers, xkb_keysym_t sym) {
    /*
     * Here we handle compositor keybindings. This is everything from
     * terminal spawn to closing windows.
     */
    if (!(modifiers & WLR_MODIFIER_LOGO)) {
        return false;
    }

    switch (sym) {
    case XKB_KEY_Return:
        if (fork() == 0) {
            execl("/bin/sh", "/bin/sh", "-c", "foot", NULL);
            exit(1);
        }
        return true;
    case XKB_KEY_space:
        if (fork() == 0) {
            execl("/bin/sh", "/bin/sh", "-c", "wofi --show drun", NULL);
            exit(1);
        }
        return true;
    case XKB_KEY_q:
        if (server->seat->keyboard_state.focused_surface) {
            struct wlr_xdg_surface *xdg_surface = wlr_xdg_surface_try_from_wlr_surface(
                server->seat->keyboard_state.focused_surface);
            if (xdg_surface && xdg_surface->role == WLR_XDG_SURFACE_ROLE_TOPLEVEL) {
                wlr_xdg_toplevel_send_close(xdg_surface->toplevel);
            }
        }
        return true;
    case XKB_KEY_E:
        if (modifiers & WLR_MODIFIER_SHIFT) {
            wl_display_terminate(server->wl_display);
            return true;
        }
        break;
    case XKB_KEY_1:
    case XKB_KEY_2:
    case XKB_KEY_3:
    case XKB_KEY_4:
    case XKB_KEY_5:
    case XKB_KEY_6:
    case XKB_KEY_7:
    case XKB_KEY_8:
    case XKB_KEY_9:
        heyde_workspace_activate(server, sym - XKB_KEY_1);
        return true;
    case XKB_KEY_m:
        heyde_monet_set_palette(server, (server->monet_palette_idx + 1) % 4);
        return true;
    }
    return false;
}

static void keyboard_handle_modifiers(struct wl_listener *listener, void *data) {
    (void)data;
    struct heyde_keyboard *keyboard = wl_container_of(listener, keyboard, modifiers);
    wlr_seat_set_keyboard(keyboard->server->seat, keyboard->wlr_keyboard);
    wlr_seat_keyboard_send_modifiers(keyboard->server->seat, &keyboard->wlr_keyboard->modifiers);
}

static void keyboard_handle_key(struct wl_listener *listener, void *data) {
    struct heyde_keyboard *keyboard = wl_container_of(listener, keyboard, key);
    struct wlr_keyboard_key_event *event = data;
    struct heyde_server *server = keyboard->server;

    uint32_t keycode = event->keycode + 8;
    const xkb_keysym_t *syms;
    int nsyms = xkb_state_key_get_syms(keyboard->wlr_keyboard->xkb_state, keycode, &syms);

    bool handled = false;
    uint32_t modifiers = wlr_keyboard_get_modifiers(keyboard->wlr_keyboard);
    if (event->state == WL_KEYBOARD_KEY_STATE_PRESSED) {
        for (int i = 0; i < nsyms; i++) {
            handled = handle_keybinding(server, modifiers, syms[i]);
            if (handled) break;
        }
    }

    if (!handled) {
        wlr_seat_set_keyboard(server->seat, keyboard->wlr_keyboard);
        wlr_seat_keyboard_send_key(server->seat, event->time_msec, event->keycode, event->state);
    }
}

static void keyboard_handle_destroy(struct wl_listener *listener, void *data) {
    (void)data;
    struct heyde_keyboard *keyboard = wl_container_of(listener, keyboard, destroy);
    wl_list_remove(&keyboard->modifiers.link);
    wl_list_remove(&keyboard->key.link);
    wl_list_remove(&keyboard->destroy.link);
    wl_list_remove(&keyboard->link);
    free(keyboard);
}

static void server_new_keyboard(struct heyde_server *server, struct wlr_input_device *device) {
    struct wlr_keyboard *wlr_keyboard = wlr_keyboard_from_input_device(device);
    struct heyde_keyboard *keyboard = calloc(1, sizeof(struct heyde_keyboard));
    keyboard->server = server;
    keyboard->wlr_keyboard = wlr_keyboard;

    struct xkb_context *context = xkb_context_new(XKB_CONTEXT_NO_FLAGS);
    struct xkb_keymap *keymap = xkb_keymap_new_from_names(context, NULL, XKB_KEYMAP_COMPILE_NO_FLAGS);
    wlr_keyboard_set_keymap(wlr_keyboard, keymap);
    xkb_keymap_unref(keymap);
    xkb_context_unref(context);
    wlr_keyboard_set_repeat_info(wlr_keyboard, 25, 600);

    keyboard->modifiers.notify = keyboard_handle_modifiers;
    wl_signal_add(&wlr_keyboard->events.modifiers, &keyboard->modifiers);
    keyboard->key.notify = keyboard_handle_key;
    wl_signal_add(&wlr_keyboard->events.key, &keyboard->key);
    keyboard->destroy.notify = keyboard_handle_destroy;
    wl_signal_add(&device->events.destroy, &keyboard->destroy);

    wlr_seat_set_keyboard(server->seat, keyboard->wlr_keyboard);
    wl_list_insert(&server->keyboards, &keyboard->link);
}

static void server_new_pointer(struct heyde_server *server, struct wlr_input_device *device) {
    wlr_cursor_attach_input_device(server->cursor, device);
}

static void server_new_input_handler(struct wl_listener *listener, void *data) {
    struct heyde_server *server = wl_container_of(listener, server, new_input);
    struct wlr_input_device *device = data;
    switch (device->type) {
    case WLR_INPUT_DEVICE_KEYBOARD:
        server_new_keyboard(server, device);
        break;
    case WLR_INPUT_DEVICE_POINTER:
        server_new_pointer(server, device);
        break;
    default:
        break;
    }
    uint32_t caps = WL_SEAT_CAPABILITY_POINTER;
    if (!wl_list_empty(&server->keyboards)) {
        caps |= WL_SEAT_CAPABILITY_KEYBOARD;
    }
    wlr_seat_set_capabilities(server->seat, caps);
}

static struct heyde_toplevel *desktop_toplevel_at(struct heyde_server *server, double lx, double ly,
        struct wlr_surface **surface, double *sx, double *sy) {
    /* This searches the surface at the given layout coordinates. */
    struct wlr_scene_node *node = wlr_scene_node_at(&server->scene->tree.node, lx, ly, sx, sy);
    if (node == NULL || node->type != WLR_SCENE_NODE_BUFFER) {
        return NULL;
    }

    struct wlr_scene_buffer *scene_buffer = wlr_scene_buffer_from_node(node);
    struct wlr_scene_surface *scene_surface = wlr_scene_surface_try_from_buffer(scene_buffer);
    if (!scene_surface) {
        return NULL;
    }

    *surface = scene_surface->surface;
    /* Find the node which is the root of this toplevel. */
    struct wlr_scene_tree *tree = node->parent;
    while (tree != NULL && tree->node.data == NULL) {
        tree = tree->node.parent;
    }
    return tree ? tree->node.data : NULL;
}

static void server_cursor_update_focus(struct heyde_server *server, uint32_t time) {
    double sx, sy;
    struct wlr_surface *surface = NULL;
    struct heyde_toplevel *toplevel = desktop_toplevel_at(server, server->cursor->x, server->cursor->y, &surface, &sx, &sy);
    if (!toplevel) {
        wlr_cursor_set_xcursor(server->cursor, server->cursor_mgr, "default");
    }
    if (surface) {
        wlr_seat_pointer_notify_enter(server->seat, surface, sx, sy);
        wlr_seat_pointer_notify_motion(server->seat, time, sx, sy);
    } else {
        wlr_seat_pointer_clear_focus(server->seat);
    }
}

static void process_cursor_move(struct heyde_server *server) {
    /* Move the grabbed toplevel to the new position. */
    struct heyde_toplevel *toplevel = server->grabbed_toplevel;
    
    int new_x = server->cursor->x - server->grab_x;
    int new_y = server->cursor->y - server->grab_y;

    // Snap to edges
    struct wlr_box layout_box;
    wlr_output_layout_get_box(server->output_layout, NULL, &layout_box);

    struct wlr_box geo_box;
    wlr_xdg_surface_get_geometry(toplevel->xdg_toplevel->base, &geo_box);

    int snap_threshold = 20;

    // Snap left
    if (abs(new_x + geo_box.x) < snap_threshold) {
        new_x = -geo_box.x;
    }
    // Snap right
    if (abs((new_x + geo_box.x + geo_box.width) - layout_box.width) < snap_threshold) {
        new_x = layout_box.width - geo_box.width - geo_box.x;
    }
    // Snap top
    if (abs(new_y + geo_box.y) < snap_threshold) {
        new_y = -geo_box.y;
    }
    // Snap bottom
    if (abs((new_y + geo_box.y + geo_box.height) - layout_box.height) < snap_threshold) {
        new_y = layout_box.height - geo_box.height - geo_box.y;
    }

    wlr_scene_node_set_position(&toplevel->scene_tree->node, new_x, new_y);
}

static void process_cursor_resize(struct heyde_server *server) {
    struct heyde_toplevel *toplevel = server->grabbed_toplevel;
    double border_x = server->cursor->x - server->grab_x;
    double border_y = server->cursor->y - server->grab_y;
    int new_left = server->grab_geobox.x;
    int new_right = server->grab_geobox.x + server->grab_geobox.width;
    int new_top = server->grab_geobox.y;
    int new_bottom = server->grab_geobox.y + server->grab_geobox.height;

    if (server->resize_edges & WLR_EDGE_TOP) {
        new_top = border_y;
        if (new_top >= new_bottom) {
            new_top = new_bottom - 1;
        }
    } else if (server->resize_edges & WLR_EDGE_BOTTOM) {
        new_bottom = border_y;
        if (new_bottom <= new_top) {
            new_bottom = new_top + 1;
        }
    }
    if (server->resize_edges & WLR_EDGE_LEFT) {
        new_left = border_x;
        if (new_left >= new_right) {
            new_left = new_right - 1;
        }
    } else if (server->resize_edges & WLR_EDGE_RIGHT) {
        new_right = border_x;
        if (new_right <= new_left) {
            new_right = new_left + 1;
        }
    }

    struct wlr_box geo_box;
    wlr_xdg_surface_get_geometry(toplevel->xdg_toplevel->base, &geo_box);
    wlr_scene_node_set_position(&toplevel->scene_tree->node,
        new_left - geo_box.x, new_top - geo_box.y);

    int width = new_right - new_left;
    int height = new_bottom - new_top;
    wlr_xdg_toplevel_set_size(toplevel->xdg_toplevel, width, height);
}

static void server_cursor_motion_handler(struct wl_listener *listener, void *data) {
    struct heyde_server *server = wl_container_of(listener, server, cursor_motion);
    struct wlr_pointer_motion_event *event = data;
    wlr_cursor_move(server->cursor, &event->pointer->base, event->delta_x, event->delta_y);

    if (server->cursor_mode == HEYDE_CURSOR_MOVE) {
        process_cursor_move(server);
    } else if (server->cursor_mode == HEYDE_CURSOR_RESIZE) {
        process_cursor_resize(server);
    } else {
        server_cursor_update_focus(server, event->time_msec);
    }
}

static void server_cursor_motion_absolute_handler(struct wl_listener *listener, void *data) {
    struct heyde_server *server = wl_container_of(listener, server, cursor_motion_absolute);
    struct wlr_pointer_motion_absolute_event *event = data;
    wlr_cursor_warp_absolute(server->cursor, &event->pointer->base, event->x, event->y);

    if (server->cursor_mode == HEYDE_CURSOR_MOVE) {
        process_cursor_move(server);
    } else if (server->cursor_mode == HEYDE_CURSOR_RESIZE) {
        process_cursor_resize(server);
    } else {
        server_cursor_update_focus(server, event->time_msec);
    }
}

static void server_cursor_button_handler(struct wl_listener *listener, void *data) {
    struct heyde_server *server = wl_container_of(listener, server, cursor_button);
    struct wlr_pointer_button_event *event = data;
    wlr_seat_pointer_notify_button(server->seat, event->time_msec, event->button, event->state);

    if (event->state == WL_POINTER_BUTTON_STATE_RELEASED) {
        server->cursor_mode = HEYDE_CURSOR_PASSTHROUGH;
    } else {
        double sx, sy;
        struct wlr_surface *surface = NULL;
        struct heyde_toplevel *toplevel = desktop_toplevel_at(server, server->cursor->x, server->cursor->y, &surface, &sx, &sy);
        if (toplevel != NULL) {
            wlr_scene_node_raise_to_top(&toplevel->scene_tree->node);
            wlr_seat_keyboard_notify_enter(server->seat, toplevel->xdg_toplevel->base->surface, 
                NULL, 0, NULL);
        } else {
            wlr_seat_keyboard_clear_focus(server->seat);
        }
    }
}

static void server_cursor_axis_handler(struct wl_listener *listener, void *data) {
    struct heyde_server *server = wl_container_of(listener, server, cursor_axis);
    struct wlr_pointer_axis_event *event = data;
    wlr_seat_pointer_notify_axis(server->seat, event->time_msec, event->orientation, event->delta, event->delta_discrete, event->source, event->relative_direction);
}

static void server_cursor_frame_handler(struct wl_listener *listener, void *data) {
    (void)data;
    struct heyde_server *server = wl_container_of(listener, server, cursor_frame);
    wlr_seat_pointer_notify_frame(server->seat);
}

static void server_request_set_cursor_handler(struct wl_listener *listener, void *data) {
    struct heyde_server *server = wl_container_of(listener, server, request_set_cursor);
    struct wlr_seat_pointer_request_set_cursor_event *event = data;
    struct wlr_seat_client *focused_client = server->seat->pointer_state.focused_client;
    if (focused_client == event->seat_client) {
        wlr_cursor_set_surface(server->cursor, event->surface, event->hotspot_x, event->hotspot_y);
    }
}

static void server_swipe_begin_handler(struct wl_listener *listener, void *data) {
    struct heyde_server *server = wl_container_of(listener, server, swipe_begin);
    struct wlr_pointer_swipe_begin_event *event = data;
    if (event->fingers == 3) {
        heyde_workspace_swipe_begin(server);
    }
}

static void server_swipe_update_handler(struct wl_listener *listener, void *data) {
    struct heyde_server *server = wl_container_of(listener, server, swipe_update);
    struct wlr_pointer_swipe_update_event *event = data;
    if (event->fingers == 3) {
        heyde_workspace_swipe(server, event->dx);
    }
}

static void server_swipe_end_handler(struct wl_listener *listener, void *data) {
    struct heyde_server *server = wl_container_of(listener, server, swipe_end);
    struct wlr_pointer_swipe_end_event *event = data;
    (void)event;
    // We don't check fingers here because libinput doesn't provide it on end
    // but heyde_workspace_swipe_end handles the transition based on accumulated delta
    heyde_workspace_swipe_end(server);
}

static void toplevel_animation_update(struct heyde_animation *animation, double current) {
    struct heyde_toplevel *toplevel = animation->user_data;
    struct wlr_box geo_box;
    wlr_xdg_surface_get_geometry(toplevel->xdg_toplevel->base, &geo_box);
    wlr_scene_node_set_position(&toplevel->scene_tree->node, toplevel->scene_tree->node.x, (int)current - geo_box.y);
}

static void toplevel_map_handler(struct wl_listener *listener, void *data) {
    (void)listener;
    (void)data;
    struct heyde_toplevel *toplevel = wl_container_of(listener, toplevel, map);
    
    const char *title = toplevel->xdg_toplevel->title;
    wlr_log(WLR_DEBUG, "Toplevel mapped: %s", title ? title : "(null)");

    wl_list_insert(&toplevel->server->toplevels, &toplevel->link);

    // Initial position for animation
    int target_y = toplevel->scene_tree->node.y;
    int start_y = target_y + 50; // Slide up from below
    
    struct heyde_animation *animation = calloc(1, sizeof(struct heyde_animation));
    animation->spring.current = start_y;
    animation->spring.target = target_y;
    animation->spring.velocity = 0;
    animation->spring.tension = 150;
    animation->spring.friction = 15;
    animation->update = toplevel_animation_update;
    animation->user_data = toplevel;
    
    heyde_animation_add(&toplevel->server->animation_mgr, animation);
}

static void toplevel_unmap_handler(struct wl_listener *listener, void *data) {
    (void)listener;
    (void)data;
    struct heyde_toplevel *toplevel = wl_container_of(listener, toplevel, unmap);
    wl_list_remove(&toplevel->link);
}

static void toplevel_destroy_handler(struct wl_listener *listener, void *data) {
    (void)listener;
    (void)data;
    struct heyde_toplevel *toplevel = wl_container_of(listener, toplevel, destroy);
    
    heyde_animation_cancel(&toplevel->server->animation_mgr, toplevel);

    wl_list_remove(&toplevel->map.link);
    wl_list_remove(&toplevel->unmap.link);
    wl_list_remove(&toplevel->destroy.link);
    wl_list_remove(&toplevel->request_move.link);
    wl_list_remove(&toplevel->request_resize.link);
    wl_list_remove(&toplevel->request_maximize.link);
    wl_list_remove(&toplevel->request_fullscreen.link);
    free(toplevel);
}

static void begin_interactive(struct heyde_toplevel *toplevel, enum heyde_cursor_mode mode, uint32_t edges) {
    struct heyde_server *server = toplevel->server;
    struct wlr_surface *focused_surface = server->seat->pointer_state.focused_surface;
    if (toplevel->xdg_toplevel->base->surface != focused_surface) {
        /* Deny move/resize requests from unfocused clients. */
        return;
    }

    server->grabbed_toplevel = toplevel;
    server->cursor_mode = mode;

    if (mode == HEYDE_CURSOR_MOVE) {
        server->grab_x = server->cursor->x - toplevel->scene_tree->node.x;
        server->grab_y = server->cursor->y - toplevel->scene_tree->node.y;
    } else {
        struct wlr_box geo_box;
        wlr_xdg_surface_get_geometry(toplevel->xdg_toplevel->base, &geo_box);

        double border_x = toplevel->scene_tree->node.x + geo_box.x;
        double border_y = toplevel->scene_tree->node.y + geo_box.y;
        server->grab_x = server->cursor->x - border_x;
        server->grab_y = server->cursor->y - border_y;

        server->grab_geobox = geo_box;
        server->grab_geobox.x += toplevel->scene_tree->node.x;
        server->grab_geobox.y += toplevel->scene_tree->node.y;

        server->resize_edges = edges;
    }
}

static void toplevel_request_move_handler(struct wl_listener *listener, void *data) {
    (void)data;
    struct heyde_toplevel *toplevel = wl_container_of(listener, toplevel, request_move);
    begin_interactive(toplevel, HEYDE_CURSOR_MOVE, 0);
}

static void toplevel_request_resize_handler(struct wl_listener *listener, void *data) {
    struct heyde_toplevel *toplevel = wl_container_of(listener, toplevel, request_resize);
    struct wlr_xdg_toplevel_resize_event *event = data;
    begin_interactive(toplevel, HEYDE_CURSOR_RESIZE, event->edges);
}

static void toplevel_request_maximize_handler(struct wl_listener *listener, void *data) {
    (void)listener;
    (void)data;
    /* This compositor does not support maximizing yet. */
}

static void toplevel_request_fullscreen_handler(struct wl_listener *listener, void *data) {
    (void)listener;
    (void)data;
    /* This compositor does not support fullscreen yet. */
}

static void layer_surface_map_handler(struct wl_listener *listener, void *data) {
    (void)listener;
    (void)data;
    struct heyde_layer_surface *layer_surface = wl_container_of(listener, layer_surface, map);
    wl_list_insert(&layer_surface->server->layer_surfaces, &layer_surface->link);
}

static void layer_surface_unmap_handler(struct wl_listener *listener, void *data) {
    (void)listener;
    (void)data;
    struct heyde_layer_surface *layer_surface = wl_container_of(listener, layer_surface, unmap);
    wl_list_remove(&layer_surface->link);
}

static void layer_surface_destroy_handler(struct wl_listener *listener, void *data) {
    (void)listener;
    (void)data;
    struct heyde_layer_surface *layer_surface = wl_container_of(listener, layer_surface, destroy);
    wl_list_remove(&layer_surface->map.link);
    wl_list_remove(&layer_surface->unmap.link);
    wl_list_remove(&layer_surface->destroy.link);
    wl_list_remove(&layer_surface->surface_commit.link);
    free(layer_surface);
}

static void layer_surface_surface_commit_handler(struct wl_listener *listener, void *data) {
    (void)data;
    struct heyde_layer_surface *layer_surface = wl_container_of(listener, layer_surface, surface_commit);
    struct wlr_layer_surface_v1 *wlr_layer_surface = layer_surface->wlr_layer_surface;

    if (wlr_layer_surface->initial_commit) {
        wlr_layer_surface_v1_configure(wlr_layer_surface, 0, 0);
    }
}

static void server_new_layer_shell_surface_handler(struct wl_listener *listener, void *data) {
    struct heyde_server *server = wl_container_of(listener, server, new_layer_shell_surface);
    struct wlr_layer_surface_v1 *wlr_layer_surface = data;

    struct heyde_layer_surface *layer_surface = calloc(1, sizeof(struct heyde_layer_surface));
    layer_surface->server = server;
    layer_surface->wlr_layer_surface = wlr_layer_surface;

    // Map wlroots layers to our scene graph layers
    int layer_idx;
    switch (wlr_layer_surface->pending.layer) {
    case ZWLR_LAYER_SHELL_V1_LAYER_BACKGROUND: layer_idx = 0; break;
    case ZWLR_LAYER_SHELL_V1_LAYER_BOTTOM:     layer_idx = 1; break;
    case ZWLR_LAYER_SHELL_V1_LAYER_TOP:        layer_idx = 3; break;
    case ZWLR_LAYER_SHELL_V1_LAYER_OVERLAY:    layer_idx = 4; break;
    default: layer_idx = 3; break;
    }

    struct wlr_scene_tree *layer_tree = server->layers[layer_idx];
    layer_surface->scene_layer_surface = wlr_scene_layer_surface_v1_create(layer_tree, wlr_layer_surface);

    layer_surface->map.notify = layer_surface_map_handler;
    wl_signal_add(&wlr_layer_surface->surface->events.map, &layer_surface->map);
    layer_surface->unmap.notify = layer_surface_unmap_handler;
    wl_signal_add(&wlr_layer_surface->surface->events.unmap, &layer_surface->unmap);
    layer_surface->destroy.notify = layer_surface_destroy_handler;
    wl_signal_add(&wlr_layer_surface->events.destroy, &layer_surface->destroy);
    layer_surface->surface_commit.notify = layer_surface_surface_commit_handler;
    wl_signal_add(&wlr_layer_surface->surface->events.commit, &layer_surface->surface_commit);
}

static void server_new_xdg_surface_handler(struct wl_listener *listener, void *data) {
    struct heyde_server *server = wl_container_of(listener, server, new_xdg_surface);
    struct wlr_xdg_surface *xdg_surface = data;

    if (xdg_surface->role != WLR_XDG_SURFACE_ROLE_TOPLEVEL) {
        return;
    }

    const char *title = xdg_surface->toplevel->title;
    wlr_log(WLR_DEBUG, "New XDG surface: %s", title ? title : "(null)");

    struct heyde_toplevel *toplevel = calloc(1, sizeof(struct heyde_toplevel));
    toplevel->server = server;
    toplevel->xdg_toplevel = xdg_surface->toplevel;
    
    struct heyde_workspace *ws = &server->workspaces[server->current_workspace];
    toplevel->scene_tree = wlr_scene_xdg_surface_create(ws->scene_tree, xdg_surface);
    toplevel->scene_tree->node.data = toplevel;
    xdg_surface->data = toplevel;

    toplevel->map.notify = toplevel_map_handler;
    wl_signal_add(&xdg_surface->surface->events.map, &toplevel->map);
    toplevel->unmap.notify = toplevel_unmap_handler;
    wl_signal_add(&xdg_surface->surface->events.unmap, &toplevel->unmap);
    toplevel->destroy.notify = toplevel_destroy_handler;
    wl_signal_add(&xdg_surface->events.destroy, &toplevel->destroy);

    toplevel->request_move.notify = toplevel_request_move_handler;
    wl_signal_add(&xdg_surface->toplevel->events.request_move, &toplevel->request_move);
    toplevel->request_resize.notify = toplevel_request_resize_handler;
    wl_signal_add(&xdg_surface->toplevel->events.request_resize, &toplevel->request_resize);
    toplevel->request_maximize.notify = toplevel_request_maximize_handler;
    wl_signal_add(&xdg_surface->toplevel->events.request_maximize, &toplevel->request_maximize);
    toplevel->request_fullscreen.notify = toplevel_request_fullscreen_handler;
    wl_signal_add(&xdg_surface->toplevel->events.request_fullscreen, &toplevel->request_fullscreen);

    // Center the window on the output layout
    struct wlr_box layout_box;
    wlr_output_layout_get_box(server->output_layout, NULL, &layout_box);
    
    struct wlr_box geo_box;
    wlr_xdg_surface_get_geometry(xdg_surface, &geo_box);

    int x = (layout_box.width - geo_box.width) / 2;
    int y = (layout_box.height - geo_box.height) / 2;

    // Adjust for workspace offset (relative to ws->scene_tree which is already offset)
    wlr_scene_node_set_position(&toplevel->scene_tree->node, x - geo_box.x, y - geo_box.y);
}

static void output_frame_handler(struct wl_listener *listener, void *data) {
    (void)data;
    struct heyde_output *output = wl_container_of(listener, output, frame);
    struct heyde_server *server = output->server;

    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);

    if (output->last_frame.tv_sec != 0) {
        double dt = (now.tv_sec - output->last_frame.tv_sec) +
                    (now.tv_nsec - output->last_frame.tv_nsec) / 1000000000.0;
        heyde_animation_tick(&server->animation_mgr, dt);
    }
    output->last_frame = now;

    struct wlr_scene_output *scene_output = wlr_scene_get_scene_output(server->scene, output->wlr_output);
    if (scene_output) {
        heyde_render_output(server, scene_output, NULL);
    }
    wlr_scene_output_send_frame_done(scene_output, &now);
}

static void output_request_state_handler(struct wl_listener *listener, void *data) {
    struct heyde_output *output = wl_container_of(listener, output, request_state);
    const struct wlr_output_event_request_state *event = data;
    wlr_output_commit_state(output->wlr_output, event->state);
}

static void output_destroy_handler(struct wl_listener *listener, void *data) {
    (void)data;
    struct heyde_output *output = wl_container_of(listener, output, destroy);
    wl_list_remove(&output->frame.link);
    wl_list_remove(&output->request_state.link);
    wl_list_remove(&output->destroy.link);
    wl_list_remove(&output->link);
    free(output);
}

static void server_new_output_handler(struct wl_listener *listener, void *data) {
    struct heyde_server *server = wl_container_of(listener, server, new_output);
    struct wlr_output *wlr_output = data;

    if (!wlr_output_init_render(wlr_output, server->allocator, server->renderer)) {
        return;
    }

    struct heyde_output *output = calloc(1, sizeof(struct heyde_output));
    output->wlr_output = wlr_output;
    output->server = server;

    output->frame.notify = output_frame_handler;
    wl_signal_add(&wlr_output->events.frame, &output->frame);

    output->request_state.notify = output_request_state_handler;
    wl_signal_add(&wlr_output->events.request_state, &output->request_state);

    output->destroy.notify = output_destroy_handler;
    wl_signal_add(&wlr_output->events.destroy, &output->destroy);

    wl_list_insert(&server->outputs, &output->link);

    wlr_output_layout_add_auto(server->output_layout, wlr_output);
    wlr_scene_output_create(server->scene, wlr_output);

    struct wlr_output_state state;
    wlr_output_state_init(&state);
    wlr_output_state_set_enabled(&state, true);
    
    struct wlr_output_mode *mode = wlr_output_preferred_mode(wlr_output);
    if (mode) {
        wlr_output_state_set_mode(&state, mode);
    }

    wlr_output_commit_state(wlr_output, &state);
    wlr_output_state_finish(&state);

    wlr_output_create_global(wlr_output, server->wl_display);
}

int main(int argc, char *argv[]) {
    (void)argc;
    (void)argv;
    wlr_log_init(WLR_DEBUG, NULL);

    // Ensure we can fallback to software rendering (LLVMpipe) if hardware fails.
    // We unset LIBGL_ALWAYS_SOFTWARE because it can cause EGL to fail if a hardware device 
    // is explicitly selected by wlroots but is partially broken (e.g. VMware without 3D).
    // Wlroots handles the software fallback internally more gracefully via WLR_RENDERER_ALLOW_SOFTWARE.
    setenv("WLR_RENDERER_ALLOW_SOFTWARE", "1", 1);
    unsetenv("LIBGL_ALWAYS_SOFTWARE");

    struct heyde_server server;
    wl_list_init(&server.outputs);
    wl_list_init(&server.toplevels);
    wl_list_init(&server.layer_surfaces);
    wl_list_init(&server.keyboards);
    server.cursor_mode = HEYDE_CURSOR_PASSTHROUGH;

    server.wl_display = wl_display_create();
    if (!server.wl_display) {
        wlr_log(WLR_ERROR, "failed to create wayland display");
        return 1;
    }

    struct wl_event_loop *event_loop = wl_display_get_event_loop(server.wl_display);
    server.backend = wlr_backend_autocreate(event_loop, NULL);
    if (!server.backend) {
        wlr_log(WLR_ERROR, "failed to create backend");
        return 1;
    }

    server.renderer = wlr_renderer_autocreate(server.backend);
    if (!server.renderer) {
        wlr_log(WLR_INFO, "failed to create hardware renderer, retrying with forced GLES2 software...");
        setenv("WLR_RENDERER", "gles2", 1);
        setenv("LIBGL_ALWAYS_SOFTWARE", "1", 1);
        server.renderer = wlr_renderer_autocreate(server.backend);
    }

    if (!server.renderer) {
        wlr_log(WLR_INFO, "failed to create GLES2 renderer, retrying with Pixman...");
        setenv("WLR_RENDERER", "pixman", 1);
        unsetenv("LIBGL_ALWAYS_SOFTWARE");
        server.renderer = wlr_renderer_autocreate(server.backend);
    }

    if (!server.renderer) {
        wlr_log(WLR_ERROR, "failed to create renderer (tried hardware, software GLES2, and Pixman)");
        return 1;
    }

    wlr_renderer_init_wl_display(server.renderer, server.wl_display);

    server.allocator = wlr_allocator_autocreate(server.backend, server.renderer);
    if (!server.allocator) {
        wlr_log(WLR_ERROR, "failed to create allocator");
        return 1;
    }

    server.scene = wlr_scene_create();
    if (!server.scene) {
        wlr_log(WLR_ERROR, "failed to create scene");
        return 1;
    }

    // Create scene graph layers (z-order: back to front)
    server.layers[0] = wlr_scene_tree_create(&server.scene->tree); // background
    server.layers[1] = wlr_scene_tree_create(&server.scene->tree); // bottom
    server.layers[2] = wlr_scene_tree_create(&server.scene->tree); // workspaces
    server.layers[3] = wlr_scene_tree_create(&server.scene->tree); // top
    server.layers[4] = wlr_scene_tree_create(&server.scene->tree); // overlay

    server.output_layout = wlr_output_layout_create(server.wl_display);
    if (!server.output_layout) {
        wlr_log(WLR_ERROR, "failed to create output layout");
        return 1;
    }

    server.cursor = wlr_cursor_create();
    wlr_cursor_attach_output_layout(server.cursor, server.output_layout);

    server.cursor_motion.notify = server_cursor_motion_handler;
    wl_signal_add(&server.cursor->events.motion, &server.cursor_motion);
    server.cursor_motion_absolute.notify = server_cursor_motion_absolute_handler;
    wl_signal_add(&server.cursor->events.motion_absolute, &server.cursor_motion_absolute);
    server.cursor_button.notify = server_cursor_button_handler;
    wl_signal_add(&server.cursor->events.button, &server.cursor_button);
    server.cursor_axis.notify = server_cursor_axis_handler;
    wl_signal_add(&server.cursor->events.axis, &server.cursor_axis);
    server.cursor_frame.notify = server_cursor_frame_handler;
    wl_signal_add(&server.cursor->events.frame, &server.cursor_frame);

    server.swipe_begin.notify = server_swipe_begin_handler;
    wl_signal_add(&server.cursor->events.swipe_begin, &server.swipe_begin);
    server.swipe_update.notify = server_swipe_update_handler;
    wl_signal_add(&server.cursor->events.swipe_update, &server.swipe_update);
    server.swipe_end.notify = server_swipe_end_handler;
    wl_signal_add(&server.cursor->events.swipe_end, &server.swipe_end);

    server.cursor_mgr = wlr_xcursor_manager_create(NULL, 24);

    server.new_input.notify = server_new_input_handler;
    wl_signal_add(&server.backend->events.new_input, &server.new_input);

    server.seat = wlr_seat_create(server.wl_display, "seat0");
    server.request_set_cursor.notify = server_request_set_cursor_handler;
    wl_signal_add(&server.seat->events.request_set_cursor, &server.request_set_cursor);

    heyde_monet_init(&server);
    heyde_render_init(&server);
    heyde_animation_manager_init(&server.animation_mgr);
    heyde_workspaces_init(&server);

    struct wlr_box layout_box;
    wlr_output_layout_get_box(server.output_layout, NULL, &layout_box);
    server.background_rect = wlr_scene_rect_create(server.layers[0],
        layout_box.width, layout_box.height, server.monet_colors->background);

    server.new_output.notify = server_new_output_handler;
    wl_signal_add(&server.backend->events.new_output, &server.new_output);

    server.xdg_shell = wlr_xdg_shell_create(server.wl_display, 3);
    server.new_xdg_surface.notify = server_new_xdg_surface_handler;
    wl_signal_add(&server.xdg_shell->events.new_surface, &server.new_xdg_surface);

    server.layer_shell = wlr_layer_shell_v1_create(server.wl_display, 4);
    server.new_layer_shell_surface.notify = server_new_layer_shell_surface_handler;
    wl_signal_add(&server.layer_shell->events.new_surface, &server.new_layer_shell_surface);

    struct wl_event_source *sigint_source = wl_event_loop_add_signal(event_loop, SIGINT, handle_signal, &server);
    struct wl_event_source *sigterm_source = wl_event_loop_add_signal(event_loop, SIGTERM, handle_signal, &server);

    if (!wlr_backend_start(server.backend)) {
        wlr_log(WLR_ERROR, "failed to start backend");
        return 1;
    }

    wlr_log(WLR_INFO, "Starting heyDE");
    wl_display_run(server.wl_display);

    wl_event_source_remove(sigint_source);
    wl_event_source_remove(sigterm_source);

    wl_display_destroy_clients(server.wl_display);
    wlr_xcursor_manager_destroy(server.cursor_mgr);
    wlr_cursor_destroy(server.cursor);
    wl_display_destroy(server.wl_display);

    wlr_output_layout_destroy(server.output_layout);

    return 0;
}
