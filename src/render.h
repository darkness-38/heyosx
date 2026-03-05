#ifndef HEYDE_RENDER_H
#define HEYDE_RENDER_H

#include <wlr/types/wlr_scene.h>

struct heyde_server;

void heyde_render_init(struct heyde_server *server);
// Custom render callback for wlr_scene_output_commit
void heyde_render_output(struct heyde_server *server, struct wlr_scene_output *scene_output, struct wlr_scene_output_state_options *options);

#endif
