all: build test
test:
	yarn build
	yarn start
build:
	cd v1; cargo contract build
	cd v2; cargo contract build
	cd proxy; cargo contract build
