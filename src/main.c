#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <wayland-server-core.h>
#include <wlr/backend.h>
#include <wlr/render/allocator.h>
#include <wlr/render/wlr_renderer.h>
#include <wlr/types/wlr_scene.h>
#include <wlr/types/wlr_output_layout.h>
#include <wlr/util/log.h>
#include "server.h"

int main(int argc, char *argv[]) {
    wlr_log_init(WLR_DEBUG, NULL);

    struct heyde_server server;

    server.wl_display = wl_display_create();
    if (!server.wl_display) {
        wlr_log(WLR_ERROR, "failed to create wayland display");
        return 1;
    }

    server.backend = wlr_backend_autocreate(server.wl_display, NULL);
    if (!server.backend) {
        wlr_log(WLR_ERROR, "failed to create backend");
        return 1;
    }

    server.renderer = wlr_renderer_autocreate(server.backend);
    if (!server.renderer) {
        wlr_log(WLR_ERROR, "failed to create renderer");
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

    server.output_layout = wlr_output_layout_create();
    if (!server.output_layout) {
        wlr_log(WLR_ERROR, "failed to create output layout");
        return 1;
    }

    if (!wlr_backend_start(server.backend)) {
        wlr_log(WLR_ERROR, "failed to start backend");
        return 1;
    }

    wlr_log(WLR_INFO, "Starting heyDE");
    wl_display_run(server.wl_display);

    wl_display_destroy(server.wl_display);
    return 0;
}
