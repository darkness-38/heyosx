#ifndef HEYDE_ANIMATION_H
#define HEYDE_ANIMATION_H

#include <wayland-server-core.h>

struct heyde_spring {
	double current;
	double target;
	double velocity;
	double tension;
	double friction;
};

struct heyde_animation {
	struct wl_list link;
	struct heyde_spring spring;
	void (*update)(struct heyde_animation *animation, double current);
	void (*on_complete)(struct heyde_animation *animation);
	void *user_data;
};

struct heyde_animation_manager {
	struct wl_list animations;
};

void heyde_spring_tick(struct heyde_spring *spring, double dt);
void heyde_animation_manager_init(struct heyde_animation_manager *manager);
void heyde_animation_tick(struct heyde_animation_manager *manager, double dt);
void heyde_animation_add(struct heyde_animation_manager *manager, struct heyde_animation *animation);
void heyde_animation_cancel(struct heyde_animation_manager *manager, void *user_data);

#endif
