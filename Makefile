all: prompt test

%: %.rs
	rustc --opt-level=3 $^ -o $@

test:
	./prompt