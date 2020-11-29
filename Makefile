CROSS_VERSION := $(shell cross --version | grep cross | awk '{print $$2}')
TARGET_PI0 = arm-unknown-linux-gnueabihf
TARGET_PI3 = armv7-unknown-linux-gnueabihf

build-cross-builder:
	docker build \
	  -t beatble-cross-builder:$(TARGET) \
	  --build-arg TARGET=$(TARGET) \
	  --build-arg VERSION=$(CROSS_VERSION) \
	  --build-arg ARCH=armhf \
	  ./build

build-pi0-builder:
	$(MAKE) TARGET=$(TARGET_PI0) build-cross-builder

build-pi0:
	cross build --target $(TARGET_PI0)

build-pi3:
	cross build --target $(TARGET_PI3)
