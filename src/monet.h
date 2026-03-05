#ifndef HEYDE_MONET_H
#define HEYDE_MONET_H

#include <stdint.h>

struct heyde_colors {
    float accent[4];
    float background[4];
    float surface[4];
    float on_surface[4];
};

struct heyde_server;

void heyde_monet_init(struct heyde_server *server);
struct heyde_colors *heyde_monet_get_colors(struct heyde_server *server);

#endif
