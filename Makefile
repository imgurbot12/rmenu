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
	cp -vf ./target/release/desktop ${DEST}/drun
	cp -vf ./target/release/run ${DEST}/run
	cp -vf ./rmenu/public/config.yaml ${DEST}/config.yaml
	cp -vf ./rmenu/public/default.css ${DEST}/style.css

build: build-rmenu build-plugins

build-rmenu:
	${CARGO} build -p rmenu ${FLAGS}

build-plugins:
	${CARGO} build -p run ${FLAGS}
	${CARGO} build -p desktop ${FLAGS}
