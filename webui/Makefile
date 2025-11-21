API_LOGIN_URL:=http://localhost:3000/auth/login

build:
	npm run build-css
	~/.cargo/bin/trunk build

release:
	npm run build-css
	~/.cargo/bin/trunk build --release

css:
	npm run dev

serve:
	API_LOGIN_URL=$(API_LOGIN_URL) ~/.cargo/bin/trunk serve --port 3030
