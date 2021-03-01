all:
	cd v1; cargo contract build
	cd proxy; cargo contract build
