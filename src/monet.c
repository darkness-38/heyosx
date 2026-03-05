#include <stdlib.h>
#include <string.h>
#include "monet.h"
#include "server.h"

void heyde_monet_init(struct heyde_server *server) {
    server->monet_colors = calloc(1, sizeof(struct heyde_colors));
    
    // Default "end-4" palette (Vibrant Deep Blue/Purple theme)
    // RGBA (0.0 - 1.0)
    
    // Accent: Vibrant Cyan/Blue (#00D2FF)
    server->monet_colors->accent[0] = 0.0f;
    server->monet_colors->accent[1] = 0.82f;
    server->monet_colors->accent[2] = 1.0f;
    server->monet_colors->accent[3] = 1.0f;
    
    // Background: Deep Dark Blue/Black (#0A0B10)
    server->monet_colors->background[0] = 0.04f;
    server->monet_colors->background[1] = 0.04f;
    server->monet_colors->background[2] = 0.06f;
    server->monet_colors->background[3] = 1.0f;
    
    // Surface: Semi-transparent Slate (#1E202A)
    server->monet_colors->surface[0] = 0.12f;
    server->monet_colors->surface[1] = 0.13f;
    server->monet_colors->surface[2] = 0.16f;
    server->monet_colors->surface[3] = 0.8f;
    
    // On Surface: Off-white (#E1E2E7)
    server->monet_colors->on_surface[0] = 0.88f;
    server->monet_colors->on_surface[1] = 0.89f;
    server->monet_colors->on_surface[2] = 0.91f;
    server->monet_colors->on_surface[3] = 1.0f;
}

struct heyde_colors *heyde_monet_get_colors(struct heyde_server *server) {
    return server->monet_colors;
}
