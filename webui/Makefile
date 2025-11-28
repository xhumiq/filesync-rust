ENV_LOCAL:=.env.local
ENV_RELEASE:=.env.release
PROFILE:=$(ENV_LOCAL)

build:
	npm run build-css
	rm -rf ./dist/debug
	cp .env.local .env
	~/.cargo/bin/trunk build --dist ./dist/debug

release:
	npm run build-css
	rm -rf ./dist/release
	cp .env.release .env
	~/.cargo/bin/trunk build --release -vv --minify --dist ./dist/release
	cp .env.local .env

css:
	npm run dev

serve:
	npm run build-css
	rm -rf ./dist/debug
	cp .env.local .env
	~/.cargo/bin/trunk serve --port 3030 --dist ./dist/debug

serve-with-profile:
	@echo "Usage: make serve-with-profile PROFILE=.env.development"
	~/.cargo/bin/trunk serve --port 3030 --dist ./dist/debug
