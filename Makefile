PHONY: test clean

compiler-book-rs: ./target/debug/compiler-book-rs

./target/debug/compiler-book-rs: ./src/*.rs
	cargo build

test: compiler-book-rs
	./test.sh

clean:
	cargo clean
	rm tmp*
