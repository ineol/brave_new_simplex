CARGO = cargo
 
CARGO_OPTS =
 
all:
	$(MAKE) build
 
opt:
	$(MAKE) all

print:
	$(MAKE) all
 
build:
	$(CARGO) $(CARGO_OPTS) build --release
 
clean:
	$(CARGO) $(CARGO_OPTS) clean
 
test:
	$(CARGO) $(CARGO_OPTS) test
 
bench:
	$(CARGO) $(CARGO_OPTS) bench
 
doc:
	$(CARGO) $(CARGO_OPTS) doc
 
.PHONY: all build clean check test bench doc