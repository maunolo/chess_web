FROM rustlang/rust:nightly

# Install node
RUN curl -sL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && npm i -g yarn

# Leptos dependencies
RUN rustup target add wasm32-unknown-unknown \
    && cargo install -f cargo-leptos

# Create app directory
WORKDIR /usr/src/app

# Bundle app source
COPY . .

# Install deps
RUN yarn

# Build
RUN yarn build

# Start
CMD [ "yarn", "start" ]
