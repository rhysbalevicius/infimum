FROM ubuntu:20.04
LABEL Rhys Balevicius "rhys@apollos.tech"

ENV TZ=America/New_York
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

# Install project dependencies
RUN apt-get update -y && apt-get install -y \
    build-essential \
    clang \
    curl \
    git \
    lsof \
    libssl-dev \
    openssh-client \
    protobuf-compiler \
    ssh \
    sudo \
    systemd \
    tmux \
    vim

# Vagrant setup
RUN useradd --create-home -s /bin/bash vagrant \
    && echo -n 'vagrant:vagrant' | chpasswd \
    && echo 'vagrant ALL = NOPASSWD: ALL' > /etc/sudoers.d/vagrant \
    && chmod 440 /etc/sudoers.d/vagrant \ 
    && mkdir -p /home/vagrant/.ssh \ 
    && chmod 700 /home/vagrant/.ssh \ 
    && echo "ssh-rsa AAAAB3NzaC1yc2EAAAABIwAAAQEA6NF8iallvQVp22WDkTkyrtvp9eWW6A8YVr+kz4TjGYe7gHzIw+niNltGEFHzD8+v1I2YJ6oXevct1YeS0o9HZyN1Q9qgCgzUFtdOKLv6IedplqoPkcmF0aYet2PkEDo3MlTBckFXPITAMzF8dJSIFo9D8HfdOV0IAdx4O7PtixWKn5y2hMNG0zQPyUecp4pzC6kivAIhyfHilFR61RGL+GPXQ2MWZWFYbAGjyiYJnAmCP3NOTd0jMZEnDkbUvxhMmBYSdETk1rRgm+R4LOzFUGaHqHDLKLX+FIPKcF96hrucXzcWyLbIbEgE98OHlnVYCzRdK8jlqm8tehUc9c9WhQ==" > /home/vagrant/.ssh/authorized_keys \ 
    && chmod 600 /home/vagrant/.ssh/authorized_keys \ 
    && chown -R vagrant:vagrant /home/vagrant/.ssh \ 
    && sed -i -e 's/Defaults.*requiretty/#&/' /etc/sudoers \ 
    && sed -i -e 's/\(UsePAM \)yes/\1 no/' /etc/ssh/sshd_config \ 
    && mkdir -p /var/run/sshd

# Node.js setup
ENV NVM_VERSION v0.39.7
ENV NODE_VERSION v20.10.0
ENV NVM_DIR /home/vagrant/nvm
RUN mkdir -p $NVM_DIR
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.1/install.sh | bash
ENV NODE_PATH $NVM_DIR/$NODE_VERSION/lib/node_modules
ENV PATH $NVM_DIR/versions/node/$NODE_VERSION/bin:$PATH
RUN echo "source $NVM_DIR/nvm.sh && \
    nvm install $NODE_VERSION && \
    nvm alias default $NODE_VERSION && \
    nvm use default && \
    source $NVM_DIR/nvm.sh" | bash
RUN echo "source /home/vagrant/nvm/nvm.sh\n" >> /home/vagrant/.bashrc

# show backtraces
ENV RUST_BACKTRACE 1

# Install and setup rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y \
    && echo 'source $HOME/.cargo/env' >> $HOME/.bashrc \
    && $HOME/.cargo/bin/rustup default stable \
    && $HOME/.cargo/bin/rustup update \ 
    && $HOME/.cargo/bin/rustup update nightly \
    && $HOME/.cargo/bin/rustup target add wasm32-unknown-unknown

# Setup frontend template
RUN git clone https://github.com/rhysbalevicius/substrate-front-end-template /home/vagrant/frontend-template \
    && cd /home/vagrant/frontend-template \
    && npm install

# Setup blockchain node template
RUN git clone --branch infimum https://github.com/rhysbalevicius/substrate-node-template /home/vagrant/node-template \
    && cp -r /home/vagrant/data/ substrate-node-template/pallets/infimum \
    && cd /home/vagrant/node-template \
    && $HOME/.cargo/bin/cargo build --release

EXPOSE 22 8000 8080 9944

CMD ["/usr/sbin/sshd", "-D"]