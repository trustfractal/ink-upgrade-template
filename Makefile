all:
	cd v1; cargo contract build
	cd v2; cargo contract build
	cd proxy; cargo contract build
