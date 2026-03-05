#ifndef HEYDE_WORKSPACE_H
#define HEYDE_WORKSPACE_H

#include <wlr/types/wlr_scene.h>
#include "server.h"

#define HEYDE_NUM_WORKSPACES 9

struct heyde_workspace {
    struct heyde_server *server;
    int index;
    struct wlr_scene_tree *scene_tree;
};

void heyde_workspaces_init(struct heyde_server *server);
void heyde_workspace_activate(struct heyde_server *server, int index);
void heyde_workspace_swipe(struct heyde_server *server, double delta_x);
void heyde_workspace_swipe_begin(struct heyde_server *server);
void heyde_workspace_swipe_end(struct heyde_server *server);

#endif
