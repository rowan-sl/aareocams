OUTDIR=build
OPT_PROFILE=speed


# out directory configuration

out_dir:
	[ -d $(OUTDIR) ] || mkdir -p $(OUTDIR)
	[ -d $(OUTDIR)/debug ] || mkdir -p $(OUTDIR)/debug
	[ -d $(OUTDIR)/release ] || mkdir -p $(OUTDIR)/release

clean_out_dir:
	rm -f $(OUTDIR)/debug/aareocams-dash
	rm -f $(OUTDIR)/debug/aareocams-bot
	rm -f $(OUTDIR)/release/aareocams-dash
	rm -f $(OUTDIR)/release/aareocams-bot

# build systems

debug: out_dir clean_out_dir
	cargo build
	cargo build --target armv7-unknown-linux-gnueabihf
	mv target/debug/aareocams-dash $(OUTDIR)/debug
	mv target/armv7-unknown-linux-gnueabihf/debug/aareocams-bot $(OUTDIR)/debug

release: out_dir clean_out_dir
	cargo build --profile $(OPT_PROFILE)
	cargo build --profile $(OPT_PROFILE) --target armv7-unknown-linux-gnueabihf
	mv target/$(OPT_PROFILE)/aareocams-dash $(OUTDIR)/release
	mv target/armv7-unknown-linux-gnueabihf/$(OPT_PROFILE)/aareocams-bot $(OUTDIR)/release

# run

run-dash: release
	./$(OUTDIR)/release/aareocams-dash

# utilities

all: debug release

all-clean: clean all

clean: clean_out_dir
	cargo clean
