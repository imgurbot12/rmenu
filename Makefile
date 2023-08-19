# RMenu Installation/Deployment Configuration

CARGO=cargo
FLAGS=--release

DEST=$(HOME)/.config/rmenu
INSTALL=$(CARGO_PATH)/bin
SWAY_CONF=/etc/sway/config.d

all: install sway

sway:
	echo "Installing Configuration for Sway"
	sudo cp -vf ./rmenu/public/99-rmenu-sway.conf ${SWAY_CONF}/.

install: build deploy

deploy:
	mkdir -p ${DEST}
	cp -vf ./target/release/rmenu ${INSTALL}/rmenu
	cp -vf ./target/release/rmenu-build ${INSTALL}/rmenu-build
	cp -vf ./target/release/desktop ${DEST}/rmenu-desktop
	cp -vf ./target/release/run ${DEST}/rmenu-run
	cp -vf ./target/release/audio ${DEST}/rmenu-audio
	cp -vf ./target/release/network ${DEST}/rmenu-network
	cp -vf ./target/release/window ${DEST}/rmenu-window
	cp -vf ./rmenu/public/config.yaml ${DEST}/config.yaml

build: build-rmenu build-plugins

build-rmenu:
	${CARGO} build -p rmenu ${FLAGS}
	${CARGO} build -p rmenu-plugin --bin rmenu-build ${FLAGS}

build-plugins:
	${CARGO} build -p run ${FLAGS}
	${CARGO} build -p desktop ${FLAGS}
	${CARGO} build -p audio ${FLAGS}
	${CARGO} build -p network ${FLAGS}
	${CARGO} build -p window ${FLAGS}
