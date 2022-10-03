DIST_DIR:=${CURDIR}/dist

.PHONY: build
build:
	rm -rf ${DIST_DIR}
	mkdir ${DIST_DIR}
	cargo build --release
	mv ${CURDIR}/target/release/rusty_proxy ${DIST_DIR}

.PHONY: build-dev
build-dev:
	cargo build

.PHONY: run
run:
	cargo run

.PHONY: test
test:
	cargo test

.PHONY: clean
clean:
	cargo clean
	rm -rf ${DIST_DIR}
