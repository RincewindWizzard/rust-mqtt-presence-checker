# mqtt-presence-checker

Check if you (or your phone) is at home and notify your smarthome via mqtt.
You can configure this daemon via a toml file in _/etc/mqtt-presence-checker/mqtt-presence-checker.conf_.

This is rather rudimentary and might crash or behave strange. Feel free
to [fork me on github](https://github.com/RincewindWizzard/rust-mqtt-presence-checker) and send a PR if you find any
bug!

## Building

First you need to install [Rust](https://www.rust-lang.org/tools/install).
Then you can build with:

    $ cargo build --release

## Configuration

Configuration is done via _/etc/mqtt-presence-checker/mqtt-presence-checker.conf_:

    [minuterie]
    timeout = 60
    
    [mqtt]
    host = 'example.org'
    username = '<username>'
    password = '<password>'
    port = 1883
    heartbeat_topic = 'presence-checker/home/heartbeat'
    publish_topic = 'presence-checker/home/presence'

    [[ping.hosts]]
    host = '192.168.178.1'
    interval = 60
    
    [[ping.hosts]]
    host = '192.168.178.2'
    interval = 1

Create a system user and group for this daemon:

    $ sudo groupadd -r mqtt-presence-checker
    $ sudo useradd -r -g mqtt-presence-checker -s /bin/false -M mqtt-presence-checker

Create the log file

    $ sudo touch /var/log/mqtt-presence-checker.log
    $ sudo chown mqtt-presence-checker:mqtt-presence-checker /var/log/mqtt-presence-checker.log

Create a systemd unit file to always run it in the background.

_/etc/systemd/system/mqtt-presence-checker.service_:

    [Unit]
    Description=MQTT Presence Checker
    After=network.target
    
    [Service]
    Type=simple
    ExecStart=mqtt-presence-checker -vv -c /etc/mqtt-presence-checker/mqtt-presence-checker.conf
    Restart=always
    RestartSec=5
    User=mqtt-presence-checker
    Group=mqtt-presence-checker
    StandardOutput=file:/var/log/mqtt-presence-checker.log
    StandardError=file:/var/log/mqtt-presence-checker.log
    
    [Install]
    WantedBy=default.target

To activate the service run the following:

    $ sudo systemctl daemon-reload
    $ sudo systemctl enable mqtt-presence-checker
    $ sudo systemctl start mqtt-presence-checker


