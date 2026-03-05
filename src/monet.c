#include <stdlib.h>
#include <string.h>
#include "monet.h"
#include "server.h"

struct palette {
    float accent[4];
    float background[4];
    float surface[4];
    float on_surface[4];
};

static const struct palette PALETTES[] = {
    { // 0: Deep Blue/Purple (Original end-4)
        .accent = {0.0f, 0.82f, 1.0f, 1.0f},
        .background = {0.04f, 0.04f, 0.06f, 1.0f},
        .surface = {0.12f, 0.13f, 0.16f, 0.8f},
        .on_surface = {0.88f, 0.89f, 0.91f, 1.0f}
    },
    { // 1: Nature Green
        .accent = {0.4f, 0.9f, 0.4f, 1.0f},
        .background = {0.04f, 0.06f, 0.04f, 1.0f},
        .surface = {0.12f, 0.16f, 0.12f, 0.8f},
        .on_surface = {0.9f, 0.95f, 0.9f, 1.0f}
    },
    { // 2: Sunset Red/Orange
        .accent = {1.0f, 0.4f, 0.2f, 1.0f},
        .background = {0.06f, 0.04f, 0.04f, 1.0f},
        .surface = {0.16f, 0.12f, 0.12f, 0.8f},
        .on_surface = {0.95f, 0.9f, 0.9f, 1.0f}
    },
    { // 3: Monochrome / Silver
        .accent = {0.8f, 0.8f, 0.8f, 1.0f},
        .background = {0.05f, 0.05f, 0.05f, 1.0f},
        .surface = {0.15f, 0.15f, 0.15f, 0.8f},
        .on_surface = {1.0f, 1.0f, 1.0f, 1.0f}
    }
};

#define NUM_PALETTES (sizeof(PALETTES) / sizeof(PALETTES[0]))

void heyde_monet_init(struct heyde_server *server) {
    server->monet_colors = calloc(1, sizeof(struct heyde_colors));
    server->monet_palette_idx = 0;
    heyde_monet_set_palette(server, 0);
}

void heyde_monet_set_palette(struct heyde_server *server, int index) {
    if (index < 0 || index >= (int)NUM_PALETTES) return;
    
    server->monet_palette_idx = index;
    memcpy(server->monet_colors->accent, PALETTES[index].accent, sizeof(float) * 4);
    memcpy(server->monet_colors->background, PALETTES[index].background, sizeof(float) * 4);
    memcpy(server->monet_colors->surface, PALETTES[index].surface, sizeof(float) * 4);
    memcpy(server->monet_colors->on_surface, PALETTES[index].on_surface, sizeof(float) * 4);

    if (server->background_rect) {
        wlr_scene_rect_set_color(server->background_rect, server->monet_colors->background);
    }
}

struct heyde_colors *heyde_monet_get_colors(struct heyde_server *server) {
    return server->monet_colors;
}
