all: prompt test

%: %.rs
	rustc -C link-args=-dead_strip -Z lto --opt-level=3 $^ -o $@

test:
	./prompt
