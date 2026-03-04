CC = gcc
CFLAGS = -Wall -Wextra -Werror -g -DWLR_USE_UNSTABLE -I. `pkg-config --cflags wlroots-0.18 wayland-server xkbcommon glesv2`
LIBS = `pkg-config --libs wlroots-0.18 wayland-server xkbcommon glesv2`

WAYLAND_PROTOCOLS = $(shell pkg-config --variable=pkgdatadir wayland-protocols)
WAYLAND_SCANNER = $(shell pkg-config --variable=wayland_scanner wayland-scanner)

XDG_SHELL_XML = $(WAYLAND_PROTOCOLS)/stable/xdg-shell/xdg-shell.xml
LAYER_SHELL_XML = /usr/share/wlr-protocols/unstable/wlr-layer-shell-unstable-v1.xml

SRC = src/main.c src/monet.c src/render.c src/animation.c xdg-shell-protocol.c wlr-layer-shell-unstable-v1-protocol.c
OBJ = $(SRC:.c=.o)
TARGET = heyde

all: $(TARGET)

xdg-shell-protocol.h: $(XDG_SHELL_XML)
	$(WAYLAND_SCANNER) server-header $< $@

xdg-shell-protocol.c: $(XDG_SHELL_XML)
	$(WAYLAND_SCANNER) private-code $< $@

wlr-layer-shell-unstable-v1-protocol.h: $(LAYER_SHELL_XML)
	$(WAYLAND_SCANNER) server-header $< $@

wlr-layer-shell-unstable-v1-protocol.c: $(LAYER_SHELL_XML)
	$(WAYLAND_SCANNER) private-code $< $@

src/main.o: src/main.c xdg-shell-protocol.h wlr-layer-shell-unstable-v1-protocol.h

$(TARGET): $(OBJ)
	$(CC) $(CFLAGS) -o $@ $^ $(LIBS)

%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f $(OBJ) $(TARGET) xdg-shell-protocol.h xdg-shell-protocol.c wlr-layer-shell-unstable-v1-protocol.h wlr-layer-shell-unstable-v1-protocol.c

.PHONY: all clean
