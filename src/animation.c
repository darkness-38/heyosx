#include <stdlib.h>
#include <math.h>
#include "animation.h"

#define EPSILON 0.001

void heyde_spring_tick(struct heyde_spring *spring, double dt) {
	// Semi-implicit Euler
	double force = (spring->target - spring->current) * spring->tension - (spring->velocity * spring->friction);
	spring->velocity += force * dt;
	spring->current += spring->velocity * dt;

	if (fabs(spring->target - spring->current) < EPSILON && fabs(spring->velocity) < EPSILON) {
		spring->current = spring->target;
		spring->velocity = 0;
	}
}

void heyde_animation_manager_init(struct heyde_animation_manager *manager) {
	wl_list_init(&manager->animations);
}

void heyde_animation_tick(struct heyde_animation_manager *manager, double dt) {
	struct heyde_animation *animation, *tmp;
	wl_list_for_each_safe(animation, tmp, &manager->animations, link) {
		heyde_spring_tick(&animation->spring, dt);

		if (animation->update) {
			animation->update(animation, animation->spring.current);
		}

		if (animation->spring.current == animation->spring.target && animation->spring.velocity == 0) {
			wl_list_remove(&animation->link);
			if (animation->on_complete) {
				animation->on_complete(animation);
			}
			free(animation);
		}
	}
}

void heyde_animation_add(struct heyde_animation_manager *manager, struct heyde_animation *animation) {
	wl_list_insert(&manager->animations, &animation->link);
}
