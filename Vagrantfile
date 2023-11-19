# -*- mode: ruby -*-
# vi: set ft=ruby :
# https://docs.vagrantup.com.

Vagrant.configure("2") do |config|

    # Every Vagrant development environment requires a box. You can search for
    # boxes at https://vagrantcloud.com/search.
    config.vm.define "groth" do |groth|
        groth.vm.provider :docker do |d|
            d.build_dir = "."
            d.remains_running = true
            d.has_ssh = true
            d.name = "groth"
        end
    end

    # Create a private network, which allows host-only access to the machine
    # using a specific IP.
    config.vm.network "forwarded_port", guest: 8000, host: 8000
    config.vm.network "forwarded_port", guest: 8080, host: 8080
    config.vm.network "forwarded_port", guest: 9944, host: 9944
    config.vm.hostname = "groth"

    # Share an additional folder to the guest VM. The first argument is
    # the path on the host to the actual folder. The second argument is
    # the path on the guest to mount the folder. And the optional third
    # argument is a set of non-required options.
    config.vm.synced_folder "./.shared", "/home/vagrant/data/"
end
  