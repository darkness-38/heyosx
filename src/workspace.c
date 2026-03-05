#include <stdlib.h>
#include <wlr/util/log.h>
#include <wlr/types/wlr_output_layout.h>
#include "workspace.h"
#include "server.h"
#include "animation.h"

static void workspace_animation_update(struct heyde_animation *animation, double current) {
    struct heyde_server *server = animation->user_data;
    wlr_scene_node_set_position(&server->workspace_tree->node, (int)current, 0);
}

void heyde_workspaces_init(struct heyde_server *server) {
    server->workspace_tree = wlr_scene_tree_create(server->layers[ZWLR_LAYER_SHELL_V1_LAYER_TOP]);
    server->current_workspace = 0;

    struct wlr_box layout_box;
    wlr_output_layout_get_box(server->output_layout, NULL, &layout_box);
    int width = layout_box.width;
    if (width == 0) width = 1920; // Fallback

    for (int i = 0; i < HEYDE_NUM_WORKSPACES; i++) {
        server->workspaces[i].server = server;
        server->workspaces[i].index = i;
        server->workspaces[i].scene_tree = wlr_scene_tree_create(server->workspace_tree);
        wlr_scene_node_set_position(&server->workspaces[i].scene_tree->node, i * width, 0);
    }
}

void heyde_workspace_activate(struct heyde_server *server, int index) {
    if (index < 0 || index >= HEYDE_NUM_WORKSPACES) return;
    if (index == server->current_workspace) return;

    struct wlr_box layout_box;
    wlr_output_layout_get_box(server->output_layout, NULL, &layout_box);
    int width = layout_box.width;

    server->current_workspace = index;

    heyde_animation_cancel(&server->animation_mgr, server);

    struct heyde_animation *animation = calloc(1, sizeof(struct heyde_animation));
    animation->spring.current = server->workspace_tree->node.x;
    animation->spring.target = -index * width;
    animation->spring.velocity = 0;
    animation->spring.tension = 180;
    animation->spring.friction = 20;
    animation->update = workspace_animation_update;
    animation->user_data = server;

    heyde_animation_add(&server->animation_mgr, animation);
}

void heyde_workspace_swipe_begin(struct heyde_server *server) {
    heyde_animation_cancel(&server->animation_mgr, server);
    server->swipe_delta = 0;
}

void heyde_workspace_swipe(struct heyde_server *server, double delta_x) {
    server->swipe_delta += delta_x;
    
    struct wlr_box layout_box;
    wlr_output_layout_get_box(server->output_layout, NULL, &layout_box);
    int width = layout_box.width;

    double current_x = -server->current_workspace * width + server->swipe_delta;
    wlr_scene_node_set_position(&server->workspace_tree->node, (int)current_x, 0);
}

void heyde_workspace_swipe_end(struct heyde_server *server) {
    struct wlr_box layout_box;
    wlr_output_layout_get_box(server->output_layout, NULL, &layout_box);
    int width = layout_box.width;

    int new_workspace = server->current_workspace;
    // Swiping fingers LEFT (negative delta) moves to next workspace
    if (server->swipe_delta < -width / 4.0 && server->current_workspace < HEYDE_NUM_WORKSPACES - 1) {
        new_workspace++;
    } 
    // Swiping fingers RIGHT (positive delta) moves to previous workspace
    else if (server->swipe_delta > width / 4.0 && server->current_workspace > 0) {
        new_workspace--;
    }

    server->current_workspace = new_workspace;
    server->swipe_delta = 0;

    struct heyde_animation *animation = calloc(1, sizeof(struct heyde_animation));
    animation->spring.current = server->workspace_tree->node.x;
    animation->spring.target = -new_workspace * width;
    animation->spring.velocity = 0;
    animation->spring.tension = 180;
    animation->spring.friction = 20;
    animation->update = workspace_animation_update;
    animation->user_data = server;

    heyde_animation_add(&server->animation_mgr, animation);
}
