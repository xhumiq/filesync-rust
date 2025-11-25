ENV_FILE:=.env.local

build:
	npm run build-css
	ENV_PROFILE=$(ENV_FILE) ~/.cargo/bin/trunk build

release:
	npm run build-css
	~/.cargo/bin/trunk build --release -vv

css:
	npm run dev

serve:
	ENV_PROFILE=$(ENV_FILE) ~/.cargo/bin/trunk serve --port 3030
