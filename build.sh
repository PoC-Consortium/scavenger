#!/bin/bash

# macos specific functions
function install_xcode {
	touch /tmp/.com.apple.dt.CommandLineTools.installondemand.in-progress;
	PROD=$(softwareupdate -l |grep "\*.*Command Line" |tail -n 1 | awk -F"*" '{print $2}' |sed -e 's/^ *//' |tr -d '\n')
	softwareupdate -i "$PROD" --verbose;
}

# common functions
function install_rust {
	if [ ! -f $HOME/.cargo/env ]
	then
		curl https://sh.rustup.rs -o /tmp/rustup.sh
		chmod +x /tmp/rustup.sh
		/tmp/rustup.sh -y
	fi
	source $HOME/.cargo/env
}

function download_sources {
	if [ "${1}" == "" ]
	then
		branch="master"
	else
		branch="${1}"
	fi
	git clone https://github.com/PoC-Consortium/scavenger.git -b "${branch}"
	cd scavenger
	export BUILDSTR=$(git log --pretty=format:'%h' -n 1)
}

function build_and_pack {
	cargo build --release 2> build.log
	if [ $? -eq 0 ]
	then
		echo "build done successfully!"
	fi
	mkdir output
	cp target/release/scavenger output/
	cp config.yaml output/
	cp build.log output/
	tar cfz "scavenger-${OSSTRING}-${BUILDSTR}.tar.gz" output
} 

if [[ $(uname -a) =~ Darwin ]]
then
	OSSTRING="macos$(sw_vers -productVersion)"
	install_xcode
fi

if [[ $(uname -a) =~ Debian ]]
then
	OSSTRING="debian$(cat /etc/debian_version)"
fi

if [[ $(uname -a) =~ Ubuntu ]]
then
	OSSTRING="ubuntu$(lsb_release -r |awk '{print $2}')"
fi

install_rust
download_sources
build_and_pack
