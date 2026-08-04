EXECUTABLE_NAME := markdown2html-converter

all: ./target/x86_64-unknown-linux-musl/release/$(EXECUTABLE_NAME)

SOURCE_FILES := $(shell find . -type f \( -iname '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) | sed 's/ /\\ /g')
RESOURCE_PATHS := $(shell find ./resources \( -type f -o -type d \) | sed 's/ /\\ /g')
./target/x86_64-unknown-linux-musl/release/$(EXECUTABLE_NAME): $(SOURCE_FILES) $(RESOURCE_PATHS)
	cargo build --release --target x86_64-unknown-linux-musl
	
install:
	$(MAKE)
	sudo cp ./target/x86_64-unknown-linux-musl/release/$(EXECUTABLE_NAME) /usr/local/bin/$(EXECUTABLE_NAME)
	sudo chown root: /usr/local/bin/$(EXECUTABLE_NAME)
	sudo chmod 0755 /usr/local/bin/$(EXECUTABLE_NAME)

uninstall:
	sudo rm /usr/local/bin/$(EXECUTABLE_NAME)

test:
	cargo test --verbose

clean:
	cargo clean
