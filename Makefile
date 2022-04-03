OUTDIR=build
OPT_PROFILE=speed
CARGO_CMD=cargo


out_dir:
	[ -d $(OUTDIR) ] || mkdir -p $(OUTDIR)
	[ -d $(OUTDIR)/debug ] || mkdir -p $(OUTDIR)/debug
	[ -d $(OUTDIR)/release ] || mkdir -p $(OUTDIR)/release

clean_out_dir_dbg:
	rm -f $(OUTDIR)/debug/aareocams-dash
	rm -f $(OUTDIR)/debug/aareocams-bot

clean_out_dir_rel:
	rm -f $(OUTDIR)/release/aareocams-dash
	rm -f $(OUTDIR)/release/aareocams-bot

debug: out_dir clean_out_dir_dbg
	$(CARGO_CMD) build
	$(CARGO_CMD) build --target armv7-unknown-linux-gnueabihf
	mv target/debug/aareocams-dash $(OUTDIR)/debug
	mv target/armv7-unknown-linux-gnueabihf/debug/aareocams-bot $(OUTDIR)/debug

release: out_dir clean_out_dir_rel
	$(CARGO_CMD) build --profile $(OPT_PROFILE)
	$(CARGO_CMD) build --profile $(OPT_PROFILE) --target armv7-unknown-linux-gnueabihf
	mv target/$(OPT_PROFILE)/aareocams-dash $(OUTDIR)/release
	mv target/armv7-unknown-linux-gnueabihf/$(OPT_PROFILE)/aareocams-bot $(OUTDIR)/release

deploy_r: release
	export AAREOCAMS_DEPLOY_BUILD_MODE=release && ./tools/deploy.sh

deploy_d: debug
	export AAREOCAMS_DEPLOY_BUILD_MODE=debug && ./tools/deploy.sh

clean:
	$(CARGO_CMD) clean
	rm -rf --preserve-root=all $(OUTDIR)
