.DEFAULT_GOAL := build

BIN_FILE=my-arbolitos

build:
	cargo build --release

run:
	./target/release/"${BIN_FILE}" -h
