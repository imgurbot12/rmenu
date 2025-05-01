# RMenu Installation/Deployment Configuration

CARGO=cargo
FLAGS=--release

DEST=$(HOME)/.config/rmenu
INSTALL=$(CARGO_PATH)/bin
SWAY_CONF=/etc/sway/config.d

all: install sway

#: deploy sway configuration to sway config folder
sway:
	echo "Installing Configuration for Sway"
	sudo cp -f ./rmenu/public/99-rmenu-sway.conf ${SWAY_CONF}/.

#: clean remaining build artifcats
clean:
	rm -rf $(PWD)/rmenu.zip /tmp/rmenu-build
	${CARGO} clean

#: build and locally deploy rmenu
install: build deploy

#: build rmenu components and zip into final artifact
package: DEST=/tmp/rmenu-build/config
package: INSTALL=/tmp/rmenu-build/bin
package: build deploy
	cd /tmp/rmenu-build && zip -r $(PWD)/rmenu.zip .
	rm -rf /tmp/rmenu-build

#: locally deploy build-artifcats into their designated locations
deploy:
	mkdir -p ${DEST}/plugins ${INSTALL}
	cp -fr themes ${DEST}/.
	cp -fr plugins/misc/* ${DEST}/plugins/.
	cp -fr plugins/emoji/css/* ${DEST}/plugins/css/.
	cp -f ./target/release/rmenu ${INSTALL}/rmenu
	cp -f ./target/release/rmenu-build ${INSTALL}/rmenu-build
	cp -f ./target/release/desktop ${DEST}/plugins/rmenu-desktop
	cp -f ./target/release/emoji   ${DEST}/plugins/rmenu-emoji
	cp -f ./target/release/files   ${DEST}/plugins/rmenu-files
	cp -f ./target/release/network ${DEST}/plugins/rmenu-network
	cp -f ./target/release/run ${DEST}/plugins/rmenu-run
	cp -f ./target/release/search ${DEST}/plugins/rmenu-search
	cp -f ./target/release/window ${DEST}/plugins/rmenu-window
	cp -f ./rmenu/public/config.yaml ${DEST}/config.yaml
	ln -sf  ${DEST}/themes/dark.css ${DEST}/style.css

#: build rmenu and its various plugins
build: build-rmenu build-plugins

#: build rmenu and rmenu-build binaries
build-rmenu:
	${CARGO} build -p rmenu ${FLAGS}
	${CARGO} build -p rmenu-plugin --bin rmenu-build ${FLAGS}

#: build rmenu plugin binaries
build-plugins:
	${CARGO} build -p desktop ${FLAGS}
	${CARGO} build -p emoji ${FLAGS}
	${CARGO} build -p files ${FLAGS}
	${CARGO} build -p network ${FLAGS}
	${CARGO} build -p run ${FLAGS}
	${CARGO} build -p search ${FLAGS}
	${CARGO} build -p window ${FLAGS}
