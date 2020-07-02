PAGES := ..\rktrl-pages
HASH := $(shell git rev-parse HEAD)
GIT := git -C $(PAGES)

$(PAGES)\rktrl_lib.js: $(wildcard src/**/*.rs) $(wildcard src/*.rs)
	wasm-pack build --target web --no-typescript --out-dir ..\rktrl-pages

$(PAGES)\rktrl_lib_bg.wasm: $(PAGES)\rktrl_lib.js

$(PAGES)\index.html: wasm_wrapper.html
	copy $< $@

.PHONY: web
web: $(PAGES)\rktrl_lib.js $(PAGES)\rktrl_lib_bg.wasm $(PAGES)\index.html

.PHONY: web-push
web-push: web
	$(GIT) add -A .
	$(GIT) diff --quiet && $(GIT) diff --staged --quiet || $(GIT) commit -m "Build from $(HASH)"
	$(GIT) push origin gh-pages
